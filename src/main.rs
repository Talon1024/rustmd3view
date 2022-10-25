mod md3;
mod window;
mod res;
mod eye;
mod render;

use eye::{Camera, OrbitCamera};
use glow::{Context as GLContext, HasContext};
use glutin::event_loop::{EventLoopBuilder, ControlFlow};
use glutin::event::{Event, WindowEvent};
use res::AppResources;
use std::{
	collections::HashMap,
	error::Error,
	fs::File,
	sync::Arc,
	ops::RangeInclusive,
	time::{Instant/* , Duration */},
};
use anyhow::Error as AError;
use md3::MD3Model;
use render::{Vertex, VertexBuffer, IndexBuffer, Texture, ShaderProgram, ShaderStage};

use egui_file::FileDialog;

struct App {
	open_file_dialog: FileDialog,
	model: Option<Box<MD3Model>>,
	current_frame: f32,
	anim_playing: bool,
	frame_range: Option<RangeInclusive<f32>>,
	error_message: Option<String>,
	model_vb: Option<VertexBuffer<Vertex>>,
	model_ib: Option<IndexBuffer<u32>>,
	model_tx: Option<Texture>,
	model_an: Option<Texture>,
	model_sd: Option<ShaderProgram>,
	camera: OrbitCamera,
	uniform_locations: HashMap<String, <GLContext as HasContext>::UniformLocation>,
}

impl App {
	fn new() -> Self {
		App {
			open_file_dialog: FileDialog::open_file(None)
				.show_rename(false)
				.show_new_folder(false)
				.filter(String::from("md3")),
			model: None,
			current_frame: 0.,
			anim_playing: false,
			frame_range: None,
			error_message: None,
			model_vb: None,
			model_ib: None,
			model_tx: None,
			model_an: None,
			model_sd: None,
			camera: OrbitCamera::default(),
			uniform_locations: HashMap::default(),
		}
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	let el = EventLoopBuilder::new().build();
	let (wc, glc) = window::create_window(&el, None);
	let glc = Arc::new(glc);
	let mut egui_glow = egui_glow::EguiGlow::new(&el, Arc::clone(&glc));
	let mut app = App::new();
	let app_res = AppResources::try_load(None)?;
	app.model_tx = Some(Texture::try_from_texture(Arc::clone(&glc), &app_res.null_texture, app_res.null_texunit)?);
	app.model_sd = Some({
		let mut sdr = ShaderProgram::new(Arc::clone(&glc))?;
		sdr.add_shader(ShaderStage::Vertex, &app_res.md3_vertex_shader)?;
		sdr.add_shader(ShaderStage::Fragment, &app_res.md3_pixel_shader)?;
		sdr.prepare()?;
		["anim", "eye", "frame", "tex"].into_iter().for_each(|uname|{
			app.uniform_locations.entry(uname.to_string()).or_insert_with(|| {unsafe{
				glc.get_uniform_location(sdr.prog(), uname).unwrap()
			}});
		});
		sdr
	});
	let first_texunit = {
		let mut u = app_res.null_texunit.clone();
		*u += 1;
		u
	};
	let _app_start = Instant::now();
	unsafe { glc.clear_color(0., 0., 0., 1.); }
	el.run(move |event, _window, control_flow| {
		match event {
			Event::WindowEvent { window_id: _, event } => {
				if egui_glow.on_event(&event) {
					return ();
				}
				match event {
					WindowEvent::CloseRequested => {
						*control_flow = ControlFlow::ExitWithCode(0);
					},
					_ => (),
				}
			},
			Event::MainEventsCleared => {
				// DRAW MODEL
				// ==========================
				unsafe {
					glc.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
					glc.enable(glow::DEPTH_TEST);
					if app.model.is_some() && app.model_vb.is_some() && app.model_ib.is_some() {
						if let Err(e) = render::render(
							app.model_vb.as_ref().unwrap(),
							app.model_ib.as_ref().unwrap(), || {

								app.model_sd.as_ref().unwrap().activate().unwrap();

								glc.active_texture(app_res.null_texunit.gl_id());
								glc.bind_texture(glow::TEXTURE_2D, Some(app.model_tx.as_ref().unwrap().tex()));
								glc.uniform_1_u32(Some(app.uniform_locations.get("tex").unwrap()), app_res.null_texunit.gl_u());

								let texture = first_texunit;

								glc.active_texture(texture.gl_id());
								glc.bind_texture(glow::TEXTURE_2D, Some(app.model_an.as_ref().unwrap().tex()));
								glc.uniform_1_u32(Some(app.uniform_locations.get("anim").unwrap()), texture.gl_u());

								glc.uniform_1_f32(app.uniform_locations.get("frame"), app.current_frame);

								glc.uniform_matrix_4_f32_slice(app.uniform_locations.get("eye"), false, app.camera.view_projection().as_ref());
							}) {
							eprintln!("{:?}", e);
						}
					}
				}
				// DRAW EGUI
				// ==========================
				egui_glow.run(wc.window(), |ctx| {
egui::TopBottomPanel::top("menu_bar").show(&ctx, |ui| {
	egui::menu::bar(ui, |ui| {
		ui.menu_button("File", |ui| {
			if ui.button("Open").clicked() {
				app.open_file_dialog.open();
				ui.close_menu();
			}
			if ui.button("Quit").clicked() {
				ui.close_menu();
				*control_flow = ControlFlow::ExitWithCode(0);
			}
		});
	});
});
egui::TopBottomPanel::bottom("frame_bar").show(&ctx, |ui| {
	let play_button_text = match app.anim_playing {
		true => "⏸",
		false => "▶",
	};
	// let time = (Instant::now() - app_start).as_secs_f32();
	match app.frame_range {
		Some(ref range) => {
			ui.horizontal(|ui| {
				if ui.button(play_button_text).clicked() {
					app.anim_playing = !app.anim_playing;
				}
				ui.add(egui::Slider::new(&mut app.current_frame, range.clone()));
			});
		},
		None => ()
	}
});
let error_window = egui::Window::new("Error");
if let Some(message) = app.error_message.clone() {
	error_window.show(ctx, |ui| {
		ui.label(message);
		if ui.button("OK").clicked() {
			app.error_message = None;
		}
	});
}
app.open_file_dialog.show(&ctx);
if app.open_file_dialog.selected() {
	if let Some(fpath) = app.open_file_dialog.path() {
		if let Err(e) = File::open(&fpath)
			.map_err(AError::from).and_then(|mut f| {
			md3::read_md3(&mut f).map_err(AError::from)
		}).and_then(|model| {
			let num_frames = model.frames.len();
			app.frame_range = if num_frames > 1 {
				Some(RangeInclusive::new(0., (num_frames - 1) as f32))
			} else {
				None
			};
			app.model = Some(Box::new(model));
			if let Some(surf) = app.model.as_ref().unwrap().surfaces.get(0) {
				app.model_vb = Some(VertexBuffer::<Vertex>::from_surface(Arc::clone(&glc), surf));
				app.model_vb.as_mut().unwrap().upload().unwrap();
				app.model_ib = Some(IndexBuffer::<u32>::from_surface(Arc::clone(&glc), surf));
				app.model_ib.as_mut().unwrap().upload().unwrap();
				app.model_an = Texture::try_from_texture(Arc::clone(&glc), &surf.make_animation_texture(), first_texunit).ok();
				app.camera.position.z = -app.model.as_ref().unwrap().max_radius() * 2.;
			}
			Ok(())
		}) {
			app.error_message = Some(format!("Error reading file {}:\n{}", fpath.display(), e));
		}
	}
}
				});
				egui_glow.paint(wc.window());
				// SWAP BUFFERS
				// ============================
				if let Err(e) = wc.swap_buffers() {
					eprintln!("{:?}", e);
				}
			}
			_ => ()
		}
	});
}
