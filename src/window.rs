use glow::{Context as GLContext};
use glutin::{
	event_loop::EventLoop,
	window::{Window, WindowBuilder},
	ContextBuilder,
	ContextWrapper,
	PossiblyCurrent,
	GlProfile,
	GlRequest,
	Api
};

type WindowContext = ContextWrapper<PossiblyCurrent, Window>;

pub fn create_window<T>(el: &EventLoop<T>, title: Option<&str>) -> (WindowContext, GLContext) {
	let wb = WindowBuilder::new().with_title(title.unwrap_or("A fantastic window!"));

	let wc = ContextBuilder::new()
		.with_gl_profile(GlProfile::Core)
		.with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
		.build_windowed(wb, &el).unwrap();

	let wc = unsafe { wc.make_current().unwrap() };

	println!("Pixel format of the window's GL context: {:?}",
		wc.get_pixel_format());

	let glc = unsafe {
		GLContext::from_loader_function(
			|name| wc.get_proc_address(name))
	};

	(wc, glc)
}
