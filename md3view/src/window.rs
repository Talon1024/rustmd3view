use glow::Context as GLContext;
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
use raw_window_handle::HasRawWindowHandle;
use std::{ffi::CStr, num::NonZeroU32, sync::Arc};
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

pub(crate) struct WindowContext {
    wc: <Surface<WindowSurface> as GlSurface<WindowSurface>>::Context,
    surf: Surface<WindowSurface>,
}

impl WindowContext {
    pub fn swap_buffers(&self) -> Result<(), glutin::error::Error> {
        self.surf.swap_buffers(&self.wc)
    }
}

pub(crate) struct AppWindow {
    pub win: Window,
    pub glc: Arc<GLContext>,
    pub wc: WindowContext,
}

pub(crate) fn create_window<CE>(
    el: &EventLoop<CE>,
    title: Option<&str>,
) -> AppWindow {
    let ctb = ConfigTemplateBuilder::new()
        .with_api(Api::all())
        .prefer_hardware_accelerated(Some(true));
    let wb = WindowBuilder::new().with_title(title.unwrap_or("rustmd3view"));
    let (win, cfg) = DisplayBuilder::new()
        .with_window_builder(Some(wb))
        .with_preference(ApiPrefence::PreferEgl)
        .build(el, ctb, |mut c| {
            c.next()
                .expect("Could not find an appropriate configuration")
        })
        .expect("Could not build the display");
    let win = win.expect("No window was created!");

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

    let sa = SurfaceAttributesBuilder::<WindowSurface>::new()
        .with_srgb(None)
        .build(
            win.raw_window_handle(),
            unsafe { NonZeroU32::new_unchecked(800) },
            unsafe { NonZeroU32::new_unchecked(600) },
        );

    let dsp = cfg.display();
    let wc = unsafe { dsp.create_context(&cfg, &ca) }
        .expect("Could not create context");
    let surf = unsafe { dsp.create_window_surface(&cfg, &sa) }
        .expect("Could not create surface on window");
    let wc = wc
        .make_current(&surf)
        .expect("Could not make context current");
    let glc = Arc::new(unsafe {
        GLContext::from_loader_function(|name| {
            let name = CStr::from_ptr(name.as_ptr() as *const i8);
            dsp.get_proc_address(name)
        })
    });
    let wc = WindowContext { wc, surf };

    AppWindow { win, glc, wc }
}
