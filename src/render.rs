use std::error::Error;
use glam::{Vec2, Vec3};
use crate::md3::MD3Surface;
use crate::res::{Texture as RTexture, TextureType};
use glow::{Context, HasContext};
use std::{mem, sync::Arc, marker::PhantomData};
use shrinkwraprs::Shrinkwrap;
use bytemuck::{Pod, Zeroable};
use crate::eutil::gl_get_error;

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod, Default)]
pub struct VertexMD3 {
	index: u32,
	uv: Vec2,
}

pub trait InterleavedVertexAttribute {
	unsafe fn setup_vertex_attrs(glc: &Context);
}

impl InterleavedVertexAttribute for VertexMD3 {
	unsafe fn setup_vertex_attrs(glc: &Context) {
		let mut attrib_index = 0;

		glc.vertex_attrib_pointer_i32(attrib_index, 1, glow::UNSIGNED_INT,
			mem::size_of::<Self>() as i32, 0);
		glc.enable_vertex_attrib_array(attrib_index);
		attrib_index += 1;

		glc.vertex_attrib_pointer_f32(attrib_index, 2, glow::FLOAT, false,
			mem::size_of::<Self>() as i32, mem::size_of::<u32>() as i32);
		glc.enable_vertex_attrib_array(attrib_index);
		// attrib_index += 1;
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod, Default)]
pub struct VertexRes {
	pub position: Vec3,
	pub colour: Vec3,
}

impl InterleavedVertexAttribute for VertexRes {
	unsafe fn setup_vertex_attrs(glc: &Context) {
		let mut attrib_index = 0;

		glc.vertex_attrib_pointer_f32(attrib_index, 3, glow::FLOAT, false, mem::size_of::<Self>() as i32, 0);
		glc.enable_vertex_attrib_array(attrib_index);
		attrib_index += 1;

		glc.vertex_attrib_pointer_f32(attrib_index, 3, glow::FLOAT, false, mem::size_of::<Self>() as i32, mem::size_of::<Vec3>() as i32);
		glc.enable_vertex_attrib_array(attrib_index);
		// attrib_index += 1;
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

		glc.vertex_attrib_pointer_f32(attrib_index, 2, glow::FLOAT, false, mem::size_of::<Self>() as i32, 0);
		glc.enable_vertex_attrib_array(attrib_index);
		attrib_index += 1;

		glc.vertex_attrib_pointer_f32(attrib_index, 2, glow::FLOAT, false, mem::size_of::<Self>() as i32, mem::size_of::<Vec2>() as i32);
		glc.enable_vertex_attrib_array(attrib_index);
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
	pub fn new<T>(glc: Arc<Context>, buf: Vec<T>) -> Self
	where T: InterleavedVertexAttribute + Pod {
		let (vao, vbo) = unsafe {
			let glc = &glc;
			let vao = glc.create_vertex_array().unwrap();
			glc.bind_vertex_array(Some(vao));
			let vbo = glc.create_buffer().unwrap();
			glc.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
			glc.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice::<T, u8>(&buf), glow::STATIC_DRAW);
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
		let buf = surf.texcoords.iter().enumerate()
			.map(|(index, uv)| VertexMD3 {index: index as u32, uv: uv.0})
			.collect();
		VertexBuffer::new(glc, buf)
	}
}

impl Drop for VertexBuffer {
	fn drop(&mut self) {
		// println!("Drop VertexBuffer");
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
// impl IndexInteger for u64 { const GL_TYPE: u32 = glow::UNSIGNED_LONG; }
// impl IndexInteger for u128 { const GL_TYPE: u32 = glow::UNSIGNED_LONG_LONG; }
// impl IndexInteger for usize { const GL_TYPE: u32 = glow::UNSIGNED_PTR; }


#[derive(Debug)]
pub struct IndexBuffer<I> where I : IndexInteger + Pod {
	glc: Arc<Context>,
	ebo: <Context as HasContext>::Buffer,
	size: i32,
	_type: PhantomData<I>
}

impl<I> IndexBuffer<I> where I : IndexInteger + Pod {
	pub fn new(glc: Arc<Context>, buf: Vec<I>) -> Self {
		let ebo = unsafe {
			let ebo = glc.create_buffer().unwrap();
			glc.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
			glc.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, bytemuck::cast_slice::<I, u8>(&buf), glow::STATIC_DRAW);
			glc.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
			ebo
		};
		let size = buf.len() as i32;
		Self {
			glc,
			ebo,
			size,
			_type: PhantomData,
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
		// println!("Drop IndexBuffer");
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
		// println!("Drop Texture");
		let glc = &self.glc;
		unsafe {
			glc.delete_texture(self.tex);
		}
	}
}

impl Texture {
	pub fn try_from_texture(glc: Arc<Context>, tex: &RTexture) -> Result<Self, Box<dyn Error>> {
		unsafe {
			let texture = glc.create_texture()?;
			glc.bind_texture(glow::TEXTURE_2D, Some(texture));
			let tex_iformat: i32 = match tex.texture_type {
				TextureType::I32RGBA => glow::RGBA32I,
				TextureType::U8RGBA => glow::RGBA,
				TextureType::U8RGB => glow::RGB,
			}.try_into().unwrap();
			let tex_format = match tex.texture_type {
				TextureType::I32RGBA => glow::RGBA_INTEGER,
				TextureType::U8RGBA => glow::RGBA,
				TextureType::U8RGB => glow::RGB,
			};
			let data_type = match tex.texture_type {
				TextureType::I32RGBA => glow::INT,
				TextureType::U8RGBA => glow::UNSIGNED_BYTE,
				TextureType::U8RGB => glow::UNSIGNED_BYTE,
			};
			let (min_filter, mag_filter) = match tex.texture_type {
				TextureType::I32RGBA => (glow::NEAREST as i32, glow::NEAREST as i32),
				TextureType::U8RGBA => (glow::LINEAR as i32, glow::LINEAR as i32),
				TextureType::U8RGB => (glow::LINEAR as i32, glow::LINEAR as i32),
			};
			glc.tex_image_2d(glow::TEXTURE_2D, 0, tex_iformat,
				tex.width as i32, tex.height as i32, 0, tex_format,
				data_type, Some(&tex.data));
			gl_get_error(&glc)?;
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
	pub fn tex(&self) -> <Context as HasContext>::Texture {
		self.tex
	}
}

#[derive(Debug)]
pub struct ShaderProgram {
	glc: Arc<Context>,
	prog: <Context as HasContext>::Program,
	shaders: Vec<<Context as HasContext>::Shader>,
	ready: bool,
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

impl ShaderProgram {
	pub fn new(glc: Arc<Context>) -> Result<Self, Box<dyn Error>> {
		unsafe {
			let prog = glc.create_program()?;
			Ok(Self {
				glc,
				prog,
				shaders: vec![],
				ready: false,
			})
		}
	}
	pub fn add_shader(&mut self, stage: ShaderStage, source: &str) -> Result<(), String> {
		let glc = &self.glc;
		let stage = u32::from(stage);
		unsafe {
			let shader = glc.create_shader(stage)?;
			glc.shader_source(shader, source);
			glc.compile_shader(shader);
			if !glc.get_shader_compile_status(shader) {
				let e = Err(glc.get_shader_info_log(shader));
				glc.delete_shader(shader);
				return e;
			}
			self.shaders.push(shader);
		}
		Ok(())
	}
	pub fn prepare(&mut self) -> Result<(), String> {
		let glc = &self.glc;
		unsafe {
			for shader in self.shaders.iter().copied() {
				glc.attach_shader(self.prog, shader);
			}
			glc.link_program(self.prog);
			if !glc.get_program_link_status(self.prog) {
				let e = Err(glc.get_program_info_log(self.prog));
				glc.delete_program(self.prog);
				return e;
			}
			for shader in self.shaders.iter().copied() {
				glc.delete_shader(shader);
			}
			self.shaders.clear();
		}
		self.ready = true;
		Ok(())
	}
	pub fn prog(&self) -> <Context as HasContext>::Program {
		self.prog
	}
	pub fn activate(&self) -> Result<(), String> {
		if !self.ready {
			return Err(String::from("Not ready"));
		}
		let glc = &self.glc;
		unsafe {
			glc.use_program(Some(self.prog));
		}
		Ok(())
	}
}

impl Drop for ShaderProgram {
	fn drop(&mut self) {
		// println!("Drop ShaderProgram");
		let glc = &self.glc;
		unsafe {
			for shader in self.shaders.iter().copied() {
				glc.delete_shader(shader);
			}
			glc.delete_program(self.prog);
		}
	}
}

#[derive(Debug, Clone, Copy, Shrinkwrap, Default)]
#[shrinkwrap(mutable)]
pub struct TextureUnit(pub u8);

impl TextureUnit {
	pub fn gl_id(self) -> u32 {
		match self.0 {
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
			17 => glow::TEXTURE16,
			18 => glow::TEXTURE17,
			19 => glow::TEXTURE18,
			20 => glow::TEXTURE19,
			21 => glow::TEXTURE20,
			22 => glow::TEXTURE21,
			23 => glow::TEXTURE22,
			24 => glow::TEXTURE23,
			25 => glow::TEXTURE24,
			26 => glow::TEXTURE25,
			27 => glow::TEXTURE26,
			28 => glow::TEXTURE27,
			29 => glow::TEXTURE28,
			30 => glow::TEXTURE29,
			31 => glow::TEXTURE30,
			32 => glow::TEXTURE31,
			_ => 0,
		}
	}
	pub fn gl_u(self) -> i32 {
		match self.0 {
			x if x >= 1 && x <= 32 => x - 1,
			_ => 0,
		}.into()
	}
}

pub fn render<I>(
	glc: &Context,
	vertices: &VertexBuffer,
	indices: &IndexBuffer<I>) -> Result<(), Box<dyn Error>>
where
	I : IndexInteger + Pod {
	unsafe {
		glc.bind_vertex_array(Some(vertices.vao));
		glc.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(indices.ebo));
		glc.draw_elements(
		glow::TRIANGLES, indices.size, I::GL_TYPE, 0);
		gl_get_error(glc)?;
	}
	Ok(())
}

/* 
pub fn build_vbuffer(surf: &MD3Surface) -> Vec<Vertex> {
	surf.triangles.iter().flat_map(|t| t.0)
		.map(|idx| Vertex {index: idx, uv: surf.texcoords[idx as usize].0})
		.collect()
}
 */
/* 
pub fn build_ivbuffers(surf: &MD3Surface) -> (Vec<u32>, Vec<Vertex>) {
	let vertices = surf.texcoords.iter().enumerate().map(
		|(index, uv)| {let uv = uv.0; Vertex {index: index as u32, uv}})
		.collect();
	let indices = surf.triangles.iter().flat_map(|t| t.0)
		.collect();
	(indices, vertices)
}
 */
/* 
pub fn upload_ibuffer(buffer: &[u32], glc: &Context) -> Result<(), Box<dyn Error>> {
	unsafe {
		let ebo = glc.create_buffer()?;
		glc.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
		let buf_bytes: Box<[u8]> = buffer.iter().copied().flat_map(u32::to_ne_bytes).collect();
		glc.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, &buf_bytes, glow::STATIC_DRAW);
		glc.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
	}
	Ok(())
}
 */
