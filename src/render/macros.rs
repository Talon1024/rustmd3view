/* 
// Input
pub struct UniformsMD3 {
	pub gzdoom: bool,
	pub anim: Rc<Texture>,
	pub eye: Mat4,
	pub frame: f32,
	pub mode: u32,
	pub tex: Rc<Texture>,
}

// Output
pub struct UniformsMD3 {
	pub gzdoom: bool,
	gzdoom_l_: Option<NativeUniformLocation>,
	pub anim: Rc<Texture>,
	anim_l_: Option<NativeUniformLocation>,
	pub eye: Mat4,
	eye_l_: Option<NativeUniformLocation>,
	pub frame: f32,
	frame_l_: Option<NativeUniformLocation>,
	pub mode: u32,
	mode_l_: Option<NativeUniformLocation>,
	pub tex: Rc<Texture>,
	tex_l_: Option<NativeUniformLocation>,
}
*/

macro_rules! _uniform {
	($name:ident: $utyp:ty) => {
		$name: $utyp,
		$name_l_: Option<NativeUniformLocation>,
	};
}

macro_rules! uniforms {
	(struct $sname:ident {
		$field:tt,+
	}) => {
		struct $sname {
			_uniform!($field)
		}
	};
}

#[cfg(test)]
mod tests {
	#[test]
	fn uniforms_simple() {
		/* uniforms! {
			struct MyUniforms {
				integer: i32,
				unsigned: u32,
				floating: f32,
			}
		} */
		uniforms!(
		struct MyUniforms {
			integer: i32
			// uniform!(unsigned: u32),
			// uniform!(floating: f32),
		});
		let u = MyUniforms {
			integer: 0,
			integer_l_: None,
		};
		assert_eq!(u.integer_l_, None);
	}
}
