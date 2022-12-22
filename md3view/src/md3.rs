use glam::f32::{Vec2, Vec3, Mat3};
use std::io::{Read, Seek, SeekFrom};
use std::iter;
use rayon::iter as riter;
use thiserror::Error;
use rayon::prelude::*;

pub const MD3_ID: [u8; 4] = *b"IDP3";
pub const MD3_VERSION: i32 = 15;

pub type MD3Name = [u8; 64];

#[derive(Debug, Clone)]
pub struct MD3Model {
	pub version: i32,
	pub name: MD3Name,
	pub num_tags: usize,
	pub frames: Vec<MD3Frame>,
	pub tags: Vec<MD3FrameTag>,
	pub surfaces: Vec<MD3Surface>,
}

impl MD3Model {
	pub fn max_radius(&self) -> f32 {
		self.frames.iter().map(|f| f.radius).reduce(f32::max).unwrap_or(0.)
	}
}

#[derive(Debug, Clone, Default)]
pub struct MD3Frame {
	pub min: Vec3,
	pub max: Vec3,
	pub origin: Vec3,
	pub radius: f32,
	pub name: [u8; 16],
}

#[derive(Debug, Clone)]
pub struct MD3FrameTag {
	pub name: MD3Name,
	pub origin: Vec3,
	pub axes: Mat3,
}

#[derive(Debug, Clone)]
pub struct MD3Surface {
	pub name: MD3Name,
	pub num_verts: usize,
	pub num_frames: usize,
	pub shaders: Vec<MD3Shader>,
	pub triangles: Vec<MD3Triangle>,
	pub texcoords: Vec<MD3TexCoord>,
	pub vertices: Vec<MD3FrameVertex>,
}

#[derive(Debug, Clone, Default)]
pub struct Animation {
	pub vertices: u32,
	pub frames: u32,
	pub rows_per_frame: u32,
	pub data: Box<[u8]>,
}

impl MD3Surface {
	pub fn make_animation(&self, width: Option<usize>) -> Animation {
		let vertices = self.num_verts;
		let frames = self.num_frames;
		let width = width.unwrap_or(vertices);
		let rows_per_frame = (vertices as f32 / width as f32).ceil() as usize;
		let pixels_per_frame = width * rows_per_frame;
		let height = frames * rows_per_frame;
		// 4 "colour channels" * size_of(i32) bytes
		let channels = 4usize * std::mem::size_of::<i32>();
		let data = if frames > 1 {
			(0..frames).into_par_iter().flat_map(|frame| {
				let start = frame * vertices;
				let end = start + vertices;
				let by_slice = self.vertices[start..end].iter()
					.map(|vert| vert.to_pixel().map(i32::to_ne_bytes))
					.chain(iter::repeat([[0; 4]; 4])).take(pixels_per_frame)
					.flatten().flatten().collect::<Vec<u8>>();
				#[cfg(feature = "make_animation_is_bugged")]
				{
				let start = frame * pixels_per_frame;
				let end = start + pixels_per_frame;
				let by_index = (start..end).into_iter().flat_map(|vindex| {
					let vindex = vindex % pixels_per_frame;
					if vindex < vertices {
						let vindex = frame * vertices + vindex;
						self.vertices[vindex].to_pixel().map(i32::to_ne_bytes)
					} else {
						[[0; 4]; 4]
					}
				}).flatten().collect::<Vec<u8>>();
				assert_eq!(by_slice.len(), by_index.len());
				assert_eq!(by_slice, by_index);
				}
				by_slice
			}).collect::<Vec<u8>>().into_boxed_slice()
		} else {
			let extra_count = pixels_per_frame - self.vertices.len();
			let by_slice = self.vertices.par_iter()
				.map(|vert| vert.to_pixel().map(i32::to_ne_bytes))
				.chain(riter::repeatn([[0; 4]; 4], extra_count))
				.flatten().flatten().collect::<Vec<u8>>().into_boxed_slice();
			#[cfg(feature = "make_animation_is_bugged")]
			{
			let by_index = (0..pixels_per_frame).into_par_iter().flat_map(|vindex| {
				let vindex = vindex % pixels_per_frame;
				if vindex < vertices {
					self.vertices[vindex].to_pixel().map(i32::to_ne_bytes)
				} else {
					[[0; 4]; 4]
				}
			}).flatten().collect::<Vec<u8>>().into_boxed_slice();
			assert_eq!(by_slice.len(), by_index.len());
			assert_eq!(by_slice, by_index);
			}
			by_slice
		};
		assert_eq!(data.len(), width * height * channels);
		Animation {
			vertices: vertices as u32, frames: frames as u32,
			rows_per_frame: rows_per_frame as u32,
			data
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct MD3Shader {
	pub name: MD3Name,
	pub index: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MD3Triangle(pub [u32; 3]);
#[derive(Debug, Clone, Copy, Default)]
pub struct MD3TexCoord(pub Vec2);

#[derive(Debug, Clone, Copy, Default)]
pub struct MD3FrameVertex {
	pub x: i16,
	pub y: i16,
	pub z: i16,
	pub n: u16,
}

impl MD3FrameVertex {
	pub fn to_pixel(&self) -> [i32; 4] {
		[self.x as i32, self.y as i32, self.z as i32, self.n as i32]
	}
}

#[derive(Debug, Clone, Error)]
pub enum MD3ReadError {
	#[error("Wrong ID ({0:?} instead of IDP3)!")]
	WrongId([u8; 4]),
	#[error("Unsupported version (version is {0})")]
	UnsupportedVersion(i32),
	#[error("Reached end of file")]
	EOF,
	#[error("Reader is after end position (position is {0})!")]
	AfterEnd(u64),
}

// trait ReadStream : Read + Seek {}
type MD3Result<T> = Result<T, MD3ReadError>;

pub fn read_md3(data: &mut (impl Read + Seek)) -> MD3Result<MD3Model> {
	use MD3ReadError::*;
	let mut model = MD3Model {
		version: MD3_VERSION,
		name: [0; 64],
		num_tags: 0,
		frames: vec![],
		tags: vec![],
		surfaces: vec![],
	};
	let mut int_buf = [0; 4];
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	if int_buf != MD3_ID { return Err(WrongId(int_buf)); }
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	let version = i32::from_le_bytes(int_buf);
	if version != MD3_VERSION { return Err(UnsupportedVersion(version)); }
	data.read_exact(&mut model.name).or(Err(EOF))?;
	/* data.read_exact(&mut int_buf).or(Err(EOF))?; */
	data.seek(SeekFrom::Current(4)).or(Err(EOF))?;
	/* let flags = i32::from_le_bytes(int_buf); */
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	let num_frames = u32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	let num_tags = u32::from_le_bytes(int_buf);
	model.num_tags = num_tags as usize;
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	let num_surfs = u32::from_le_bytes(int_buf);
	/* data.read_exact(&mut int_buf).or(Err(EOF))?; */
	data.seek(SeekFrom::Current(4)).or(Err(EOF))?;
	/* let num_skins = u32::from_le_bytes(int_buf); */
	// Offsets
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	let offset_frames = u32::from_le_bytes(int_buf) as u64;
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	let offset_tags = u32::from_le_bytes(int_buf) as u64;
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	let offset_surfaces = u32::from_le_bytes(int_buf) as u64;
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	let offset_end = u32::from_le_bytes(int_buf) as u64;
	// Frames
	data.seek(SeekFrom::Start(offset_frames)).or(Err(EOF))?;
	model.frames = (0..num_frames).map(|_| read_frame(data))
		.collect::<MD3Result<Vec<MD3Frame>>>()?;
	// Tags
	{
	data.seek(SeekFrom::Start(offset_tags)).or(Err(EOF))?;
	let num_tags = num_tags * num_frames;
	model.tags = (0..num_tags).map(|_| read_tag(data))
		.collect::<MD3Result<Vec<MD3FrameTag>>>()?;
	}
	// Surfaces
	data.seek(SeekFrom::Start(offset_surfaces)).or(Err(EOF))?;
	model.surfaces = (0..num_surfs).map(|_| read_surface(data))
		.collect::<MD3Result<Vec<MD3Surface>>>()?;
	let pos = data.stream_position().or(Err(EOF))?;
	if pos > offset_end {
		return Err(AfterEnd(pos));
	}
	Ok(model)
}

fn read_frame(data: &mut (impl Read + Seek)) -> MD3Result<MD3Frame> {
	use MD3ReadError::*;
	let mut frame = MD3Frame {
		min: Vec3::ZERO,
		max: Vec3::ZERO,
		origin: Vec3::ZERO,
		radius: 0.,
		name: [0; 16],
	};
	let mut int_buf = [0; 4];
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	frame.min.x = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	frame.min.y = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	frame.min.z = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	frame.max.x = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	frame.max.y = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	frame.max.z = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	frame.origin.x = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	frame.origin.y = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	frame.origin.z = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	frame.radius = f32::from_le_bytes(int_buf);
	data.read_exact(&mut frame.name).or(Err(EOF))?;
	Ok(frame)
}

fn read_tag(data: &mut (impl Read + Seek)) -> MD3Result<MD3FrameTag> {
	use MD3ReadError::*;
	let mut tag = MD3FrameTag {
		name: [0; 64],
		origin: Vec3::ZERO,
		axes: Mat3::ZERO,
	};
	let mut int_buf = [0; 4];
	data.read_exact(&mut tag.name).or(Err(EOF))?;
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	tag.origin.x = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	tag.origin.y = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	tag.origin.z = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	tag.axes.x_axis.x = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	tag.axes.x_axis.y = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	tag.axes.x_axis.z = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	tag.axes.y_axis.x = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	tag.axes.y_axis.y = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	tag.axes.y_axis.z = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	tag.axes.z_axis.x = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	tag.axes.z_axis.y = f32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	tag.axes.z_axis.z = f32::from_le_bytes(int_buf);
	Ok(tag)
}

fn read_surface(data: &mut (impl Read + Seek)) -> MD3Result<MD3Surface> {
	use MD3ReadError::*;
	let mut surface = MD3Surface {
		name: [0; 64],
		num_verts: 0,
		num_frames: 0,
		shaders: vec![],
		triangles: vec![],
		texcoords: vec![],
		vertices: vec![],
	};
	let offset_ref = data.stream_position().or(Err(EOF))?;
	let mut int_buf = [0; 4];
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	if int_buf != MD3_ID {
		return Err(WrongId(int_buf));
	}
	data.read_exact(&mut surface.name).or(Err(EOF))?;
	data.seek(SeekFrom::Current(4)).or(Err(EOF))?; // flags (unused)
	// Sizes/counts
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	surface.num_frames = u32::from_le_bytes(int_buf) as usize;
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	let num_shaders = u32::from_le_bytes(int_buf);
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	surface.num_verts = u32::from_le_bytes(int_buf) as usize;
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	let num_tris = u32::from_le_bytes(int_buf);
	// Offsets
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	let offset_triangles = offset_ref + u32::from_le_bytes(int_buf) as u64;
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	let offset_shaders = offset_ref + u32::from_le_bytes(int_buf) as u64;
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	let offset_uvs = offset_ref + u32::from_le_bytes(int_buf) as u64;
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	let offset_verts = offset_ref + u32::from_le_bytes(int_buf) as u64;
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	let offset_end = offset_ref + u32::from_le_bytes(int_buf) as u64;
	// Shaders
	data.seek(SeekFrom::Start(offset_shaders)).or(Err(EOF))?;
	surface.shaders = (0..num_shaders).map(|_| read_shader(data))
		.collect::<MD3Result<Vec<MD3Shader>>>()?;
	// Triangles
	data.seek(SeekFrom::Start(offset_triangles)).or(Err(EOF))?;
	surface.triangles = (0..num_tris).map(|_| read_triangle(data))
		.collect::<MD3Result<Vec<MD3Triangle>>>()?;
	// UVs
	data.seek(SeekFrom::Start(offset_uvs)).or(Err(EOF))?;
	surface.texcoords = (0..surface.num_verts).map(|_| read_texcoord(data))
		.collect::<MD3Result<Vec<MD3TexCoord>>>()?;
	// Vertices
	{
		let num_verts = surface.num_verts * surface.num_frames;
		data.seek(SeekFrom::Start(offset_verts)).or(Err(EOF))?;
		surface.vertices = (0..num_verts).map(|_| read_vertex(data))
			.collect::<MD3Result<Vec<MD3FrameVertex>>>()?;
	}
	let pos = data.stream_position().or(Err(EOF))?;
	if pos > offset_end {
		return Err(AfterEnd(pos));
	}
	Ok(surface)
}

fn read_shader(data: &mut (impl Read + Seek)) -> MD3Result<MD3Shader> {
	use MD3ReadError::*;
	let mut shader = MD3Shader {
		name: [0; 64],
		index: 0,
	};
	let mut int_buf = [0; 4];
	data.read_exact(&mut shader.name).or(Err(EOF))?;
	data.read_exact(&mut int_buf).or(Err(EOF))?;
	shader.index = u32::from_le_bytes(int_buf);
	Ok(shader)
}

fn read_triangle(data: &mut (impl Read + Seek)) -> MD3Result<MD3Triangle> {
	use MD3ReadError::*;
	let mut triangle = [0; 3];
	let mut int_buf = [0; 4];
	// When `array_try_map` is stabilized...
	// See https://github.com/rust-lang/rust/issues/79711
	/* triangle = triangle.try_map(|_| {
		data.read_exact(&mut int_buf).or(Err(EOF))?;
		Ok(u32::from_le_bytes(int_buf))
	})?; */
	for i in 0..triangle.len() {
		data.read_exact(&mut int_buf).or(Err(EOF))?;
		triangle[i] = u32::from_le_bytes(int_buf);
	}
	let tmp = triangle[0];
	triangle[0] = triangle[2];
	triangle[2] = tmp;
	Ok(MD3Triangle(triangle))
}

fn read_texcoord(data: &mut (impl Read + Seek)) -> MD3Result<MD3TexCoord> {
	use MD3ReadError::*;
	let mut int_buf = [0; 4];
	let mut coords = [0.; 2];
	for i in 0..coords.len() {
		data.read_exact(&mut int_buf).or(Err(EOF))?;
		coords[i] = f32::from_le_bytes(int_buf);
	}
	Ok(MD3TexCoord(Vec2::from(coords)))
}

fn read_vertex(data: &mut (impl Read + Seek)) -> MD3Result<MD3FrameVertex> {
	use MD3ReadError::*;
	let mut short_buf = [0; 2];
	let mut vertex = MD3FrameVertex {
		x: 0,
		y: 0,
		z: 0,
		n: 0,
	};
	data.read_exact(&mut short_buf).or(Err(EOF))?;
	vertex.x = i16::from_le_bytes(short_buf);
	data.read_exact(&mut short_buf).or(Err(EOF))?;
	vertex.y = i16::from_le_bytes(short_buf);
	data.read_exact(&mut short_buf).or(Err(EOF))?;
	vertex.z = i16::from_le_bytes(short_buf);
	data.read_exact(&mut short_buf).or(Err(EOF))?;
	vertex.n = u16::from_le_bytes(short_buf);
	Ok(vertex)
}
