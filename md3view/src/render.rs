use anyhow::Error as AError;
use glam::{Vec2, Vec3, Mat4};
use crate::md3::MD3Surface;
use crate::res::{Surface, SurfaceType};
use glow::{Context, HasContext, NativeUniformLocation};
use std::{
	mem,
	ops::{Deref, DerefMut},
	rc::Rc,
	sync::Arc,
	marker::PhantomData,
};
use bytemuck::{Pod, Zeroable};
use crate::err_util::GLError;
use once_cell::race::OnceBox;

// #[macro_use]
// mod macros;
pub trait InterleavedVertexAttribute {
	unsafe fn setup_vertex_attrs(glc: &Context);
	fn stride() -> i32 where Self : Sized {
		mem::size_of::<Self>() as i32
	}
}

pub trait ShaderUniformLocations : Default {
	fn setup(&mut self, glc: &Context, program: <Context as HasContext>::Program);
}

pub trait ShaderUniforms<L> where L: ShaderUniformLocations {
	fn set(&self, glc: &Context, locations: &L) -> ();
}
// Brainstorming
/* 
// Input
model_data!(MD3 {
	attr index: u32,
	attr uv: Vec2,
	mut uniform gzdoom: bool,
	uniform anim: Rc<Texture>,
	mut uniform eye: Mat4,
	mut uniform frame: f32,
	mut uniform mode: u32,
	uniform tex: Rc<Texture>,
})
 */
/* 
// Output
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod, Default)]
pub struct MD3Vertex {
	index: u32,
	uv: Vec2,
}

impl InterleavedVertexAttribute for MD3Vertex {
	unsafe fn setup_vertex_attrs(glc: &Context) {
		let mut attrib_index = 0;
		let mut offset = 0;
		let stride = Self::stride();

		glc.vertex_attrib_pointer_i32(attrib_index, 1, glow::UNSIGNED_INT,
			stride, offset);
		glc.enable_vertex_attrib_array(attrib_index);
		offset += mem::size_of::<u32>() as i32;
		attrib_index += 1;

		glc.vertex_attrib_pointer_f32(attrib_index, 2, glow::FLOAT, false,
			stride, offset);
		glc.enable_vertex_attrib_array(attrib_index);
		// offset += mem::size_of::<Vec2>() as i32;
		// attrib_index += 1;
	}
}

#[derive(Debug, Clone)]
pub struct MD3Uniforms {
	pub gzdoom: bool,
	pub anim: Rc<Texture>,
	pub eye: Mat4,
	pub frame: f32,
	pub mode: u32,
	pub tex: Rc<Texture>,
}

#[derive(Debug, Clone, Default)]
pub struct MD3UniformLocations {
	gzdoom: Option<NativeUniformLocation>,
	anim: Option<NativeUniformLocation>,
	eye: Option<NativeUniformLocation>,
	frame: Option<NativeUniformLocation>,
	mode: Option<NativeUniformLocation>,
	tex: Option<NativeUniformLocation>,
}

impl ShaderUniformLocations for MD3UniformLocations {
	fn setup(&mut self, glc: &Context, program: <Context as HasContext>::Program) {
		unsafe {
			self.gzdoom = glc.get_uniform_location(program, "gzdoom");
			self.anim = glc.get_uniform_location(program, "anim");
			self.eye = glc.get_uniform_location(program, "eye");
			self.frame = glc.get_uniform_location(program, "frame");
			self.mode = glc.get_uniform_location(program, "mode");
			self.tex = glc.get_uniform_location(program, "tex");
		}
	}
}

impl ShaderUniforms<MD3UniformLocations> for MD3Uniforms {
	fn set(&self, glc: &Context, locations: &MD3UniformLocations) -> () {
		let mut texture = TextureUnit::default();
		unsafe {
			glc.uniform_1_u32(locations.gzdoom.as_ref(), self.gzdoom as u32);

			glc.active_texture(texture.slot());
			glc.bind_texture(glow::TEXTURE_2D, Some(self.anim.tex()));
			glc.uniform_1_i32(locations.anim.as_ref(), texture.uniform());

			glc.uniform_matrix_4_f32_slice(locations.eye.as_ref(), false, self.eye.as_ref());

			glc.uniform_1_f32(locations.frame.as_ref(), self.frame);

			glc.uniform_1_u32(locations.mode.as_ref(), self.mode);

			texture.next();
			glc.active_texture(texture.slot());
			glc.bind_texture(glow::TEXTURE_2D, Some(self.tex.tex()));
			glc.uniform_1_i32(locations.tex.as_ref(), texture.uniform());
		}
	}
}
 */

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod, Default)]
pub struct VertexMD3 {
	index: u32,
	uv: Vec2,
}

impl InterleavedVertexAttribute for VertexMD3 {
	unsafe fn setup_vertex_attrs(glc: &Context) {
		let mut attrib_index = 0;
		let mut offset = 0;
		let stride = Self::stride();

		glc.vertex_attrib_pointer_i32(attrib_index, 1, glow::UNSIGNED_INT,
			stride, offset);
		glc.enable_vertex_attrib_array(attrib_index);
		offset += mem::size_of::<u32>() as i32;
		attrib_index += 1;

		glc.vertex_attrib_pointer_f32(attrib_index, 2, glow::FLOAT, false,
			stride, offset);
		glc.enable_vertex_attrib_array(attrib_index);
		// offset += mem::size_of::<Vec2>() as i32;
		// attrib_index += 1;
	}
}

// TODO: Macro-ize!
#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub struct UniformsMD3 {
	pub gzdoom: bool,
	pub anim: Rc<Texture>,
	pub eye: Mat4,
	pub frame: f32,
	pub mode: u32,
	pub tex: Rc<Texture>,
	pub rowsPerFrame: i32,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Default)]
pub struct UniformsMD3Locations {
	gzdoom: Option<NativeUniformLocation>,
	anim: Option<NativeUniformLocation>,
	eye: Option<NativeUniformLocation>,
	frame: Option<NativeUniformLocation>,
	mode: Option<NativeUniformLocation>,
	tex: Option<NativeUniformLocation>,
	rowsPerFrame: Option<NativeUniformLocation>,
}

impl ShaderUniformLocations for UniformsMD3Locations {
	fn setup(&mut self, glc: &Context, program: <Context as HasContext>::Program) {
		unsafe {
			self.gzdoom = glc.get_uniform_location(program, "gzdoom");
			self.anim = glc.get_uniform_location(program, "anim");
			self.eye = glc.get_uniform_location(program, "eye");
			self.frame = glc.get_uniform_location(program, "frame");
			self.mode = glc.get_uniform_location(program, "mode");
			self.tex = glc.get_uniform_location(program, "tex");
			self.rowsPerFrame = glc.get_uniform_location(program, "rowsPerFrame");
		}
	}
}

impl ShaderUniforms<UniformsMD3Locations> for UniformsMD3 {
	fn set(&self, glc: &Context, locations: &UniformsMD3Locations) -> () {
		let mut texture = TextureUnit::default();
		unsafe {
			glc.uniform_1_u32(locations.gzdoom.as_ref(), self.gzdoom as u32);

			glc.active_texture(texture.slot());
			glc.bind_texture(glow::TEXTURE_2D, Some(self.anim.tex()));
			glc.uniform_1_i32(locations.anim.as_ref(), texture.uniform());

			glc.uniform_matrix_4_f32_slice(locations.eye.as_ref(), false, self.eye.as_ref());

			glc.uniform_1_f32(locations.frame.as_ref(), self.frame);

			glc.uniform_1_u32(locations.mode.as_ref(), self.mode);

			texture.next();
			glc.active_texture(texture.slot());
			glc.bind_texture(glow::TEXTURE_2D, Some(self.tex.tex()));
			glc.uniform_1_i32(locations.tex.as_ref(), texture.uniform());

			glc.uniform_1_i32(locations.rowsPerFrame.as_ref(), self.rowsPerFrame);
		}
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod, Default)]
pub struct VertexRes {
	pub position: Vec3,
	pub colour: Vec3,
	pub normal: Vec3,
}

impl InterleavedVertexAttribute for VertexRes {
	unsafe fn setup_vertex_attrs(glc: &Context) {
		let mut attrib_index = 0;
		let mut offset = 0;
		let stride = Self::stride();

		glc.vertex_attrib_pointer_f32(attrib_index, 3, glow::FLOAT, false, stride, offset);
		glc.enable_vertex_attrib_array(attrib_index);
		offset += mem::size_of::<Vec3>() as i32;
		attrib_index += 1;

		glc.vertex_attrib_pointer_f32(attrib_index, 3, glow::FLOAT, false, stride, offset);
		glc.enable_vertex_attrib_array(attrib_index);
		offset += mem::size_of::<Vec3>() as i32;
		attrib_index += 1;

		glc.vertex_attrib_pointer_f32(attrib_index, 3, glow::FLOAT, false, stride, offset);
		glc.enable_vertex_attrib_array(attrib_index);
		// offset += mem::size_of::<Vec3>() as i32;
		// attrib_index += 1;
	}
}

// TODO: Macro-ize!
#[derive(Debug, Clone, Default)]
pub struct UniformsRes {
	pub eye: Mat4,
	pub shaded: bool,
}

#[derive(Debug, Clone, Default)]
pub struct UniformsResLocations {
	eye: Option<NativeUniformLocation>,
	shaded: Option<NativeUniformLocation>,
}

impl ShaderUniformLocations for UniformsResLocations {
	fn setup(&mut self, glc: &Context, program: <Context as HasContext>::Program) {
		unsafe {
			self.eye = glc.get_uniform_location(program, "eye");
			self.shaded = glc.get_uniform_location(program, "shaded");
		}
	}
}

impl ShaderUniforms<UniformsResLocations> for UniformsRes {
	fn set(&self, glc: &Context, locations: &UniformsResLocations) -> () {
		let mut _texture = TextureUnit::default();
		unsafe {
			glc.uniform_matrix_4_f32_slice(locations.eye.as_ref(), false, self.eye.as_ref());
			glc.uniform_1_u32(locations.shaded.as_ref(), self.shaded as u32);
		}
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod, Default)]
pub struct VertexSprite {
	pub position: Vec2,
	pub size: Vec2,
}

impl InterleavedVertexAttribute for VertexSprite {
	unsafe fn setup_vertex_attrs(glc: &Context) {
		let mut attrib_index = 0;
		let mut offset = 0;
		let stride = Self::stride();

		glc.vertex_attrib_pointer_f32(attrib_index, 2, glow::FLOAT, false, stride, offset);
		glc.enable_vertex_attrib_array(attrib_index);
		offset += mem::size_of::<Vec2>() as i32;
		attrib_index += 1;

		glc.vertex_attrib_pointer_f32(attrib_index, 2, glow::FLOAT, false, stride, offset);
		glc.enable_vertex_attrib_array(attrib_index);
		// offset += mem::size_of::<Vec2>() as i32;
		// attrib_index += 1;
	}
}

#[derive(Debug)]
pub struct VertexBuffer {
	glc: Arc<Context>,
	vao: <Context as HasContext>::VertexArray,
	vbo: <Context as HasContext>::Buffer,
	// size: i32,
}

impl VertexBuffer {
	pub fn new<T>(glc: Arc<Context>, buf: Box<[T]>) -> Self
	where T: InterleavedVertexAttribute + Pod {
		let (vao, vbo) = unsafe {
			let glc = &glc;
			let vao = glc.create_vertex_array().unwrap();
			glc.bind_vertex_array(Some(vao));
			let vbo = glc.create_buffer().unwrap();
			glc.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
			glc.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&buf), glow::STATIC_DRAW);
			T::setup_vertex_attrs(glc);
			glc.bind_buffer(glow::ARRAY_BUFFER, None);
			glc.bind_vertex_array(None);
			(vao, vbo)
		};
		// let size = buf.len() as i32;
		Self {
			glc,
			vao,
			vbo,
			// size,
		}
	}
	pub fn from_surface(glc: Arc<Context>, surf: &MD3Surface) -> Self {
		let buf: Vec<VertexMD3> = surf.texcoords.iter().enumerate()
			.map(|(index, uv)| VertexMD3 {index: index as u32, uv: uv.0})
			.collect();
		VertexBuffer::new(glc, buf.into_boxed_slice())
	}
}

impl Drop for VertexBuffer {
	fn drop(&mut self) {
		#[cfg(feature = "log_drop_gl_resources")]
		println!("Drop VertexBuffer");
		let glc = &self.glc;
		unsafe {
			glc.delete_vertex_array(self.vao);
			glc.delete_buffer(self.vbo);
		}
	}
}

pub trait IndexInteger { const GL_TYPE: u32; }
impl IndexInteger for u8 { const GL_TYPE: u32 = glow::UNSIGNED_BYTE; }
impl IndexInteger for u16 { const GL_TYPE: u32 = glow::UNSIGNED_SHORT; }
impl IndexInteger for u32 { const GL_TYPE: u32 = glow::UNSIGNED_INT; }

#[derive(Debug)]
pub struct IndexBuffer<I> where I : IndexInteger + Pod {
	glc: Arc<Context>,
	ebo: <Context as HasContext>::Buffer,
	size: i32,
	// Used to access OpenGL constant for the index data type (GL_TYPE)
	itype: PhantomData<I>,
}

impl<I> IndexBuffer<I> where I : IndexInteger + Pod {
	pub fn new(glc: Arc<Context>, buf: Vec<I>) -> Self {
		let ebo = unsafe {
			let ebo = glc.create_buffer().unwrap();
			glc.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
			glc.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, bytemuck::cast_slice(&buf), glow::STATIC_DRAW);
			glc.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
			ebo
		};
		let size = buf.len() as i32;
		Self {
			glc,
			ebo,
			size,
			itype: PhantomData,
		}
	}
}

impl IndexBuffer<u32> {
	pub fn from_surface(glc: Arc<Context>, surf: &MD3Surface) -> Self {
		let buf = surf.triangles.iter().flat_map(|t| t.0).collect();
		IndexBuffer::new(glc, buf)
	}
}

impl<I> Drop for IndexBuffer<I> where I : IndexInteger + Pod {
	fn drop(&mut self) {
		#[cfg(feature = "log_drop_gl_resources")]
		println!("Drop IndexBuffer");
		let glc = &self.glc;
		unsafe { glc.delete_buffer(self.ebo); }
	}
}

#[derive(Debug)]
pub struct Texture {
	glc: Arc<Context>,
	tex: <Context as HasContext>::Texture,
}

impl Drop for Texture {
	fn drop(&mut self) {
		#[cfg(feature = "log_drop_gl_resources")]
		println!("Drop Texture");
		let glc = &self.glc;
		unsafe {
			glc.delete_texture(self.tex);
		}
	}
}

impl Texture {
	pub fn try_from_surface(glc: Arc<Context>, tex: &Surface) -> Result<Self, AError> {
		unsafe {
			let texture = glc.create_texture().map_err(AError::msg)?;
			glc.bind_texture(glow::TEXTURE_2D, Some(texture));
			// NOTE: 16-bit images are untested!
			let tex_iformat: i32 = match tex.texture_type {
				SurfaceType::U8RGBA => glow::RGBA32F,
				SurfaceType::U8RGB => glow::RGB32F,
				SurfaceType::U16RGB => glow::RGB32F,
				SurfaceType::U16RGBA => glow::RGBA32F,
				SurfaceType::F32RGB => glow::RGB32F,
				SurfaceType::F32RGBA => glow::RGBA32F,
			}.try_into().unwrap();
			let tex_format = match tex.texture_type {
				SurfaceType::U8RGBA => glow::RGBA,
				SurfaceType::U8RGB => glow::RGB,
				SurfaceType::U16RGB => glow::RGB16UI,
				SurfaceType::U16RGBA => glow::RGBA16UI,
				SurfaceType::F32RGB => glow::RGB32F,
				SurfaceType::F32RGBA => glow::RGBA32F,
			};
			let data_type = match tex.texture_type {
				SurfaceType::U8RGBA => glow::UNSIGNED_BYTE,
				SurfaceType::U8RGB => glow::UNSIGNED_BYTE,
				SurfaceType::U16RGB => glow::UNSIGNED_SHORT,
				SurfaceType::U16RGBA => glow::UNSIGNED_SHORT,
				SurfaceType::F32RGB => glow::FLOAT,
				SurfaceType::F32RGBA => glow::FLOAT,
			};
			let min_filter = glow::LINEAR as i32;
			let mag_filter = glow::LINEAR as i32;
			glc.tex_image_2d(glow::TEXTURE_2D, 0, tex_iformat,
				tex.width as i32, tex.height as i32, 0, tex_format,
				data_type, Some(&tex.data));
			GLError::get(&glc)?;
			glc.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
			glc.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
			glc.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, min_filter);
			glc.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, mag_filter);
			glc.bind_texture(glow::TEXTURE_2D, None);
			Ok(Texture{
				tex: texture,
				glc,
			})
		}
	}
	pub fn try_from_md3(glc: Arc<Context>, surf: &MD3Surface) -> Result<(Self, u32), AError> {
		// Animations may need some additional processing
		enum UploadError {
			GLError(GLError),
			Message(String),
			TooBig,
		}
		fn try_upload(glc: &Context, width: i32, height: i32, data: &[u8]) -> Result<<Context as HasContext>::Texture, UploadError> {
			let internal_format = glow::RGBA32I as i32;
			let tex_format = glow::RGBA_INTEGER;
			let data_type = glow::INT;
			let target = glow::TEXTURE_2D;
			unsafe {
				let texture = glc.create_texture().map_err(UploadError::Message)?;
				glc.bind_texture(target, Some(texture));
				glc.tex_image_2d(target, 0, internal_format, width, height, 0, tex_format, data_type, Some(data));
				match GLError::get(&glc) {
					Ok(_) => Ok(texture),
					Err(err) => {
						glc.delete_texture(texture);
						match err {
							GLError::InvalidValue => Err(UploadError::TooBig),
							e => Err(UploadError::GLError(e)),
						}
					},
				}
			}
		}
		let mut width = surf.num_verts as i32;
		let mut two_power = (1..MAX_TEXTURE_POT.get().copied().unwrap()).rev().filter(|&i| {
			2i32.pow(i) < width
		}).next().unwrap_or(0);
		let mut rows_per_frame;
		let tex_handle = loop {
			let an = surf.make_animation(Some(width as usize));
			rows_per_frame = an.rows_per_frame;
			let height = (an.rows_per_frame * an.frames) as i32;
			match try_upload(&glc, width, height, &an.data) {
				Ok(tex) => {
					break Ok(Texture {
						glc: Arc::clone(&glc),
						tex,
					});
				},
				Err(e) => {
					match e {
						UploadError::GLError(e) => break Err(AError::from(e)),
						UploadError::Message(m) => break Err(AError::msg(m)),
						UploadError::TooBig => {
							if two_power > 0 {
								width = 2i32.pow(two_power);
								two_power -= 1;
							} else {
								break Err(AError::msg("Animation is too big to upload to the GPU!"));
							}
						},
					}
				},
			}
		}?;
		let wrapping = glow::REPEAT as i32;
		let filter = glow::NEAREST as i32;
		let target = glow::TEXTURE_2D;
		unsafe {
			glc.tex_parameter_i32(target, glow::TEXTURE_WRAP_S, wrapping);
			glc.tex_parameter_i32(target, glow::TEXTURE_WRAP_T, wrapping);
			glc.tex_parameter_i32(target, glow::TEXTURE_MIN_FILTER, filter);
			glc.tex_parameter_i32(target, glow::TEXTURE_MAG_FILTER, filter);
		}
		Ok((tex_handle, rows_per_frame))
	}
	pub fn tex(&self) -> <Context as HasContext>::Texture {
		self.tex
	}
}

pub enum ShaderStage {
	Vertex,
	Fragment,
	Geometry,
}

impl From<ShaderStage> for u32 {
	fn from(v: ShaderStage) -> Self {
		match v {
			ShaderStage::Vertex => glow::VERTEX_SHADER,
			ShaderStage::Fragment => glow::FRAGMENT_SHADER,
			ShaderStage::Geometry => glow::GEOMETRY_SHADER,
		}
	}
}

#[derive(Debug)]
pub struct ShaderProgram<L>
where L: ShaderUniformLocations + Default {
	glc: Arc<Context>,
	prog: <Context as HasContext>::Program,
	// Make sure uniform structs match
	locations: L,
}

impl<L> ShaderProgram<L>
where L: ShaderUniformLocations + Default {
	pub fn activate(&self) -> Result<(), AError> {
		let glc = &self.glc;
		unsafe {
			glc.use_program(Some(self.prog));
		}
		Ok(())
	}
}

impl<L> Drop for ShaderProgram<L>
where L: ShaderUniformLocations + Default {
	fn drop(&mut self) {
		#[cfg(feature = "log_drop_gl_resources")]
		println!("Drop ShaderProgram");
		let glc = &self.glc;
		unsafe {
			glc.delete_program(self.prog);
		}
	}
}

struct Shader<'a> {
	stage: ShaderStage,
	source: &'a str,
}

pub struct ShaderProgramBuilder<'a, L>
where L: ShaderUniformLocations + Default {
	shaders: Vec<Shader<'a>>,
	location_type: PhantomData<L>,
}

impl<'a, L> ShaderProgramBuilder<'a, L>
where L: ShaderUniformLocations + Default {
	pub fn new() -> Self {
		Self {
			shaders: vec![],
			location_type: PhantomData
		}
	}
	pub fn add_shader(mut self, stage: ShaderStage, source: &'a str) -> Self {
		self.shaders.push(Shader { stage, source });
		self
	}
	pub fn build(self, glc: Arc<Context>) -> Result<ShaderProgram<L>, AError> {
		let prog = unsafe { glc.create_program().map_err(AError::msg)? };
		let mut shader_list = vec![];
		for shader in self.shaders {
			unsafe {
				let gl_shader = glc.create_shader(shader.stage.into()).map_err(AError::msg)?;
				glc.shader_source(gl_shader, shader.source);
				glc.compile_shader(gl_shader);
				if !glc.get_shader_compile_status(gl_shader) {
					let e = Err(glc.get_shader_info_log(gl_shader));
					for shader in shader_list {
						glc.delete_shader(shader);
					}
					glc.delete_program(prog);
					return e.map_err(AError::msg);
				}
				glc.attach_shader(prog, gl_shader);
				shader_list.push(gl_shader);
			}
		}
		unsafe {
			glc.link_program(prog);
			if !glc.get_program_link_status(prog) {
				let e = Err(glc.get_program_info_log(prog));
				for shader in shader_list {
					glc.delete_shader(shader);
				}
				glc.delete_program(prog);
				return e.map_err(AError::msg);
			}
			// The shaders are compiled, and the program is linked. The
			// shaders are not needed any more, since they are unlikely
			// to be re-used.
			for shader in shader_list {
				glc.delete_shader(shader);
			}
		}
		let locations = {
			let mut l = L::default();
			l.setup(&glc, prog);
			l
		};
		Ok(ShaderProgram {
			glc,
			prog,
			locations,
		})
	}
}

pub static MAX_TEXTURE_UNITS: OnceBox<u8> = OnceBox::new();
pub static MAX_TEXTURE_POT: OnceBox<u32> = OnceBox::new();

#[derive(Debug, Clone, Copy)]
pub struct TextureUnit(pub u8);

impl Default for TextureUnit {
	fn default() -> Self {
		Self(1)
	}
}

impl Deref for TextureUnit {
	type Target = u8;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for TextureUnit {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl TextureUnit {
	#[inline]
	pub fn max() -> u8 {
		MAX_TEXTURE_UNITS.get().copied().unwrap_or(32)
	}
	pub fn slot(self) -> u32 {
		let max_unit = Self::max();
		match self.0 {
			// All OpenGL implementations have at least 16 texture slots
			// available
			0 => 0,
			1 => glow::TEXTURE0,
			2 => glow::TEXTURE1,
			3 => glow::TEXTURE2,
			4 => glow::TEXTURE3,
			5 => glow::TEXTURE4,
			6 => glow::TEXTURE5,
			7 => glow::TEXTURE6,
			8 => glow::TEXTURE7,
			9 => glow::TEXTURE8,
			10 => glow::TEXTURE9,
			11 => glow::TEXTURE10,
			12 => glow::TEXTURE11,
			13 => glow::TEXTURE12,
			14 => glow::TEXTURE13,
			15 => glow::TEXTURE14,
			16 => glow::TEXTURE15,
			x if x == 17 && x < max_unit => glow::TEXTURE16,
			x if x == 18 && x < max_unit => glow::TEXTURE17,
			x if x == 19 && x < max_unit => glow::TEXTURE18,
			x if x == 20 && x < max_unit => glow::TEXTURE19,
			x if x == 21 && x < max_unit => glow::TEXTURE20,
			x if x == 22 && x < max_unit => glow::TEXTURE21,
			x if x == 23 && x < max_unit => glow::TEXTURE22,
			x if x == 24 && x < max_unit => glow::TEXTURE23,
			x if x == 25 && x < max_unit => glow::TEXTURE24,
			x if x == 26 && x < max_unit => glow::TEXTURE25,
			x if x == 27 && x < max_unit => glow::TEXTURE26,
			x if x == 28 && x < max_unit => glow::TEXTURE27,
			x if x == 29 && x < max_unit => glow::TEXTURE28,
			x if x == 30 && x < max_unit => glow::TEXTURE29,
			x if x == 31 && x < max_unit => glow::TEXTURE30,
			x if x == 32 && x < max_unit => glow::TEXTURE31,
			x => glow::TEXTURE0 + x.min(max_unit) as u32,
		}
	}
	pub fn uniform(self) -> i32 {
		let max_unit = Self::max();
		match self.0 {
			x if x >= 1 && x <= max_unit => x - 1,
			_ => 0,
		}.into()
	}
	pub fn next(&mut self) -> () {
		self.0 += 1;
	}
}

pub struct BasicModel<I, U, L> where
	I : IndexInteger + Pod,
	U: ShaderUniforms<L>,
	L: ShaderUniformLocations + Default
{
	pub vertex: VertexBuffer,
	pub index: IndexBuffer<I>,
	pub shader: Rc<ShaderProgram<L>>,
	pub uniforms: U,
}

impl<I, U, L> BasicModel<I, U, L> where
	I : IndexInteger + Pod,
	U: ShaderUniforms<L>,
	L: ShaderUniformLocations + Default
{
	pub fn render<F>(&mut self, glc: &Context, modify_uniforms: F) -> Result<(), AError>
	where F: Fn(&mut U) -> () {
		self.shader.activate()?;
		modify_uniforms(&mut self.uniforms);
		self.uniforms.set(glc, &self.shader.locations);
		unsafe {
			glc.bind_vertex_array(Some(self.vertex.vao));
			glc.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.index.ebo));
			glc.draw_elements(glow::TRIANGLES, self.index.size, I::GL_TYPE, 0);
			GLError::get(glc)?;
		}
		Ok(())
	}
}
