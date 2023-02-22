mod err_util;
mod eye;
mod md3;
mod platform;
mod render;
mod res;
mod str_util;

use ahash::RandomState;
use anyhow::{Context as AContext, Error as AError};
use egui::{Color32, Id, LayerId, Order, Pos2, TextStyle};
use eye::{Camera, OrbitCamera};
use glam::{Affine3A, Mat4, Vec3};
use glow::{Context as GLContext, HasContext};
use instant::Instant;
use md3::MD3Model;
use render::{
    BasicModel, IndexBuffer, ShaderProgramBuilder, ShaderStage, Texture,
    UniformsMD3, UniformsMD3Locations, UniformsRes, UniformsResLocations,
    VertexBuffer, VertexMD3,
};
use res::{AppResources, Surface};
use rfd::AsyncFileDialog;
use std::{
    borrow::Cow,
    collections::HashMap,
    env,
    f32::consts::FRAC_PI_2,
    ffi::OsString,
    fs::File,
    io::Cursor,
    ops::{Add, Bound, Mul, RangeBounds, RangeInclusive},
    path::Path,
    rc::Rc,
    sync::Arc,
};
use str_util::StringFromBytes;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};

struct TextureCache {
    cache: HashMap<String, Rc<Texture>, RandomState>,
}

const NULL_TEXTURE_NAME: &str = "__null_texture__";

impl TextureCache {
    fn new(glc: Arc<GLContext>, null_texture: &Surface) -> Self {
        let mut cache = HashMap::default();
        cache.insert(
            String::from(NULL_TEXTURE_NAME),
            Rc::new(Texture::try_from_surface(glc, null_texture).unwrap()),
        );
        Self { cache }
    }
    fn get(
        &mut self,
        glc: Arc<GLContext>,
        path: &dyn AsRef<Path>,
    ) -> (Rc<Texture>, Option<AError>) {
        let null_key = Cow::from(NULL_TEXTURE_NAME);
        let path = path.as_ref();
        let key = path.to_string_lossy();
        if let Some(r) = self.cache.get(key.as_ref()) {
            return (Rc::clone(r), None);
        }
        match Surface::read_image_file(path) {
            Ok(s) => {
                let texture = Texture::try_from_surface(glc, &s);
                match texture {
                    Ok(t) => {
                        let txref = Rc::new(t);
                        let myref = Rc::clone(&txref);
                        let path = path.to_string_lossy();
                        self.cache.insert(path.into_owned(), txref);
                        (myref, None)
                    }
                    Err(e) => (
                        Rc::clone(
                            self.cache.get(null_key.as_ref()).as_ref().unwrap(),
                        ),
                        Some(AError::msg(format!(
                            "Could not load texture {}: {:?}",
                            path.display(),
                            e
                        ))),
                    ),
                }
            }
            Err(e) => (
                Rc::clone(self.cache.get(null_key.as_ref()).as_ref().unwrap()),
                Some(AError::msg(format!(
                    "Could not load texture {}: {:?}",
                    path.display(),
                    e
                ))),
            ),
        }
    }
    fn set(
        &mut self,
        glc: Arc<GLContext>,
        name: String,
        data: Surface,
    ) -> Result<Rc<Texture>, AError> {
        match Texture::try_from_surface(glc, &data) {
            Ok(t) => {
                let txref = Rc::new(t);
                let myref = Rc::clone(&txref);
                self.cache.insert(name, txref);
                Ok(myref)
            }
            Err(e) => Err(e),
        }
    }
    fn clear(&mut self) {
        let non_null_textures: Box<[String]> = self
            .cache
            .keys()
            .cloned()
            .filter(|f| f != NULL_TEXTURE_NAME)
            .collect();
        non_null_textures.into_iter().map(String::as_str).for_each(|k| {
            self.cache.remove(k);
        });
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
    model_data: Option<Box<MD3Model>>,
    current_frame: f32,
    anim_playing: bool,
    anim_start_time: Instant,
    anim_start_frame: f32,
    frame_range: Option<RangeInclusive<f32>>,
    error_log: Option<String>,
    models: Vec<BasicModel<u32, UniformsMD3, UniformsMD3Locations>>,
    axes: BasicModel<u8, UniformsRes, UniformsResLocations>,
    tag_axes: BasicModel<u8, UniformsRes, UniformsResLocations>,
    camera: OrbitCamera,
    controls: AppControls,
    texture_cache: TextureCache,
}

impl App {
    fn new(res: &AppResources, glc: &Arc<GLContext>) -> Self {
        let axes_shader = {
            let sp = ShaderProgramBuilder::new()
                .add_shader(ShaderStage::Vertex, &res.res_vertex_shader)
                .add_shader(ShaderStage::Fragment, &res.res_pixel_shader)
                .build(Arc::clone(&glc))
                .unwrap();
            Rc::new(sp)
        };
        App {
            model_data: None,
            current_frame: 0.,
            anim_playing: false,
            anim_start_time: Instant::now(),
            anim_start_frame: 0.,
            frame_range: None,
            error_log: None,
            models: vec![],
            axes: BasicModel {
                vertex: VertexBuffer::new(
                    Arc::clone(glc),
                    Box::new(res::AXES_V),
                ),
                index: IndexBuffer::new(
                    Arc::clone(glc),
                    Vec::from(res::AXES_I),
                ),
                shader: Rc::clone(&axes_shader),
                uniforms: UniformsRes::default(),
            },
            tag_axes: BasicModel {
                vertex: VertexBuffer::new(
                    Arc::clone(glc),
                    Box::new(res::TAGAXES_V),
                ),
                index: IndexBuffer::new(
                    Arc::clone(glc),
                    Vec::from(res::TAGAXES_I),
                ),
                shader: Rc::clone(&axes_shader),
                uniforms: UniformsRes::default(),
            },
            controls: AppControls::default(),
            camera: OrbitCamera::default(),
            texture_cache: TextureCache::new(
                Arc::clone(glc),
                &res.null_surface,
            ),
        }
    }
}

const MOUSE_FACTOR: f32 = 0.0078125; // 1./128
const LOOK_LIMIT: f32 = {
    use std::mem;
    let v = unsafe { mem::transmute::<f32, u32>(FRAC_PI_2) };
    // It's a pain in the butt having to generate this code... But it's all
    // done at compile time, so there are no runtime costs.
    /*
    Shell (zsh) code used to generate this mess:
    (bits=32
    for bit in {0..$((bits-1))}; do
        print -v hxb -f "0x%0$((bits/4))X" $((1 << bit))
        if ((bit > 0)); then
            print -n "else "
        fi
        print "if v & $hxb != 0 { $hxb }"
    done
    print "else { 0 };")
     */
    let lowest_bit = if v & 0x00000001 != 0 {
        0x00000001
    } else if v & 0x00000002 != 0 {
        0x00000002
    } else if v & 0x00000004 != 0 {
        0x00000004
    } else if v & 0x00000008 != 0 {
        0x00000008
    } else if v & 0x00000010 != 0 {
        0x00000010
    } else if v & 0x00000020 != 0 {
        0x00000020
    } else if v & 0x00000040 != 0 {
        0x00000040
    } else if v & 0x00000080 != 0 {
        0x00000080
    } else if v & 0x00000100 != 0 {
        0x00000100
    } else if v & 0x00000200 != 0 {
        0x00000200
    } else if v & 0x00000400 != 0 {
        0x00000400
    } else if v & 0x00000800 != 0 {
        0x00000800
    } else if v & 0x00001000 != 0 {
        0x00001000
    } else if v & 0x00002000 != 0 {
        0x00002000
    } else if v & 0x00004000 != 0 {
        0x00004000
    } else if v & 0x00008000 != 0 {
        0x00008000
    } else if v & 0x00010000 != 0 {
        0x00010000
    } else if v & 0x00020000 != 0 {
        0x00020000
    } else if v & 0x00040000 != 0 {
        0x00040000
    } else if v & 0x00080000 != 0 {
        0x00080000
    } else if v & 0x00100000 != 0 {
        0x00100000
    } else if v & 0x00200000 != 0 {
        0x00200000
    } else if v & 0x00400000 != 0 {
        0x00400000
    } else if v & 0x00800000 != 0 {
        0x00800000
    } else if v & 0x01000000 != 0 {
        0x01000000
    } else if v & 0x02000000 != 0 {
        0x02000000
    } else if v & 0x04000000 != 0 {
        0x04000000
    } else if v & 0x08000000 != 0 {
        0x08000000
    } else if v & 0x10000000 != 0 {
        0x10000000
    } else if v & 0x20000000 != 0 {
        0x20000000
    } else if v & 0x40000000 != 0 {
        0x40000000
    } else if v & 0x80000000 != 0 {
        0x80000000
    } else {
        0
    };
    unsafe { mem::transmute::<u32, f32>(v ^ lowest_bit) }
};

const BLANK_SURFACE_SHADER_NAME: &str = "_____blank_____";

#[derive(Debug)]
struct ModelStuff {
    vb: Vec<VertexMD3>,
    ib: Vec<u32>,
    texture: OsString,
}

#[derive(Debug)]
enum AppEvent {
    LoadMD3 {
        model: MD3Model,
        stuff: Vec<ModelStuff>,
    },
    LoadTextureReplacement {
        name: String,
        image: Surface
    },
    ErrorMessage(String),
}

fn main() -> Result<(), AError> {
    platform::init();
    let app_res = AppResources::try_load(env::var("ASSETS_PATH").ok())
        .context("Failed to load app resources!")?;
    let el = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let elproxy = el.create_proxy();
    let win = WindowBuilder::new()
        .with_title("RustMD3View")
        .build(&el)
        .expect("Could not build the window!");
    let (wc, glc) = platform::show_window(&win, &el, platform::DrawingContextRequest::OpenGL);
    let mut egui_glow = egui_glow::EguiGlow::new(&el, Arc::clone(&glc), None);
    let mut app = App::new(&app_res, &glc);
    let md3_shader = Rc::new({
        let sdr = ShaderProgramBuilder::new()
            .add_shader(ShaderStage::Vertex, &app_res.md3_vertex_shader)
            .add_shader(ShaderStage::Fragment, &app_res.md3_pixel_shader)
            .build(Arc::clone(&glc))?;
        sdr
    });
    app.camera.aspect = {
        let logical_size =
            win.inner_size().to_logical::<f32>(win.scale_factor());
        logical_size.width / logical_size.height
    };
    let mut window_size =
        win.inner_size().to_logical::<f32>(win.scale_factor());
    let md3_model_scale = Vec3::new(1., -1., 1.);
    let md3_model_matrix = Mat4::from_scale(md3_model_scale);
    unsafe {
        glc.clear_color(0., 0., 0., 1.);
        match render::MAX_TEXTURE_UNITS
            .set(Box::new(
                glc.get_parameter_i32(glow::MAX_TEXTURE_IMAGE_UNITS)
                    .try_into()
                    .unwrap_or(u8::MAX),
            ))
            .map_err(|_| {
                format!("Maximum number of texture units already set!")
            }) {
            Ok(_) => println!(
                "Maximum texture units: {}",
                render::MAX_TEXTURE_UNITS.get().copied().unwrap()
            ),
            Err(e) => println!("{}", e),
        }
        match render::MAX_TEXTURE_POT
            .set({
                let max_texture_size =
                    glc.get_parameter_i32(glow::MAX_TEXTURE_SIZE);
                let max_texture_pot: u32 =
                    (max_texture_size as f32).log2().floor().to_int_unchecked();
                let max_texture_pot =
                    max_texture_pot.checked_sub(1).unwrap_or(max_texture_pot);
                Box::new(max_texture_pot)
            })
            .map_err(|_| format!("Maximum texture size already set!"))
        {
            Ok(_) => println!(
                "Maximum texture size: {}",
                2i32.pow(render::MAX_TEXTURE_POT.get().copied().unwrap())
            ),
            Err(e) => println!("{}", e),
        }
    }
    el.run(move |event, _window, control_flow| {
        match event {
            Event::WindowEvent {
                window_id: _,
                event,
            } => {
                use winit::event::{ElementState, MouseButton, WindowEvent::*};
                let egui_response = egui_glow.on_event(&event);
                if egui_response.consumed {
                    return ();
                }
                match event {
                    CloseRequested => {
                        *control_flow = ControlFlow::ExitWithCode(0);
                    }
                    Resized(new_size) => {
                        window_size = new_size.to_logical::<f32>(win.scale_factor());
                        app.camera.aspect = window_size.width / window_size.height;
                    }
                    MouseInput { state, button, .. } => match button {
                        MouseButton::Left => {
                            app.controls.lmb_dragging = match state {
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            };
                        }
                        MouseButton::Right => {
                            app.controls.rmb_dragging = match state {
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            };
                        }
                        _ => (),
                    },
                    CursorLeft { .. } => {
                        app.controls.lmb_dragging = false;
                        app.controls.rmb_dragging = false;
                    }
                    _ => (),
                }
            }
            Event::DeviceEvent { event, .. } => {
                use winit::event::DeviceEvent::*;
                if app.controls.lmb_dragging {
                    match event {
                        MouseMotion { delta: (dx, dy) } => {
                            let dx = dx as f32 * MOUSE_FACTOR;
                            let dy = dy as f32 * MOUSE_FACTOR;
                            app.camera.longtude += dx;
                            app.camera.latitude -= dy;
                            app.camera.latitude =
                                app.camera.latitude.clamp(-LOOK_LIMIT, LOOK_LIMIT);
                        }
                        _ => (),
                    }
                }
                if app.controls.rmb_dragging {
                    match event {
                        MouseMotion { delta: (_dx, dy) } => {
                            let dy = dy as f32 * MOUSE_FACTOR * app.camera.distance.max(1.);
                            app.camera.distance += dy;
                        }
                        _ => (),
                    }
                }
            }
            Event::UserEvent(app_event) => {
                match app_event {
                    AppEvent::LoadMD3 {model, stuff} => {
                        let elp = elproxy.clone();
                        let num_frames = model.frames.len();
                        app.frame_range = if num_frames > 1 {
                            Some(0.0..=(num_frames - 1) as f32)
                        } else {
                            None
                        };
                        app.texture_cache.clear();
                        app.anim_playing = false;
                        app.current_frame = 0.;
                        app.model_data = Some(Box::new(model));
                        app.camera.distance = app.model_data.as_ref().unwrap().max_radius() * 2.;
                        app.models = stuff.into_iter()
                        .zip(app.model_data.as_ref().unwrap().surfaces.iter())
                        .filter_map(|(data, surf)| {
                            let (anim, rows_per_frame) = Texture::try_from_md3(Arc::clone(&glc), surf)
                            .map_err(|e| {
                                elp.send_event(
                                    AppEvent::ErrorMessage(e.to_string()))
                                    .expect("Could not send event");
                            }).ok()?;
                            let anim = Rc::new(anim);
                            Some(BasicModel {
                                vertex: VertexBuffer::new(Arc::clone(&glc), data.vb.into_boxed_slice()),
                                index: IndexBuffer::new(Arc::clone(&glc), data.ib),
                                shader: Rc::clone(&md3_shader),
                                uniforms: UniformsMD3 {
                                    tex: {
                                        let (texture, error) = app.texture_cache.get(Arc::clone(&glc), &data.texture);
                                        if let Some(e) = error {
                                            elp.send_event(AppEvent::ErrorMessage(e.to_string())).expect("Could not send event");
                                        }
                                        texture
                                    },
                                    anim,
                                    gzdoom: Default::default(),
                                    eye: Default::default(),
                                    frame: Default::default(),
                                    mode: Default::default(),
                                    rowsPerFrame: rows_per_frame as i32,
                                }
                            })
                        }).collect()
                    },
                    AppEvent::LoadTextureReplacement { name, image } => {
                        match app.texture_cache.set(Arc::clone(&glc), name.clone(), image) {
                            Ok(t) => {
                                if let Some(model) = &app.model_data {
                                    let replace_texture_on_surface: Vec<bool> = model.surfaces.iter().map(|m| {
                                        m.shaders.iter().any(|sdr| String::from_utf8_stop(&sdr.name) == name)
                                        || (name == BLANK_SURFACE_SHADER_NAME && m.shaders.is_empty())
                                    }).collect();
                                    replace_texture_on_surface.iter().enumerate().for_each(|(index, &y)| {
                                        if y {
                                            app.models[index].uniforms.tex = Rc::clone(&t);
                                        }
                                    })
                                }
                            },
                            Err(e) => {
                                let el = app.error_log.get_or_insert(String::new());
                                if !el.is_empty() {
                                    el.push('\n');
                                }
                                el.push_str(&e.to_string());
                            },
                        }
                    },
                    AppEvent::ErrorMessage(e) => {
                        let el = app.error_log.get_or_insert(String::new());
                        if !el.is_empty() {
                            el.push('\n');
                        }
                        el.push_str(&e);
                    },
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
                    glc.enable(glow::CULL_FACE);
                    glc.cull_face(glow::BACK);
                }
                app.models.iter_mut().for_each(|model| {
                    if let Err(e) = model.render(&glc, |uniforms| {
                        uniforms.eye = app.camera.view_projection() * md3_model_matrix;
                        uniforms.frame = app.current_frame;
                        uniforms.mode = app.controls.view_mode as u32;
                        uniforms.gzdoom = app.controls.gzdoom_normals;
                    }) {
                        eprintln!("{:?}", e);
                    }
                });

                // DRAW TAG AXES
                // ==================================================================

                app.tag_axes.shader.activate().unwrap();
                if let Some(model) = app.model_data.as_ref() {
                    let current_frame = app.current_frame.floor() as usize;
                    let next_frame = app.current_frame.ceil() as usize;
                    let lerp_factor = app.current_frame.fract();
                    let num_tags = model.num_tags;
                    (0..num_tags).for_each(|tag_index| {
                        let tag_a = tag_index + num_tags * current_frame;
                        let tag_b = tag_index + num_tags * next_frame;
                        let tag_a = &model.tags[tag_a];
                        let tag_b = &model.tags[tag_b];
                        let tag_axes = lerp(tag_a.axes, tag_b.axes, lerp_factor);
                        let tag_origin = lerp(tag_a.origin, tag_b.origin, lerp_factor);
                        let tag_distance =
                            (app.camera.position() * md3_model_scale).distance(tag_origin) / 256.;
                        let mvp = app.camera.view_projection()
                            * md3_model_matrix
                            * Affine3A::from_mat3_translation(tag_axes, tag_origin)
                            * Mat4::from_scale(Vec3::splat(tag_distance));

                        if let Err(e) = app.tag_axes.render(&glc, |uniforms| {
                            uniforms.eye = mvp;
                            uniforms.shaded = true;
                        }) {
                            eprintln!("{:?}", e);
                        }
                    });
                }

                // DRAW AXES
                // ==================================================================
                unsafe {
                    glc.depth_func(glow::ALWAYS);
                }
                app.axes.shader.activate().unwrap();
                let mvp = {
                    let eye = Vec3::new(
                        app.camera.longtude.cos() * app.camera.latitude.cos(),
                        app.camera.longtude.sin() * app.camera.latitude.cos(),
                        app.camera.latitude.sin(),
                    ) * -60.;
                    // 160 pixels left from top right corner, 80 pixels down from top right corner
                    let trans = Mat4::from_translation(Vec3::new(
                        1.0 - (320. / window_size.width),
                        1.0 - (160. / window_size.height),
                        0.,
                    ));
                    let scale = Mat4::from_scale(Vec3::new(0.125, 0.125, 0.125));
                    let view = Mat4::look_at_lh(eye, Vec3::ZERO, Vec3::Z);
                    let proj = Mat4::perspective_lh(app.camera.fov, app.camera.aspect, 0.25, 512.);
                    trans * proj * view * scale * md3_model_matrix
                };

                if let Err(e) = app.axes.render(&glc, |uniforms| {
                    uniforms.eye = mvp;
                    uniforms.shaded = false;
                }) {
                    eprintln!("{:?}", e);
                }

                // DRAW EGUI
                // ==================================================================
                egui_glow.run(&win, |ctx| {
                    egui::TopBottomPanel::top("menu_bar").show(&ctx, |ui| {
                        egui::menu::bar(ui, |ui| {
                            ui.menu_button("File", |ui| {
                                if ui.button("Open").clicked() {
                                    let elp = elproxy.clone();
                                    platform::spawn_local(async move {
let picker = AsyncFileDialog::new().add_filter("MD3", &["md3"]);
if let Some(file_handle) = picker.pick_file().await {
    let fpath = file_handle.path();
    // let fdata = file_handle.read().await;
    if let Err(e) = File::open(&fpath)
        .map_err(AError::from)
        .and_then(|mut f| {
            md3::read_md3(&mut f).map_err(AError::from)
                .with_context(|| format!(
                    "Error reading file {}",
                    fpath.to_string_lossy()))
        })
        .and_then(|model| {
            let stuff: Vec<_> = model.surfaces.iter().filter_map(|surf| {
                let vb = VertexBuffer::from_surface(surf);
                let ib = IndexBuffer::from_surface(surf);
                let texture = surf.shaders.get(0).map(|s|
                OsString::from(fpath.parent().unwrap_or(&fpath).join(
                    String::from_utf8_stop(&s.name)
                    .trim_matches(|c| c == char::from_u32(0).unwrap())
                    .trim()))
                ).unwrap_or(OsString::new());
                Some(ModelStuff {
                    vb,
                    ib,
                    texture,
                })
            }).collect();
            elp.send_event(AppEvent::LoadMD3 { model, stuff })
                .expect("Could not send event");
            Ok(())
    }) {
        elp.send_event(AppEvent::ErrorMessage(e.to_string()))
            .expect("Could not send event");
    }
}
});
                                    ui.close_menu();
                                }
                                if ui.button("Quit").clicked() {
                                    ui.close_menu();
                                    *control_flow = ControlFlow::ExitWithCode(0);
                                }
                            });
                            ui.menu_button("View", |ui| {
                                if ui
                                    .radio_value(
                                        &mut app.controls.view_mode,
                                        ViewMode::Textured,
                                        "Textured",
                                    )
                                    .clicked()
                                    || ui
                                        .radio_value(
                                            &mut app.controls.view_mode,
                                            ViewMode::Untextured,
                                            "Untextured",
                                        )
                                        .clicked()
                                    || ui
                                        .radio_value(
                                            &mut app.controls.view_mode,
                                            ViewMode::Normals,
                                            "Normals",
                                        )
                                        .clicked()
                                {
                                    ui.close_menu();
                                }
                                if ui
                                    .checkbox(&mut app.controls.gzdoom_normals, "GZDoom normals")
                                    .clicked()
                                {
                                    ui.close_menu();
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
                                        if app.anim_playing {
                                            app.anim_start_time = Instant::now();
                                            app.anim_start_frame = app.current_frame;
                                        }
                                    }
                                    if app.anim_playing {
                                        app.current_frame = if let Bound::Included(&fc) =
                                            range.end_bound()
                                        {
                                            ((Instant::now() - app.anim_start_time).as_secs_f32()
                                                + app.anim_start_frame)
                                                % fc
                                        } else {
                                            0.
                                        };
                                    }
                                    ui.spacing_mut().slider_width = 400.;
                                    ui.add(egui::Slider::new(
                                        &mut app.current_frame,
                                        range.clone(),
                                    ));
                                });
                            }
                            None => (),
                        }
                    });
                    let error_window = egui::Window::new("Error")
                        .default_height(200.)
                        .vscroll(true);
                    {
                        let el = &mut app.error_log;
                        if el.is_some() {
                            error_window.show(ctx, |ui| {
                                if ui.button("Clear").clicked() {
                                    *el = None;
                                } else {
                                    ui.label(el.as_ref().unwrap());
                                }
                            });
                        }
                    }
                    egui::SidePanel::right("infoz").show(ctx, |ui| {
                        ui.heading("Shaders");
                        if let Some(model) = app.model_data.as_ref() {
                            model.surfaces.iter().enumerate().for_each(|(index, surf)| {
                                egui::CollapsingHeader::new(format!("Surface {}", index)).show(
                                    ui,
                                    |ui| {
                                        surf.shaders.iter()
                                        .enumerate()
                                        .for_each(|(index, sdr)| {
                                            ui.horizontal(|ui| {
                                                ui.label(format!("{index}."));
                                                ui.label(String::from_utf8_stop(&sdr.name));
                                            });
                                        });
                                        if ui.button("Replace texture").clicked() {
                                            // Need surface index and new texture
                                            let elp = elproxy.clone();
                                            let sdr_name = surf.shaders.iter()
                                                .next().map(|sdr| String::from_utf8_stop(&sdr.name))
                                                .unwrap_or(Cow::from(BLANK_SURFACE_SHADER_NAME))
                                                .to_string();
                                            platform::spawn_local(async move {
                                                let file_handle = AsyncFileDialog::new()
                                                    .add_filter("Image", &["png", "jpg", "tga", "pcx", "dds"])
                                                    .pick_file()
                                                    .await;
                                                if let Some(file_handle) = file_handle {
                                                    let file_data = Cursor::new(file_handle.read().await);
                                                    match Surface::read_image_data(file_data) {
                                                        Ok(image) => {
                                                            elp.send_event(AppEvent::LoadTextureReplacement { name: sdr_name, image }).expect("Could not send event");
                                                        },
                                                        Err(e) => {
                                                            elp.send_event(AppEvent::ErrorMessage(e.to_string())).expect("Could not send event");
                                                        },
                                                    }
                                                }
                                            });
                                        }
                                    },
                                );
                            });
                        }
                    });
                    // DRAW TAG NAMES AT TAG POSITIONS
                    // ==================================================================
                    let painter = ctx.layer_painter(LayerId {
                        order: Order::Foreground,
                        id: Id::new("tag_name_overlays"),
                    });
                    if let Some(model) = app.model_data.as_ref() {
                        let current_frame = app.current_frame.floor() as usize;
                        let next_frame = app.current_frame.ceil() as usize;
                        let lerp_factor = app.current_frame.fract();
                        let num_tags = model.num_tags;
                        (0..num_tags).for_each(|tag_index| {
                            let tag_a = tag_index + num_tags * current_frame;
                            let tag_b = tag_index + num_tags * next_frame;
                            let tag_a = &model.tags[tag_a];
                            let tag_b = &model.tags[tag_b];
                            let tag_origin = lerp(tag_a.origin, tag_b.origin, lerp_factor);
                            let tag_name = String::from_utf8_stop(&tag_a.name).to_string();
                            let font =
                                egui::style::default_text_styles()[&TextStyle::Small].clone();
                            let galley = painter.layout_no_wrap(tag_name, font, Color32::WHITE);
                            let pos = {
                                let pos = (app.camera.view_projection() * md3_model_matrix)
                                    .project_point3(tag_origin);
                                let Vec3 { x, y, .. } = pos;
                                let x = x.mul_add(0.5, 0.5) * window_size.width;
                                // In OpenGL NDC, +y is up and -y is down
                                let y = (-y).mul_add(0.5, 0.5) * window_size.height;
                                Pos2 { x, y }
                            };
                            painter.galley(pos, galley);
                        });
                    }
                });
                egui_glow.paint(&win);
                // SWAP BUFFERS
                // ==================================================================
                if let Err(e) = wc.swap_buffers() {
                    eprintln!("{:?}", e);
                }
            }
            _ => (),
        }
    });
}

#[inline]
fn lerp<T>(a: T, b: T, f: f32) -> T
where
    T: Mul<f32, Output = T> + Add<T, Output = T>,
{
    a * (1. - f) + b * f
}
