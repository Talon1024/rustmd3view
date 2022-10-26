mod md3;
mod window;
mod res;
mod eye;
mod render;

use eye::{Camera, OrbitCamera};
use glow::{Context as GLContext, HasContext};
use glutin::event_loop::{EventLoopBuilder, ControlFlow};
use glutin::event::Event;
use res::AppResources;
use std::{
	collections::HashMap,
	error::Error,
	f32::consts::FRAC_PI_2,
	fs::File,
	sync::Arc,
	ops::RangeInclusive,
	time::{Instant/* , Duration */},
};
use anyhow::Error as AError;
use md3::MD3Model;
use render::{Vertex, VertexBuffer, IndexBuffer, Texture, ShaderProgram, ShaderStage};

use egui_file::FileDialog;

#[derive(Debug, Clone, Copy, Default)]
struct AppControls {
	lmb_dragging: bool,
}

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
	controls: AppControls,
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
			controls: AppControls::default(),
			camera: OrbitCamera::default(),
			uniform_locations: HashMap::default(),
		}
	}
}

const MOUSE_FACTOR: f32 = 0.0078125; // 1./128
const LOOK_LIMIT: f32 = {
	use std::mem;
	let v = unsafe{mem::transmute::<f32, u32>(FRAC_PI_2)};
	// It's a pain in the butt having to generate this code... But it's all
	// done at compile time, so there are no runtime costs.
	/* 
	Shell (zsh) code used to generate this mess:
	(bits=32
	for bit in {0..$((bits-1))}; do
		print -v hxb -f "%0$((bits/4))X" $((1 << bit))
		if ((bit > 0)); then
			print -n "else "
		fi
		print "if v & $hxb != 0 { $hxb }"
	done
	print "else { 0 };")
	 */
	let lowest_bit = if v & 00000001 != 0 { 00000001 }
	else if v & 00000002 != 0 { 00000002 }
	else if v & 00000004 != 0 { 00000004 }
	else if v & 00000008 != 0 { 00000008 }
	else if v & 00000010 != 0 { 00000010 }
	else if v & 00000020 != 0 { 00000020 }
	else if v & 00000040 != 0 { 00000040 }
	else if v & 00000080 != 0 { 00000080 }
	else if v & 00000100 != 0 { 00000100 }
	else if v & 00000200 != 0 { 00000200 }
	else if v & 00000400 != 0 { 00000400 }
	else if v & 00000800 != 0 { 00000800 }
	else if v & 00001000 != 0 { 00001000 }
	else if v & 00002000 != 0 { 00002000 }
	else if v & 00004000 != 0 { 00004000 }
	else if v & 00008000 != 0 { 00008000 }
	else if v & 00010000 != 0 { 00010000 }
	else if v & 00020000 != 0 { 00020000 }
	else if v & 00040000 != 0 { 00040000 }
	else if v & 00080000 != 0 { 00080000 }
	else if v & 00100000 != 0 { 00100000 }
	else if v & 00200000 != 0 { 00200000 }
	else if v & 00400000 != 0 { 00400000 }
	else if v & 00800000 != 0 { 00800000 }
	else if v & 01000000 != 0 { 01000000 }
	else if v & 02000000 != 0 { 02000000 }
	else if v & 04000000 != 0 { 04000000 }
	else if v & 08000000 != 0 { 08000000 }
	else if v & 10000000 != 0 { 10000000 }
	else if v & 20000000 != 0 { 20000000 }
	else if v & 40000000 != 0 { 40000000 }
	else if v & 80000000 != 0 { 80000000 }
	else { 0 };
	unsafe{mem::transmute::<u32, f32>(v ^ lowest_bit)}
};

fn main() -> Result<(), Box<dyn Error>> {
	let el = EventLoopBuilder::new().build();
	let (wc, glc) = window::create_window(&el, None);
	let glc = Arc::new(glc);
	let mut egui_glow = egui_glow::EguiGlow::new(&el, Arc::clone(&glc));
	let mut app = App::new();
	let app_res = AppResources::try_load(None)?;
	app.model_tx = Some(Texture::try_from_texture(Arc::clone(&glc), &app_res.null_texture)?);
	app.model_sd = Some({
		let mut sdr = ShaderProgram::new(Arc::clone(&glc))?;
		sdr.add_shader(ShaderStage::Vertex, &app_res.md3_vertex_shader)?;
		sdr.add_shader(ShaderStage::Fragment, &app_res.md3_pixel_shader)?;
		sdr.prepare()?;
		["anim", "eye", "frame", "tex"].into_iter().for_each(|uname|{
			let uloc = unsafe { glc.get_uniform_location(sdr.prog(), uname) };
			if let Some(uloc) = uloc {
				app.uniform_locations.insert(uname.to_string(), uloc);
			}
		});
		sdr
	});
	app.camera.aspect = {
		let logical_size = wc.window().inner_size().to_logical::<f32>(wc.window().scale_factor());
		logical_size.width / logical_size.height
	};
	let _app_start = Instant::now();
	unsafe { glc.clear_color(0., 0., 0., 1.); }
	el.run(move |event, _window, control_flow| {
		match event {
			Event::WindowEvent { window_id: _, event } => {
				use glutin::event::{
					WindowEvent::*,
					MouseButton,
					ElementState,
				};
				if egui_glow.on_event(&event) {
					return ();
				}
				match event {
					CloseRequested => {
						*control_flow = ControlFlow::ExitWithCode(0);
					},
					Resized(new_size) => {
						let logical_size = new_size.to_logical::<f32>(wc.window().scale_factor());
						app.camera.aspect = logical_size.width / logical_size.height;
					},
					MouseInput {state, button, .. } => {
						if button == MouseButton::Left {
							app.controls.lmb_dragging = match state {
								ElementState::Pressed => true,
								ElementState::Released => false,
							};
						}
					},
					CursorLeft{..} => {
						app.controls.lmb_dragging = false;
					},
					_ => (),
				}
			},
			Event::DeviceEvent {event, ..} => {
				use glutin::event::DeviceEvent::*;
				if !app.controls.lmb_dragging { return; }
				match event {
					MouseMotion { delta: (dx, dy) } => {
						let dx = dx as f32 * MOUSE_FACTOR;
						let dy = dy as f32 * MOUSE_FACTOR;
						app.camera.position.x += dx;
						app.camera.position.y -= dy;
						app.camera.position.y = app.camera.position.y.clamp(-LOOK_LIMIT, LOOK_LIMIT);
					},
					_ => ()
				}
			}
			Event::MainEventsCleared => {
// DRAW MODEL
// ==========================
unsafe {
	glc.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
	glc.enable(glow::DEPTH_TEST);
	if app.model.is_some() {
		app.model_sd.as_ref().unwrap().activate().unwrap();

		let mut texture = app_res.null_texunit;

		glc.active_texture(texture.gl_id());
		glc.bind_texture(glow::TEXTURE_2D, Some(app.model_tx.as_ref().unwrap().tex()));
		glc.uniform_1_i32(app.uniform_locations.get("tex"), texture.gl_uniform());

		*texture += 1;

		glc.active_texture(texture.gl_id());
		glc.bind_texture(glow::TEXTURE_2D, Some(app.model_an.as_ref().unwrap().tex()));
		glc.uniform_1_i32(app.uniform_locations.get("anim"), texture.gl_uniform());

		glc.uniform_1_f32(app.uniform_locations.get("frame"), app.current_frame);

		glc.uniform_matrix_4_f32_slice(app.uniform_locations.get("eye"), false, app.camera.view_projection().as_ref());

		if let Err(e) = render::render(
			app.model_vb.as_ref().unwrap(),
			app.model_ib.as_ref().unwrap()) {
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
					ui.spacing_mut().slider_width = 200.;
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
				app.current_frame = 0.;
				app.model = Some(Box::new(model));
				if let Some(surf) = app.model.as_ref().unwrap().surfaces.get(0) {
					app.model_vb = Some(VertexBuffer::from_surface(Arc::clone(&glc), surf));
					app.model_vb.as_mut().unwrap().upload().unwrap();
					app.model_ib = Some(IndexBuffer::from_surface(Arc::clone(&glc), surf));
					app.model_ib.as_mut().unwrap().upload().unwrap();
					app.model_an = Texture::try_from_texture(Arc::clone(&glc), &surf.make_animation_texture()).ok();
					app.camera.position.z = -app.model.as_ref().unwrap().max_radius() * 2.;
				}
				Ok(())
			}) {
				app.error_message = Some(format!("Error reading file {}:\n{}", fpath.display(), e));
			}
		}
	}
	egui::SidePanel::right("infoz").show(ctx, |ui| {
		ui.heading("Shaders");
		if let Some(model) = app.model.as_ref() {
			model.surfaces.iter().enumerate().for_each(|(index, surf)| {
				egui::CollapsingHeader::new(format!("Surface {}", index)).show(ui, |ui| {
					surf.shaders.iter().for_each(|sdr| {
						ui.label(String::from_utf8_lossy(&sdr.name));
					});
				});
			});
		}
	});
	/* egui::Window::new("camera position").show(ctx, |ui| {
		ui.label(format!("longitude {}\nlatitude {}", app.camera.position.x, app.camera.position.y));
	}); */
});
egui_glow.paint(wc.window());
// SWAP BUFFERS
// ============================
if let Err(e) = wc.swap_buffers() {
	eprintln!("{:?}", e);
}
			},
			_ => ()
		}
	});
}
