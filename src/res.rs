use std::borrow::Cow;
use std::error::Error;
use std::path::Path;
use std::env;
use std::fs::{self, File};
use png::{Decoder, Info as PNGInfo};
use crate::render::{TextureUnit/* , VertexRes */};
// use glam::Vec3;

#[derive(Debug, Clone, Copy, Default)]
pub enum TextureType {
	I32RGBA,
	#[default]
	U8RGBA,
	U8RGB,
}

impl TextureType {
	pub fn channels(&self) -> u8 {
		match self {
			TextureType::I32RGBA => 4,
			TextureType::U8RGBA => 4,
			TextureType::U8RGB => 3,
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct Texture {
	pub width: u32,
	pub height: u32,
	pub texture_type: TextureType,
	pub data: Box<[u8]>,
}

impl Texture {
	pub fn read_png(path: impl AsRef<Path>) -> Result<Texture, Box<dyn Error>> {
		use png::BitDepth::*;
		use png::ColorType::*;
		use TextureType::*;
		let png_decoder = Decoder::new(File::open(path)?);
		let mut png_reader = png_decoder.read_info()?;
		let PNGInfo {width, height, bit_depth, color_type, ..} = *png_reader.info();
		Ok(Texture {
			width,
			height,
			texture_type: match (bit_depth, color_type) {
				(Eight, Rgb) => Ok(U8RGB),
				(Eight, Rgba) => Ok(U8RGBA),
				_ => Err("Unsupported texture type")
			}?,
			data: {
				let channels = match color_type {
					Rgb => Ok(3),
					Rgba => Ok(4),
					_ => Err("Unsupported color type"),
				}?;
				let mut pixels = vec![0; (width * height * channels) as usize];
				png_reader.next_frame(&mut pixels)?;
				pixels.into_boxed_slice()
			},
		})
	}
}

pub struct AppResources {
	pub null_texture: Texture,
	pub null_texunit: TextureUnit,
	pub md3_pixel_shader: String,
	pub md3_vertex_shader: String,
}

impl AppResources {
	pub fn try_load(path: Option<&dyn AsRef<Path>>) -> Result<Box<AppResources>, Box<dyn Error>> {
		let path = match path {
			Some(ref p) => Cow::from(p.as_ref()),
			None => {
				let mut pwd = env::current_dir()?;
				pwd.push("assets");
				Cow::from(pwd)
			},
		};
		let null_texture = Texture::read_png(path.join("null.png"))?;
		let md3_vertex_shader = fs::read_to_string(path.join("md3.vert"))?;
		let md3_pixel_shader = fs::read_to_string(path.join("md3.frag"))?;
		Ok(Box::new(AppResources {
			null_texture,
			null_texunit: TextureUnit(1),
			md3_pixel_shader,
			md3_vertex_shader,
		}))
	}
}

/* 
const AXES_V: [VertexRes; 24] = [
VertexRes { position: Vec3::new(-1.0, 21.0, -1.0), colour: Vec3::new(0.0, 1.0, 0.0) },
VertexRes { position: Vec3::new(-1.0, 21.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 21.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 21.0, -1.0), colour: Vec3::new(0.0, 1.0, 0.0) },
VertexRes { position: Vec3::new(-1.0, 1.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0) },
VertexRes { position: Vec3::new(-1.0, -1.0, -1.0), colour: Vec3::new(0.0, 1.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, -1.0), colour: Vec3::new(0.0, 1.0, 0.0) },
VertexRes { position: Vec3::new(21.0, 1.0, -1.0), colour: Vec3::new(1.0, 0.0, 0.0) },
VertexRes { position: Vec3::new(21.0, 1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0) },
VertexRes { position: Vec3::new(21.0, -1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0) },
VertexRes { position: Vec3::new(21.0, -1.0, -1.0), colour: Vec3::new(1.0, 0.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, -1.0), colour: Vec3::new(1.0, 0.0, 0.0) },
VertexRes { position: Vec3::new(1.0, -1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0) },
VertexRes { position: Vec3::new(-1.0, -1.0, -1.0), colour: Vec3::new(1.0, 0.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 21.0), colour: Vec3::new(0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(-1.0, 1.0, 21.0), colour: Vec3::new(0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(-1.0, -1.0, 21.0), colour: Vec3::new(0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(1.0, -1.0, 21.0), colour: Vec3::new(0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(-1.0, 1.0, 1.0), colour: Vec3::new(0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 1.0), colour: Vec3::new(0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(-1.0, -1.0, -1.0), colour: Vec3::new(0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(1.0, -1.0, 1.0), colour: Vec3::new(0.0, 0.0, 1.0) }
];
const AXES_I: [u32; 90] = [0, 1, 2, 0, 2, 3, 4, 1, 0, 4, 0, 5, 6, 2, 1, 6, 1, 4, 7, 3, 2, 7, 2, 6, 5, 0, 3, 5, 3, 7, 8, 9, 10, 8, 10, 11, 12, 9, 8, 12, 8, 13, 14, 10, 9, 14, 9, 12, 15, 11, 10, 15, 10, 14, 13, 8, 11, 13, 11, 15, 16, 17, 18, 16, 18, 19, 20, 17, 16, 20, 16, 21, 22, 18, 17, 22, 17, 20, 23, 19, 18, 23, 18, 22, 21, 16, 19, 21, 19, 23];
 */