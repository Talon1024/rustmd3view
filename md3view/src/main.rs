mod err_util;
mod eye;
mod md3;
mod platform;
mod render;
mod res;
mod str_util;
mod data;

use ahash::RandomState;
use anyhow::{Context as AContext, Error as AError};
use data::ScreenSize;
use egui::{Color32, Id, LayerId, Order, Pos2, TextStyle};
use eye::{Camera, OrbitCamera};
use futures::executor;
use glam::{Affine3A, Mat4, Vec3, Vec2, Vec3Swizzles};
use glow::{Context as GLContext, HasContext};
use instant::Instant;
use md3::MD3Model;
use render::{
    BasicModel, SeparateVertexAttributes, IndexBuffer, ShaderProgramBuilder,
    ShaderStage, Texture, UniformsMD3, UniformsMD3Locations, UniformsRes,
    UniformsResLocations, VertexBuffer, VertexMD3, VertexRes, ThickLines,
    ThickLineInstanceInfo,
};
use res::{AppResources, Surface};
use rfd::AsyncFileDialog;
use std::{
    borrow::Cow,
    collections::HashMap,
    env,
    f32::consts::FRAC_PI_2,
    ffi::{OsString, OsStr},
    fs::File,
    io::{Cursor, Read, Seek},
    ops::{Add, Bound, Mul, RangeBounds, RangeInclusive},
    path::Path,
    rc::Rc,
    sync::Arc,
};
use str_util::StringFromBytes;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
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
        non_null_textures.iter().map(String::as_str).for_each(|k| {
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
    snap_frame_to_integer: bool,
}

struct App {
    md3_data: Vec<MD3Model>,
    current_frame: f32,
    anim_playing: bool,
    anim_start_time: Instant,
    anim_start_frame: f32,
    frame_range: Option<RangeInclusive<f32>>,
    error_log: Option<String>,
    models: Vec<Vec<BasicModel<u32, UniformsMD3, UniformsMD3Locations>>>,
    tag_axes: BasicModel<u8, UniformsRes, UniformsResLocations>,
    camera: OrbitCamera,
    controls: AppControls,
    texture_cache: TextureCache,
    screen_size: ScreenSize,
    lines: ThickLines,
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
            md3_data: vec![],
            current_frame: 0.,
            anim_playing: false,
            anim_start_time: Instant::now(),
            anim_start_frame: 0.,
            frame_range: None,
            error_log: None,
            models: vec![],
            tag_axes: BasicModel {
                vertex: unsafe { VertexRes::setup_vertex_attrs(
                    Arc::clone(&glc),
                    &res::TAGAXES_V,
                ) },
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
            screen_size: ScreenSize::default(),
            lines: ThickLines::new(Arc::clone(glc), res),
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
    texture: Option<Cow<'static, OsStr>>,
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
    LoadSurfaceTextureReplacement {
        surface_index: usize,
        image: Surface
    },
    ErrorMessage(String),
}

fn main() -> Result<(), AError> {
    platform::init();
    let app_res = executor::block_on(AppResources::try_load(env::var("ASSETS_PATH").ok()))
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
    app.screen_size = ScreenSize::from(
        win.inner_size()
            .to_logical::<f32>(
            win.scale_factor()));
    app.camera.aspect = app.screen_size.aspect_ratio();
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

    // For testing
    /* let circle_points: Vec<_> = {
        let circle_start_angle = std::f32::consts::PI / 12.;
        let circle_angle = std::f32::consts::TAU / 12.;
        let circle_radius = 100.;
        (0..12).map(|c| {
            Vec2::from_angle(circle_start_angle + circle_angle * c as f32) * circle_radius
        }).collect()
    }; */
    // For testing

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
                        app.screen_size = ScreenSize::from(
                            new_size
                            .to_logical::<f32>(
                                win.scale_factor()));
                        app.camera.aspect = app.screen_size.aspect_ratio();
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
                    DroppedFile(fpath) => {
                        let elp = elproxy.clone();
                        platform::spawn_local(async move {
                            let path = fpath.clone();
                            let res = File::open(fpath).map_err(AError::from)
                                .and_then(|ref mut file| {
                                    load_model(file, Some(&path))
                                });
                            match res {
                                Ok(loaded) => {
                                    elp.send_event(loaded)
                                        .expect("Could not send event");
                                },
                                Err(e) => {
                                    elp.send_event(
                                        AppEvent::ErrorMessage(e.to_string()))
                                        .expect("Could not send event");
                                },
                            }
                            ()
                        });
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
                        app.md3_data = vec![model];
                        app.camera.distance = app.md3_data.get(0).unwrap().max_radius() * 2.;
                        app.models = vec![stuff.into_iter()
                        .zip(app.md3_data.get(0).unwrap().surfaces.iter())
                        .filter_map(|(data, surf)| {
                            let (anim, rows_per_frame) = Texture::try_from_md3(Arc::clone(&glc), surf)
                            .map_err(|e| {
                                elp.send_event(
                                    AppEvent::ErrorMessage(e.to_string()))
                                    .expect("Could not send event");
                            }).ok()?;
                            let anim = Rc::new(anim);
                            Some(BasicModel {
                                // Interleaved vertex attributes
                                // vertex: VertexBuffer::new(Arc::clone(&glc), data.vb.into_boxed_slice()),
                                // Separate vertex attributes
                                vertex: unsafe { <VertexMD3 as SeparateVertexAttributes>::setup_vertex_attrs(Arc::clone(&glc), &data.vb) },
                                index: IndexBuffer::new(Arc::clone(&glc), data.ib),
                                shader: Rc::clone(&md3_shader),
                                uniforms: UniformsMD3 {
                                    tex: {
                                        let (texture, error) = app.texture_cache
                                        .get(
                                            Arc::clone(&glc),
                                            &data.texture.unwrap_or(Cow::from(AsRef::<OsStr>::as_ref(NULL_TEXTURE_NAME)))
                                        );
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
                        }).collect()]
                    },
                    AppEvent::LoadTextureReplacement { name, image } => {
                        match app.texture_cache.set(Arc::clone(&glc), name.clone(), image) {
                            Ok(t) => {
                                if let Some(model) = app.md3_data.get(0) {
                                    let replace_texture_on_surface: Vec<bool> = model.surfaces.iter().map(|m| {
                                        m.shaders.iter().any(|sdr| String::from_utf8_stop(&sdr.name) == name)
                                        || (name == BLANK_SURFACE_SHADER_NAME && m.shaders.is_empty())
                                    }).collect();
                                    replace_texture_on_surface.iter().enumerate().for_each(|(index, &y)| {
                                        if y {
                                            app.models[0][index].uniforms.tex = Rc::clone(&t);
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
                    AppEvent::LoadSurfaceTextureReplacement { surface_index, image } => {
                        if let Ok(tex) = Texture::try_from_surface(Arc::clone(&glc), &image) {
                            if let Some(model) = app.models[0].get_mut(surface_index) {
                                model.uniforms.tex = Rc::new(tex);
                            }
                        }
                    }
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
                app.models.iter_mut().for_each(|submodels| {
                    submodels.iter_mut().for_each(|model| {
                        if let Err(e) = model.render(&glc, |uniforms| {
                            uniforms.eye = app.camera.view_projection() * md3_model_matrix;
                            uniforms.frame = app.current_frame;
                            uniforms.mode = app.controls.view_mode as u32;
                            uniforms.gzdoom = app.controls.gzdoom_normals;
                        }) {
                            eprintln!("{:?}", e);
                        }
                    });
                });

                // DRAW TAG AXES
                // ==================================================================

                app.tag_axes.shader.activate().unwrap();
                if let Some(model) = app.md3_data.get(0) {
                    let current_frame = app.current_frame.floor() as u32;
                    let next_frame = app.current_frame.ceil() as u32;
                    let lerp_factor = app.current_frame.fract();
                    let num_tags = model.num_tags;
                    (0..num_tags).for_each(|tag_index| {
                        let tag_a = tag_index + num_tags * current_frame as usize;
                        let tag_b = tag_index + num_tags * next_frame as usize;
                        let tag_a = &model.tags[tag_a as usize];
                        let tag_b = &model.tags[tag_b as usize];
                        let tag_axes = lerp(tag_a.axes, tag_b.axes, lerp_factor);
                        let tag_origin = lerp(tag_a.origin, tag_b.origin, lerp_factor);
                        let tag_distance =
                            (app.camera.position(None) * md3_model_scale).distance(tag_origin) / 256.;
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
                // app.axes.shader.activate().unwrap();
                let mvp = {
                    let eye = app.camera.position(Some(60.));
                    let view = Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Z);
                    let proj = Mat4::orthographic_rh_gl(-50., 50., -50., 50., 0.25, 512.);
                    let scale = Mat4::from_scale(Vec3::splat(60.0));
                    scale * proj * view
                };

                let axes_origin_xy = Vec2::new(
                    app.screen_size.width - 160.,
                    100.
                );
                let axes_points = [
                    Vec3::X * -40.,
                    Vec3::Y * -40.,
                    Vec3::Z * -40.,
                ].map(|pt| mvp.transform_point3(pt).xy() + axes_origin_xy);

                let axes_colours = [
                    Vec3::new(1.0, 0., 0.),
                    Vec3::new(0., 0.75, 0.375),
                    Vec3::new(0.1875, 0.4375, 1.0)
                ];

                let axes_letters = [
                    [
                        Vec2 { x: 4., y: 10. },
                        Vec2 { x: -4., y: -10. },
                        Vec2 { x: -4., y: 10. },
                        Vec2 { x: 4., y: -10. },
                        Vec2 { x: 0., y: 0. }, // Please type checker
                        Vec2 { x: 0., y: 0. },
                    ], // X
                    [
                        Vec2 { x: 4., y: -10. },
                        Vec2 { x: 0., y: 1. },
                        Vec2 { x: 0., y: 1. },
                        Vec2 { x: -4., y: -10. },
                        Vec2 { x: 0., y: 1. },
                        Vec2 { x: 0., y: 10. },
                    ], // Y
                    [
                        Vec2 { x: 4., y: 10. },
                        Vec2 { x: -4., y: 10. },
                        Vec2 { x: -4., y: 10. },
                        Vec2 { x: 4., y: -10. },
                        Vec2 { x: 4., y: -10. },
                        Vec2 { x: -4., y: -10. },
                    ], // Z
                ];
                axes_points.iter().zip(axes_colours.iter())
                .for_each(|(point, colour)| {
                    ThickLineInstanceInfo {
                        a: axes_origin_xy,
                        b: *point,
                        colour: Some(colour.extend(1.0)),
                        res: Some(app.screen_size),
                    }.add_to(&mut app.lines.instances);
                });
                axes_letters.iter().zip(axes_colours.iter()).zip(axes_points.iter())
                .for_each(|((letter, colour), &point)| {
                    letter.chunks_exact(2).filter_map(|line| {
                        if let &[a, b] = line {
                            Some(ThickLineInstanceInfo {
                                a: a + point, b: b + point,
                                colour: Some(colour.extend(1.0)),
                                res: Some(app.screen_size)
                            })
                        } else {
                            None
                        }
                    }).for_each(|line| line.add_to(&mut app.lines.instances));
                });
                // For testing
                /* circle_points.windows(2).chain(
                    std::iter::once([
                        circle_points.last().copied().unwrap(),
                        circle_points.first().copied().unwrap()
                    ].as_slice()))
                .enumerate().for_each(|(index, window)| {
                    if let [a, b] = window {
                        let a = *a + (Vec2::from(app.screen_size) / 2.);
                        let b = *b + (Vec2::from(app.screen_size) / 2.);

                        let hue = index as f32 / circle_points.len() as f32 * std::f32::consts::TAU;
                        let subtract: [f32; 3] = [0., 0.333333333, 0.666666666];
                        let rgb = Vec3::from_array(subtract.map(|sub| {
                            (hue - sub * std::f32::consts::PI * 2.).cos() + 0.5
                        }));
                        
                        app.lines.instances.push(ThickLineInstanceInfo {
                            a, b, colour: Some(rgb.extend(1.0)), res: Some(app.screen_size)
                        }.into());
                    }
                });
                */
                /* app.lines.instances.push(ThickLineInstanceInfo {
                    a: Vec2::new(20., 70.), b: Vec2::new(70., 30.), colour: None, res: Some(app.screen_size)
                }.into());
                app.lines.instances.push(ThickLineInstanceInfo {
                    a: Vec2::new(70., 30.), b: Vec2::new(50., 30.), colour: None, res: Some(app.screen_size)
                }.into());
                app.lines.instances.push(ThickLineInstanceInfo {
                    a: Vec2::new(70., 30.), b: Vec2::new(70., 50.), colour: None, res: Some(app.screen_size)
                }.into()); */
                // For testing

                if let Err(e) = app.lines.render(|uniforms| {
                    uniforms.window_resolution = app.screen_size;
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
                                    platform::spawn_local(pick_and_load_model(elp));
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
                                if ui.checkbox(&mut app.controls.snap_frame_to_integer, "Snap animation frame to integers").clicked() {
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
            app.anim_start_frame = if !app.controls.snap_frame_to_integer {
                app.current_frame
            } else {
                app.current_frame.floor()
            };
        }
    }
    if app.anim_playing {
        app.current_frame = {
            if let Bound::Included(&frame_count) = range.end_bound() {
                let elapsed = Instant::now() - app.anim_start_time;
                (if !app.controls.snap_frame_to_integer {
                    elapsed.as_secs_f32()
                } else {
                    elapsed.as_secs() as f32
                } + app.anim_start_frame) % frame_count
            } else {
                0.
            }
        };
    }
    ui.spacing_mut().slider_width = 400.;
    ui.add(egui::Slider::new(&mut app.current_frame, range.clone(),)
        .step_by(if !app.controls.snap_frame_to_integer {0.0} else {1.0}));
});
                            }
                            None => {
                                ui.label("No animation");
                            },
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
                        if let Some(model) = app.md3_data.get(0) {
                            ui.heading("Shaders");
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
                                            platform::spawn_local(pick_and_load_surface_texture(elp, index));
                                        }
                                        if ui.button("Replace texture (global)").clicked() {
                                            // Need surface index and new texture
                                            let elp = elproxy.clone();
                                            let sdr_name = surf.shaders.iter()
                                                .next().map(|sdr| String::from_utf8_stop(&sdr.name))
                                                .unwrap_or(Cow::from(BLANK_SURFACE_SHADER_NAME))
                                                .to_string();
                                            platform::spawn_local(pick_and_load_texture(elp, sdr_name));
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
                    if let Some(model) = app.md3_data.get(0) {
                        let current_frame = app.current_frame.floor() as u32;
                        let next_frame = app.current_frame.ceil() as u32;
                        let lerp_factor = app.current_frame.fract();
                        let num_tags = model.num_tags;
                        (0..num_tags).for_each(|tag_index| {
                            let tag_a = tag_index + num_tags * current_frame as usize;
                            let tag_b = tag_index + num_tags * next_frame as usize;
                            let tag_a = &model.tags[tag_a as usize];
                            let tag_b = &model.tags[tag_b as usize];
                            let tag_origin = lerp(tag_a.origin, tag_b.origin, lerp_factor);
                            let tag_name = String::from_utf8_stop(&tag_a.name).to_string();
                            let font =
                                egui::style::default_text_styles()[&TextStyle::Small].clone();
                            let galley = painter.layout_no_wrap(tag_name, font, Color32::WHITE);
                            let pos = {
                                let pos = (app.camera.view_projection() * md3_model_matrix)
                                    .project_point3(tag_origin);
                                let Vec3 { x, y, .. } = pos;
                                let x = x.mul_add(0.5, 0.5) * app.screen_size.width;
                                // In OpenGL NDC, +y is up and -y is down
                                let y = (-y).mul_add(0.5, 0.5) * app.screen_size.height;
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

async fn pick_and_load_surface_texture(elp: EventLoopProxy<AppEvent>, surface_index: usize) -> () {
    let file_handle = AsyncFileDialog::new()
        .add_filter("Image", &["png", "jpg", "tga", "pcx", "dds"])
        .pick_file()
        .await;
    if let Some(file_handle) = file_handle {
        let file_data = Cursor::new(file_handle.read().await);
        match Surface::read_image_data(file_data) {
            Ok(image) => {
                elp.send_event(AppEvent::LoadSurfaceTextureReplacement { surface_index, image }).expect("Could not send event");
            },
            Err(e) => {
                elp.send_event(AppEvent::ErrorMessage(e.to_string())).expect("Could not send event");
            },
        }
    }
}

async fn pick_and_load_texture(elp: EventLoopProxy<AppEvent>, sdr_name: String) -> () {
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
}

async fn pick_and_load_model(elp: EventLoopProxy<AppEvent>) -> () {
    let picker = AsyncFileDialog::new().add_filter("MD3", &["md3"]);
    if let Some(file_handle) = picker.pick_file().await {
        let fpath = if cfg!(not(target_family = "wasm")) {
            Some(file_handle.path())
        } else {
            None
        };
        let fname = file_handle.file_name();
        let fdata = file_handle.read().await;
        let mut cursor = Cursor::new(&fdata);
        match load_model(&mut cursor, fpath)
            .with_context(|| format!("Error reading file {fname}")) {
                Ok(loaded) => {
                    elp.send_event(loaded).expect("Could not send event");
                }
                Err(e) => {
                    elp.send_event(AppEvent::ErrorMessage(e.to_string()))
                        .expect("Could not send event");
                }
        }
    }
}

fn load_model(file: &mut (impl Read + Seek), fpath: Option<&Path>) -> Result<AppEvent, AError> {
    //MD3Model::read(file)
    md3::read_md3(file).map_err(AError::from)
        .and_then(|model| {
            let stuff: Vec<_> = model.surfaces.iter().filter_map(|surf| {
                let vb = VertexBuffer::from_surface(surf);
                let ib = IndexBuffer::from_surface(surf);
                let texture = surf.shaders.get(0)
                .zip(fpath).and_then(|(shader, fpath)| {
                    let shader = String::from_utf8_stop(shader.name.as_slice());
                    let fullpath = fpath.parent().unwrap().join(shader.as_ref());
                    let fullpath = OsString::from(fullpath);
                    Some(Cow::from(fullpath))
                });
                Some(ModelStuff {
                    vb,
                    ib,
                    texture,
                })
            }).collect();
            Ok(AppEvent::LoadMD3 { model, stuff })
    })
}