use anyhow::Error;
use std::{
	borrow::Cow,
	env,
	path::Path,
	fs::{self, File},
	io::BufReader,
	ops::Deref,
};
use crate::render::VertexRes;
use glam::Vec3;
use image::{io::Reader, ImageBuffer, Pixel, DynamicImage::*};
use bytemuck::Pod;

#[derive(Debug, Clone, Copy, Default)]
pub enum SurfaceType {
	#[default]
	U8RGBA,
	U8RGB,
	U16RGB,
	U16RGBA,
	F32RGB,
	F32RGBA,
}

/* impl SurfaceType {
	pub fn channels(&self) -> u8 {
		match self {
			SurfaceType::U8RGBA => 4,
			SurfaceType::U8RGB => 3,
			SurfaceType::U16RGB => 3,
			SurfaceType::U16RGBA => 4,
			SurfaceType::F32RGB => 3,
			SurfaceType::F32RGBA => 4,
		}
	}
} */

#[derive(Debug, Clone, Default)]
pub struct Surface {
	pub width: u32,
	pub height: u32,
	pub texture_type: SurfaceType,
	pub data: Box<[u8]>,
}

impl Surface {
	pub fn read_image(path: impl AsRef<Path>) -> Result<Surface, Error> {
		use SurfaceType::*;
		let file_reader = BufReader::new(File::open(path)?);
		let image = Reader::new(file_reader)
			.with_guessed_format()?
			.decode()?;
		fn to_surface<P: Pixel, T>(buf: ImageBuffer<P, T>, fmt: SurfaceType) -> Surface
		where
			P: Pixel,
			T: Deref<Target = [<P as image::Pixel>::Subpixel]>,
			<P as image::Pixel>::Subpixel: Pod
			{
			let (width, height) = buf.dimensions();
			Surface {
				width, height,
				texture_type: fmt,
				data: bytemuck::cast_slice(&buf.into_raw()).into()
			}
		}
		match image {
			ImageLuma8(_i) => Err(Error::msg("Unsupported format: ImageLuma8")),
			ImageLumaA8(_i) => Err(Error::msg("Unsupported format: ImageLumaA8")),
			ImageRgb8(i) => Ok(to_surface(i, U8RGB)),
			ImageRgba8(i) => Ok(to_surface(i, U8RGBA)),
			ImageLuma16(_i) => Err(Error::msg("Unsupported format: ImageLuma16")),
			ImageLumaA16(_i) => Err(Error::msg("Unsupported format: ImageLumaA16")),
			ImageRgb16(i) => Ok(to_surface(i, U16RGB)),
			ImageRgba16(i) => Ok(to_surface(i, U16RGBA)),
			ImageRgb32F(i) => Ok(to_surface(i, F32RGB)),
			ImageRgba32F(i) => Ok(to_surface(i, F32RGBA)),
			_ => todo!(),
		}
	}
}

pub struct AppResources {
	pub null_surface: Surface,
	pub md3_pixel_shader: String,
	pub md3_vertex_shader: String,
	pub res_pixel_shader: String,
	pub res_vertex_shader: String,
}

impl AppResources {
	pub fn try_load(path: Option<&dyn AsRef<Path>>) -> Result<Box<AppResources>, Error> {
		let path = match path {
			Some(ref p) => Cow::from(p.as_ref()),
			None => {
				let mut pwd = env::current_dir()?;
				pwd.push("assets");
				Cow::from(pwd)
			},
		};
		let null_texture = Surface::read_image(path.join("null.png"))?;
		let md3_vertex_shader = fs::read_to_string(path.join("md3.vert"))?;
		let md3_pixel_shader = fs::read_to_string(path.join("md3.frag"))?;
		let res_vertex_shader = fs::read_to_string(path.join("res.vert"))?;
		let res_pixel_shader = fs::read_to_string(path.join("res.frag"))?;
		Ok(Box::new(AppResources {
			null_surface: null_texture,
			md3_pixel_shader,
			md3_vertex_shader,
			res_pixel_shader,
			res_vertex_shader,
		}))
	}
}

// A nice blue colour is (32, 144, 255) = (0.125, 0.5625, 1.0)
// Generated by a script in assets/axes.blend
pub const AXES_V: [VertexRes; 24] = [
VertexRes { position: Vec3::new(0.0, 50.0, 0.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-0.0, -0.0, -1.0) },
VertexRes { position: Vec3::new(0.0, 50.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(1.0, 50.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 50.0, 0.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-0.0, 0.0, -1.0) },
VertexRes { position: Vec3::new(0.0, 1.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(0.0, 0.0, 0.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-0.0, 0.0, -1.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 0.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-0.0, 0.0, -1.0) },
VertexRes { position: Vec3::new(50.0, 1.0, 0.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(0.0, 0.0, -1.0) },
VertexRes { position: Vec3::new(50.0, 1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(-0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(50.0, 0.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(0.0, -1.0, -0.0) },
VertexRes { position: Vec3::new(50.0, 0.0, 0.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(0.0, 0.0, -1.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(-0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 0.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(0.0, 0.0, -1.0) },
VertexRes { position: Vec3::new(1.0, 0.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(0.0, -1.0, -0.0) },
VertexRes { position: Vec3::new(0.0, 0.0, 0.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(0.0, 0.0, -1.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 50.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(0.0, 1.0, 50.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(-1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(0.0, 0.0, 50.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(0.0, -1.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 0.0, 50.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(0.0, 1.0, 1.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(-1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 1.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(0.0, 0.0, 0.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(0.0, -1.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 0.0, 1.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(1.0, -0.0, 0.0) }
];
pub const AXES_I: [u8; 90] = [0, 1, 2, 0, 2, 3, 4, 1, 0, 4, 0, 5, 6, 2, 1, 6, 1, 4, 7, 3, 2, 7, 2, 6, 5, 0, 3, 5, 3, 7, 8, 9, 10, 8, 10, 11, 12, 9, 8, 12, 8, 13, 14, 10, 9, 14, 9, 12, 15, 11, 10, 15, 10, 14, 13, 8, 11, 13, 11, 15, 16, 17, 18, 16, 18, 19, 20, 17, 16, 20, 16, 21, 22, 18, 17, 22, 17, 20, 23, 19, 18, 23, 18, 22, 21, 16, 19, 21, 19, 23];

pub const TAGAXES_V: [VertexRes; 60] = [
VertexRes { position: Vec3::new(1.0, 1.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(1.0, 0.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, -1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(1.0, 0.0, -0.0) },
VertexRes { position: Vec3::new(1.0, 30.0, -1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(1.0, 0.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 30.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(1.0, 0.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, -1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(0.0, 1.0, -0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(0.0, 1.0, 0.0) },
VertexRes { position: Vec3::new(30.0, 1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(0.0, 1.0, -0.0) },
VertexRes { position: Vec3::new(30.0, 1.0, -1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(0.0, 1.0, -0.0) },
VertexRes { position: Vec3::new(-1.0, 1.0, 1.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(-1.0, -0.0, -0.0) },
VertexRes { position: Vec3::new(-1.0, -1.0, -1.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(-1.0, 0.0, 0.0) },
VertexRes { position: Vec3::new(-1.0, -1.0, 30.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(-1.0, -0.0, -0.0) },
VertexRes { position: Vec3::new(-1.0, 1.0, 30.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(-1.0, -0.0, -0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 30.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(0.0, -0.0, 1.0) },
VertexRes { position: Vec3::new(-1.0, 1.0, 30.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(-1.0, -1.0, 30.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(0.0, -0.0, 1.0) },
VertexRes { position: Vec3::new(1.0, -1.0, 30.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(0.0, -0.0, 1.0) },
VertexRes { position: Vec3::new(-1.0, -1.0, -1.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(0.0, -1.0, -0.0) },
VertexRes { position: Vec3::new(1.0, -1.0, 1.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(0.0, -1.0, -0.0) },
VertexRes { position: Vec3::new(1.0, -1.0, 30.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(0.0, -1.0, -0.0) },
VertexRes { position: Vec3::new(-1.0, -1.0, 30.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(0.0, -1.0, -0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 1.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(0.0, 1.0, 0.0) },
VertexRes { position: Vec3::new(-1.0, 1.0, 1.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(-0.0, 1.0, 0.0) },
VertexRes { position: Vec3::new(-1.0, 1.0, 30.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(0.0, 1.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 30.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(0.0, 1.0, 0.0) },
VertexRes { position: Vec3::new(1.0, -1.0, 1.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 1.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(1.0, 0.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 30.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(1.0, -1.0, 30.0), colour: Vec3::new(0.1875, 0.4375, 1.0), normal: Vec3::new(1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(-1.0, 30.0, -1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(0.0, 1.0, -0.0) },
VertexRes { position: Vec3::new(-1.0, 30.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(0.0, 1.0, 0.0) },
VertexRes { position: Vec3::new(1.0, 30.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(0.0, 1.0, -0.0) },
VertexRes { position: Vec3::new(1.0, 30.0, -1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(0.0, 1.0, -0.0) },
VertexRes { position: Vec3::new(1.0, 1.0, -1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-0.0, -0.0, -1.0) },
VertexRes { position: Vec3::new(-1.0, -1.0, -1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(0.0, 0.0, -1.0) },
VertexRes { position: Vec3::new(-1.0, 30.0, -1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-0.0, -0.0, -1.0) },
VertexRes { position: Vec3::new(1.0, 30.0, -1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-0.0, -0.0, -1.0) },
VertexRes { position: Vec3::new(-1.0, 1.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(1.0, 30.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(-1.0, 30.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(-1.0, -1.0, -1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(-1.0, 1.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(-1.0, 30.0, 1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(-1.0, 30.0, -1.0), colour: Vec3::new(0.0, 1.0, 0.0), normal: Vec3::new(-1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(30.0, 1.0, -1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(30.0, 1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(30.0, -1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(30.0, -1.0, -1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(1.0, -0.0, 0.0) },
VertexRes { position: Vec3::new(1.0, -1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(-0.0, -1.0, -0.0) },
VertexRes { position: Vec3::new(-1.0, -1.0, -1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(0.0, -1.0, 0.0) },
VertexRes { position: Vec3::new(30.0, -1.0, -1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(-0.0, -1.0, -0.0) },
VertexRes { position: Vec3::new(30.0, -1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(-0.0, -1.0, -0.0) },
VertexRes { position: Vec3::new(-1.0, -1.0, -1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(-0.0, 0.0, -1.0) },
VertexRes { position: Vec3::new(1.0, 1.0, -1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(-0.0, 0.0, -1.0) },
VertexRes { position: Vec3::new(30.0, 1.0, -1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(-0.0, 0.0, -1.0) },
VertexRes { position: Vec3::new(30.0, -1.0, -1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(-0.0, 0.0, -1.0) },
VertexRes { position: Vec3::new(1.0, 1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(1.0, -1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(0.0, -0.0, 1.0) },
VertexRes { position: Vec3::new(30.0, -1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(0.0, 0.0, 1.0) },
VertexRes { position: Vec3::new(30.0, 1.0, 1.0), colour: Vec3::new(1.0, 0.0, 0.0), normal: Vec3::new(0.0, 0.0, 1.0) }
];
pub const TAGAXES_I: [u8; 90] = [0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16, 17, 18, 16, 18, 19, 20, 21, 22, 20, 22, 23, 24, 25, 26, 24, 26, 27, 28, 29, 30, 28, 30, 31, 32, 33, 34, 32, 34, 35, 36, 37, 38, 36, 38, 39, 40, 41, 42, 40, 42, 43, 44, 45, 46, 44, 46, 47, 48, 49, 50, 48, 50, 51, 52, 53, 54, 52, 54, 55, 56, 57, 58, 56, 58, 59];
