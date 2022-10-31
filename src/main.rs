mod md3;
mod window;
mod res;
mod eye;
mod render;
mod eutil;

use ahash::RandomState;
use egui::{Color32, LayerId, TextStyle, Order, Pos2, Id};
use eye::{Camera, OrbitCamera};
use glam::{Vec3, Mat4};
use glow::{Context as GLContext, HasContext};
use glutin::event_loop::{EventLoopBuilder, ControlFlow};
use glutin::event::Event;
use res::{AppResources, Surface};
use std::{
	borrow::Cow,
	cell::RefCell,
	collections::HashMap,
	error::Error,
	f32::consts::FRAC_PI_2,
	ffi::OsString,
	fs::File,
	sync::Arc,
	ops::{RangeInclusive, RangeBounds, /* Add, Mul */},
	path::Path,
	rc::Rc,
	time::Instant,
};
use anyhow::Error as AError;
use md3::MD3Model;
use render::{BasicModel, BufferModel, VertexBuffer, IndexBuffer, Texture, ShaderProgram, TextureUnit, ShaderStage};

use egui_file::FileDialog;

struct TextureCache {
	cache: HashMap<String, Rc<Texture>, RandomState>,
}

const NULL_TEXTURE_NAME: &str = "__null_texture__";

impl TextureCache {
	fn new(glc: Arc<GLContext>, null_texture: &Surface) -> Self {
		let mut cache = HashMap::default();
		cache.insert(String::from(NULL_TEXTURE_NAME), Rc::new(Texture::try_from_surface(glc, null_texture).unwrap()));
		Self { cache }
	}
	fn get(&mut self, glc: Arc<GLContext>, path: &dyn AsRef<Path>) -> (Rc<Texture>, Option<Box<dyn Error>>) {
		let null_key = Cow::from(NULL_TEXTURE_NAME);
		let path = path.as_ref();
		let key = path.to_string_lossy();
		if let Some(r) = self.cache.get(key.as_ref()) {
			return (Rc::clone(r), None);
		}
		match Surface::read_png(path) {
			Ok(s) => {
				let texture = Texture::try_from_surface(glc, &s);
				match texture {
					Ok(t) => {
						let txref = Rc::new(t);
						let myref = Rc::clone(&txref);
						let path = path.to_string_lossy();
						self.cache.insert(path.into_owned(), txref);
						(myref, None)
					},
					Err(e) => {
						(Rc::clone(self.cache.get(null_key.as_ref()).as_ref().unwrap()), Some(format!("Could not load texture {}: {:?}", path.display(), e).into()))
					},
				}
			},
			Err(e) => {
				(Rc::clone(self.cache.get(null_key.as_ref()).as_ref().unwrap()), Some(format!("Could not load texture {}: {:?}", path.display(), e).into()))
			},
		}
	}
	fn clear(&mut self) {
		let non_null_textures: Box<[String]> = self.cache.keys().cloned()
			.filter(|f| f != NULL_TEXTURE_NAME).collect();
		non_null_textures.into_iter().map(String::as_str)
			.for_each(|k| {self.cache.remove(k);});
	}
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u32)]
enum ViewMode {
	#[default]
	Textured,
	Untextured,
	Normals,
}

#[derive(Debug, Clone, Copy, Default)]
struct AppControls {
	lmb_dragging: bool,
	rmb_dragging: bool,
	view_mode: ViewMode,
	gzdoom_normals: bool,
}

struct App {
	open_file_dialog: FileDialog,
	model_data: Option<Box<MD3Model>>,
	current_frame: f32,
	anim_playing: bool,
	frame_range: Option<RangeInclusive<f32>>,
	error_message: Option<String>,
	models: Vec<BufferModel<u32>>,
	axes: BasicModel<u8>,
	camera: OrbitCamera,
	controls: AppControls,
	texture_cache: TextureCache,
}

impl App {
	fn new(res: &AppResources, glc: &Arc<GLContext>) -> Self {
		App {
			open_file_dialog: FileDialog::open_file(None)
				.show_rename(false)
				.show_new_folder(false)
				.filter(String::from("md3")),
			model_data: None,
			current_frame: 0.,
			anim_playing: false,
			frame_range: None,
			error_message: None,
			models: vec![],
			axes: BasicModel {
				vertex: VertexBuffer::new(Arc::clone(glc), Box::new(res::AXES_V)),
				index: IndexBuffer::new(Arc::clone(glc), Vec::from(res::AXES_I)),
				shader: {
					let mut sp = ShaderProgram::new(Arc::clone(glc)).unwrap();
					sp.add_shader(ShaderStage::Vertex, &res.res_vertex_shader).unwrap();
					sp.add_shader(ShaderStage::Fragment, &res.res_pixel_shader).unwrap();
					sp.prepare().unwrap();
					Rc::new(RefCell::new(sp))
				},
			},
			controls: AppControls::default(),
			camera: OrbitCamera::default(),
			texture_cache: TextureCache::new(Arc::clone(glc), &res.null_surface),
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
	let app_res = AppResources::try_load(None)?;
	let el = EventLoopBuilder::new().build();
	let (wc, glc) = window::create_window(&el, None);
	let glc = Arc::new(glc);
	let mut egui_glow = egui_glow::EguiGlow::new(&el, Arc::clone(&glc));
	let mut app = App::new(&app_res, &glc);
	let md3_shader = Rc::new(RefCell::new({
		let mut sdr = ShaderProgram::new(Arc::clone(&glc))?;
		sdr.add_shader(ShaderStage::Vertex, &app_res.md3_vertex_shader)?;
		sdr.add_shader(ShaderStage::Fragment, &app_res.md3_pixel_shader)?;
		sdr.prepare()?;
		sdr
	}));
	app.camera.aspect = {
		let logical_size = wc.window().inner_size().to_logical::<f32>(wc.window().scale_factor());
		logical_size.width / logical_size.height
	};
	let mut window_size = wc.window().inner_size().to_logical::<f32>(wc.window().scale_factor());
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
						window_size = new_size.to_logical::<f32>(wc.window().scale_factor());
						app.camera.aspect = window_size.width / window_size.height;
					},
					MouseInput {state, button, .. } => {
						match button {
							MouseButton::Left => {
							app.controls.lmb_dragging = match state {
								ElementState::Pressed => true,
								ElementState::Released => false,
							};
							},
							MouseButton::Right => {
							app.controls.rmb_dragging = match state {
								ElementState::Pressed => true,
								ElementState::Released => false,
							};
							},
							_ => (),
						}
					},
					CursorLeft{..} => {
						app.controls.lmb_dragging = false;
						app.controls.rmb_dragging = false;
					},
					_ => (),
				}
			},
			Event::DeviceEvent {event, ..} => {
				use glutin::event::DeviceEvent::*;
				if app.controls.lmb_dragging {
				match event {
					MouseMotion { delta: (dx, dy) } => {
						let dx = dx as f32 * MOUSE_FACTOR;
						let dy = dy as f32 * MOUSE_FACTOR;
						app.camera.longtude += dx;
						app.camera.latitude -= dy;
						app.camera.latitude = app.camera.latitude.clamp(-LOOK_LIMIT, LOOK_LIMIT);
					},
					_ => ()
				}
				}
				if app.controls.rmb_dragging {
				match event {
					MouseMotion { delta: (_dx, dy) } => {
						let dy = dy as f32 * MOUSE_FACTOR * app.camera.distance.max(1.);
						app.camera.distance += dy;
					},
					_ => (),
				}
				}
			}
			Event::MainEventsCleared => {
// CLEAR SCREEN BEFORE DRAWING ANYTHING
// ==================================================================
unsafe {
	glc.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
	glc.enable(glow::DEPTH_TEST);
}
// DRAW MODELS
// ==================================================================
unsafe {
	glc.depth_func(glow::LESS);
	app.models.iter().for_each(|model| {
		model.base.shader.borrow().activate().unwrap();
		let mut texture = TextureUnit(1);

		glc.active_texture(texture.gl_id());
		glc.bind_texture(glow::TEXTURE_2D, Some(model.skin.tex()));
		glc.uniform_1_i32(model.base.shader.borrow_mut().uniform_location(Cow::from("tex")).as_ref(), texture.gl_u());

		*texture += 1;

		glc.active_texture(texture.gl_id());
		glc.bind_texture(glow::TEXTURE_2D, model.animation.as_ref().map(Texture::tex));
		glc.uniform_1_i32(model.base.shader.borrow_mut().uniform_location(Cow::from("anim")).as_ref(), texture.gl_u());

		glc.uniform_1_f32(model.base.shader.borrow_mut().uniform_location(Cow::from("frame")).as_ref(), app.current_frame);

		glc.uniform_1_u32(model.base.shader.borrow_mut().uniform_location(Cow::from("mode")).as_ref(), app.controls.view_mode as u32);

		glc.uniform_1_u32(model.base.shader.borrow_mut().uniform_location(Cow::from("gzdoom")).as_ref(), if app.controls.gzdoom_normals {1} else {0});

		glc.uniform_matrix_4_f32_slice(model.base.shader.borrow_mut().uniform_location(Cow::from("eye")).as_ref(), false, app.camera.view_projection().as_ref());

		if let Err(e) = render::render(
			&glc,
			&model.base.vertex,
			&model.base.index) {
			eprintln!("{:?}", e);
		}
	});
}
// DRAW AXES
// ==================================================================
unsafe {
	glc.depth_func(glow::ALWAYS);
	app.axes.shader.borrow().activate().unwrap();
	let mvp = {
		let eye = Vec3::new(
			app.camera.longtude.cos() * app.camera.latitude.cos(),
			app.camera.longtude.sin() * app.camera.latitude.cos(),
			app.camera.latitude.sin(),
		) * -60.;
		// 160 pixels left from top right corner, 80 pixels down from top right corner
		let trans = Mat4::from_translation(Vec3::new(1.0 - (320./window_size.width), 1.0 - (160./window_size.height), 0.));
		let scale = Mat4::from_scale(Vec3::new(0.125, 0.125, 0.125));
		let view = Mat4::look_at_lh(eye, Vec3::ZERO, Vec3::Z);
		let proj = Mat4::perspective_lh(app.camera.fov, app.camera.aspect, 0.25, 512.);
		trans * proj * view * scale
	};
	glc.uniform_matrix_4_f32_slice(app.axes.shader.borrow_mut().uniform_location(Cow::from("eye")).as_ref(), false, mvp.as_ref());
	if let Err(e) = render::render(
		&glc,
		&app.axes.vertex,
		&app.axes.index) {
		eprintln!("{:?}", e);
	}
}
// DRAW EGUI
// ==================================================================
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
			ui.menu_button("View", |ui| {
				if ui.radio_value(&mut app.controls.view_mode,
					ViewMode::Textured, "Textured").clicked() ||
					ui.radio_value(&mut app.controls.view_mode,
						ViewMode::Untextured, "Untextured").clicked() ||
					ui.radio_value(&mut app.controls.view_mode,
						ViewMode::Normals, "Normals").clicked()
				{ ui.close_menu(); }
				if ui.checkbox(&mut app.controls.gzdoom_normals, "GZDoom normals").clicked() { ui.close_menu(); }
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
					if app.anim_playing {
						app.current_frame = if let std::ops::Bound::Included(&fc) = range.end_bound() {(Instant::now() - _app_start).as_secs_f32() % fc} else {0.};
					}
					ui.spacing_mut().slider_width = 400.;
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
				#[cfg(feature = "log_successful_load")]
				println!("Model {} loaded successfully!", fpath.display());
				let num_frames = model.frames.len();
				app.frame_range = if num_frames > 1 {
					Some(RangeInclusive::new(0., (num_frames - 1) as f32))
				} else {
					None
				};
				app.texture_cache.clear();
				app.anim_playing = false;
				app.current_frame = 0.;
				app.model_data = Some(Box::new(model));
				app.camera.distance = app.model_data.as_ref().unwrap().max_radius() * 2.;
				app.models = app.model_data.as_ref().unwrap().surfaces
				.iter().map(|surf| {
					let vb = VertexBuffer::from_surface(Arc::clone(&glc), surf);
					let ib = IndexBuffer::from_surface(Arc::clone(&glc), surf);
					let an = Texture::try_from_surface(Arc::clone(&glc), &surf.make_animation_surface()).map_err(|e| {app.error_message = Some(e.to_string()); e}).ok();
					BufferModel {
						base: BasicModel {
							vertex: vb,
							index: ib,
							shader: Rc::clone(&md3_shader),
						},
						skin: {
					let (texture, error) = app.texture_cache.get(Arc::clone(&glc), &surf.shaders.get(0).map(|s|
						Cow::from(OsString::from(fpath.parent().unwrap_or(&fpath).join(
						String::from_utf8_lossy(&s.name)
						.trim_matches(|c| c == char::from_u32(0).unwrap())
						.trim())))
					).unwrap_or(Cow::from(OsString::new())));
					if let Some(e) = error {
						match &mut app.error_message {
							Some(ee) => {
								ee.push('\n');
								ee.push_str(&e.to_string());
							},
							None => {
								app.error_message = Some(e.to_string());
							},
						}
					}
					texture
						},
						animation: an,
					}
				}).collect();
				Ok(())
			}) {
				app.error_message = Some(format!("Error reading file {}:\n{}", fpath.display(), e));
			}
		}
	}
	egui::SidePanel::right("infoz").show(ctx, |ui| {
		ui.heading("Shaders");
		if let Some(model) = app.model_data.as_ref() {
			model.surfaces.iter().enumerate().for_each(|(index, surf)| {
				egui::CollapsingHeader::new(format!("Surface {}", index)).show(ui, |ui| {
					surf.shaders.iter().for_each(|sdr| {
						ui.label(String::from_utf8_lossy(&sdr.name));
					});
				});
			});
		}
	});
	// DRAW TAG NAMES AT TAG POSITIONS
	// ==================================================================
	if !app.open_file_dialog.visible(){
	let painter = ctx.layer_painter(
		LayerId { order: Order::Foreground, id: Id::new("tag_name_overlays") });
	if let Some(model) = app.model_data.as_ref() {
		let current_frame = app.current_frame.trunc() as usize;
		let num_tags = model.num_tags;
		(0..num_tags).for_each(|tag_index| {
			let tag_index = tag_index + num_tags * current_frame;
			let tag = &model.tags[tag_index];
			let tag_name = String::from_utf8_lossy(&tag.name).to_string();
			let font = egui::style::default_text_styles()[&TextStyle::Small].clone();
			let galley = painter.layout_no_wrap(tag_name, font, Color32::WHITE);
			let pos = {
				let pos = app.camera.view_projection().project_point3(tag.origin);
				let Vec3 {x, y, ..} = pos;
				let x = x.mul_add(0.5, 0.5) * window_size.width;
				// In OpenGL NDC, +y is up and -y is down
				let y = (-y).mul_add(0.5, 0.5) * window_size.height;
				Pos2 {x, y}
			};
			painter.galley(pos, galley);
		});
	}}
});
egui_glow.paint(wc.window());
// SWAP BUFFERS
// ==================================================================
if let Err(e) = wc.swap_buffers() {
	eprintln!("{:?}", e);
}
			},
			_ => ()
		}
	});
}
/* 
#[inline]
fn lerp<T>(a: T, b: T, f: f32) -> T where T : Add<T> + Mul<f32>, <T as Mul<f32>>::Output: Add<T> {
	a * (1. - f) + b * f
}
 */