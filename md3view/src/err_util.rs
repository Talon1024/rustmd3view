use std::error::Error;
use glow::{
	Context as GLContext, HasContext,
	NO_ERROR,
	INVALID_ENUM,
	INVALID_VALUE,
	INVALID_OPERATION,
	INVALID_FRAMEBUFFER_OPERATION,
	OUT_OF_MEMORY
};

macro_rules! err_check {
	($x: ident, $arr: ident, $v: expr) => {
		if $x ^ $v & $v == 0 {
			$arr.push(stringify!($v));
		};
	};
}

#[derive(Debug)]
pub enum GLError {
	InvalidEnum,
	InvalidValue,
	InvalidOperation,
	InvalidFramebufferOperation,
	OutOfMemory,
	Other(u32),
}

impl std::fmt::Display for GLError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			GLError::InvalidEnum => write!(f, "InvalidEnum"),
			GLError::InvalidValue => write!(f, "InvalidValue"),
			GLError::InvalidOperation => write!(f, "InvalidOperation"),
			GLError::InvalidFramebufferOperation => write!(f, "InvalidFramebufferOperation"),
			GLError::OutOfMemory => write!(f, "OutOfMemory"),
			GLError::Other(errs) => {
				let mut errors = Vec::with_capacity(5);
				err_check!(errs, errors, INVALID_ENUM);
				err_check!(errs, errors, INVALID_VALUE);
				err_check!(errs, errors, INVALID_OPERATION);
				err_check!(errs, errors, INVALID_FRAMEBUFFER_OPERATION);
				err_check!(errs, errors, OUT_OF_MEMORY);
				let errors = errors.join(" | ");
				write!(f, "{}", errors)
			},
		}
	}
}

impl Error for GLError {}

impl GLError {
	pub fn get(glc: &GLContext) -> Result<(), GLError> {
		match unsafe { glc.get_error() } {
			NO_ERROR => Ok(()),
			INVALID_ENUM => Err(GLError::InvalidEnum),
			INVALID_VALUE => Err(GLError::InvalidValue),
			INVALID_OPERATION => Err(GLError::InvalidOperation),
			INVALID_FRAMEBUFFER_OPERATION => Err(GLError::InvalidFramebufferOperation),
			OUT_OF_MEMORY => Err(GLError::OutOfMemory),
			errs => Err(GLError::Other(errs))
		}
	}
}
