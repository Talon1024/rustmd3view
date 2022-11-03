use anyhow::{Error, Result};
use glow::{Context as GLContext, HasContext, NO_ERROR, INVALID_ENUM, INVALID_VALUE, INVALID_OPERATION, INVALID_FRAMEBUFFER_OPERATION, OUT_OF_MEMORY};

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
		NO_ERROR => Ok(()),
		INVALID_ENUM => Err(s!("INVALID_ENUM")),
		INVALID_VALUE => Err(s!("INVALID_VALUE")),
		INVALID_OPERATION => Err(s!("INVALID_OPERATION")),
		INVALID_FRAMEBUFFER_OPERATION => Err(s!("INVALID_FRAMEBUFFER_OPERATION")),
		OUT_OF_MEMORY => Err(s!("OUT_OF_MEMORY")),
		errs => {
			let mut errors = Vec::new();
			err_check!(errs, errors, INVALID_ENUM);
			err_check!(errs, errors, INVALID_VALUE);
			err_check!(errs, errors, INVALID_OPERATION);
			err_check!(errs, errors, INVALID_FRAMEBUFFER_OPERATION);
			err_check!(errs, errors, OUT_OF_MEMORY);
			let errors = errors.join(" | ");
			Err(format!("{}", errors))
		},
	}.map_err(Error::msg)?;
	Ok(())
}
