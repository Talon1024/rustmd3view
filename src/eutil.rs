use anyhow::{Error, Result};
use glow::{Context as GLContext, HasContext};

macro_rules! s {
	($v: literal) => { String::from($v) }
}

macro_rules! err_check {
	($x: ident, $arr: ident, $v: expr) => {
		if $x ^ $v & $v == 0 {
			$arr.push("$v");
		};
	};
}

pub fn gl_get_error(glc: &GLContext) -> Result<()> {
	match unsafe { glc.get_error() } {
		glow::NO_ERROR => Ok(()),
		glow::INVALID_ENUM => Err(s!("INVALID_ENUM")),
		glow::INVALID_VALUE => Err(s!("INVALID_VALUE")),
		glow::INVALID_OPERATION => Err(s!("INVALID_OPERATION")),
		glow::INVALID_FRAMEBUFFER_OPERATION => Err(s!("INVALID_FRAMEBUFFER_OPERATION")),
		glow::OUT_OF_MEMORY => Err(s!("OUT_OF_MEMORY")),
		errs => {
			let mut errors = Vec::new();
			err_check!(errs, errors, glow::INVALID_ENUM);
			err_check!(errs, errors, glow::INVALID_VALUE);
			err_check!(errs, errors, glow::INVALID_OPERATION);
			err_check!(errs, errors, glow::INVALID_FRAMEBUFFER_OPERATION);
			err_check!(errs, errors, glow::OUT_OF_MEMORY);
			let errors = errors.join(", ");
			Err(format!("Errors: {}", errors))
		},
	}.map_err(Error::msg)?;
	Ok(())
}
