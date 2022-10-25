use std::error::Error;
use std::path::Path;
use std::env;
use std::fs::{self, File};
use png::{Decoder, Info as PNGInfo};
use crate::render::TextureUnit;

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
		let pwd = {
			let mut pwd = env::current_dir()?;
			pwd.push("assets");
			pwd
		};
		let path = match path {
			Some(ref p) => p.as_ref(),
			None => pwd.as_ref(),
		};
		let null_texture = Texture::read_png(path.join("null.png"))?;
		let md3_vertex_shader = fs::read_to_string(path.join("shader.vert"))?;
		let md3_pixel_shader = fs::read_to_string(path.join("shader.frag"))?;
		Ok(Box::new(AppResources {
			null_texture,
			null_texunit: TextureUnit(1),
			md3_pixel_shader,
			md3_vertex_shader,
		}))
	}
}
