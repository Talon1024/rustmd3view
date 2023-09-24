use anyhow::Error;
use super::DrawingContextRequest;
use futures::executor::ThreadPool;
use glow::Context;
use glutin::{
    config::{Api, ConfigTemplateBuilder},
    context::{
        ContextApi, ContextAttributesBuilder, GlProfile,
        NotCurrentGlContextSurfaceAccessor, Robustness, Version,
    },
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::{ApiPrefence, DisplayBuilder};
use lazy_static::lazy_static;
use raw_window_handle::HasRawWindowHandle;
use std::{
    ffi::CStr,
    fs::File,
    future::Future,
    io::Read,
    num::NonZeroU32,
    sync::Arc,
};
use winit::{
    dpi::PhysicalSize, event_loop::EventLoopWindowTarget, window::Window,
};

pub(crate) struct WindowContext {
    wc: <Surface<WindowSurface> as GlSurface<WindowSurface>>::Context,
    surf: Surface<WindowSurface>,
}

impl WindowContext {
    pub(crate) fn swap_buffers(&self) -> Result<(), glutin::error::Error> {
        self.surf.swap_buffers(&self.wc)
    }
}

lazy_static! {
    static ref FUTURE_THREAD_POOL: ThreadPool =
        ThreadPool::new().expect("Failed to create thread pool!");
}

pub(crate) fn spawn_local(f: impl Future<Output = ()> + Send + 'static) {
    FUTURE_THREAD_POOL.spawn_ok(f)
}

pub(crate) fn init() {}

pub(crate) fn show_window<CE>(
    win: &Window,
    el: &EventLoopWindowTarget<CE>,
    _ctxr: DrawingContextRequest,
) -> (WindowContext, Arc<Context>) {
    let ctb = ConfigTemplateBuilder::new()
        .with_api(Api::all())
        .prefer_hardware_accelerated(Some(true));
    let (_, cfg) = DisplayBuilder::new()
        .with_preference(ApiPrefence::PreferEgl)
        .build(el, ctb, |mut c| {
            /* #[cfg(debug_assertions)]
            {
                let first = c.next().expect("Could not find an appropriate configuration");
                dbg!(&first);
                c.for_each(|conf| {dbg!(&conf);});
                first
            }
            #[cfg(not(debug_assertions))]
            {
                c.next().expect("Could not find an appropriate configuration")
            } */
            c.next().expect("Could not find an appropriate configuration")
        })
        .expect("Could not build the display");

    let ca = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version {
            major: 3,
            minor: 3,
        })))
        .with_profile(GlProfile::Core)
        .with_robustness(if cfg!(debug_assertions) {
            Robustness::RobustNoResetNotification
        } else {
            Robustness::NoError
        })
        .build(None);

    let PhysicalSize { width: window_width, height: window_height } =
        win.inner_size();

    let sa =
        SurfaceAttributesBuilder::<WindowSurface>::new().with_srgb(None).build(
            win.raw_window_handle(),
            unsafe { NonZeroU32::new_unchecked(window_width) },
            unsafe { NonZeroU32::new_unchecked(window_height) },
        );

    let dsp = cfg.display();
    let wc = unsafe { dsp.create_context(&cfg, &ca) }
        .expect("Could not create context");
    let surf = unsafe { dsp.create_window_surface(&cfg, &sa) }
        .expect("Could not create surface on window");
    let wc = wc.make_current(&surf).expect("Could not make context current");
    let glc = unsafe {
        Context::from_loader_function(|name| {
            let name = CStr::from_ptr(name.as_ptr() as *const i8);
            dsp.get_proc_address(name)
        })
    };
    (WindowContext { wc, surf }, Arc::new(glc))
}
/* 
pub(crate) async fn save_file(contents: Vec<u8>, fname: &str) {
    let fhandle =
        AsyncFileDialog::new().set_file_name(fname).save_file().await;
    if let Some(fhandle) = fhandle {
        let fpath = fhandle.path();
        let file = File::create(fpath);
        match file {
            Ok(mut file) => {
                if let Err(e) = file.write_all(&contents) {
                    eprintln!("{e:?}");
                }
            }
            Err(e) => {
                eprintln!("{e:?}");
            }
        }
    }
}
 */
// An asset is a file that should be part of the application distribution
pub(crate) async fn load_asset(relative_path: impl AsRef<str>) -> Result<Vec<u8>, Error> {
    let mut file = File::open(relative_path.as_ref())?;
    let mut data = Vec::new();
    let size = file.read_to_end(&mut data)?;
    assert_eq!(size, data.len());
    Ok(data)
}
