use glam::f32::{Mat3, Vec2, Vec3};
use rayon::iter as riter;
use rayon::prelude::*;
use std::io::{Read, Seek, SeekFrom};
use std::iter;
use thiserror::Error;
use binrw::{BinRead, Endian, VecArgs};

pub const MD3_ID: [u8; 4] = *b"IDP3";
pub const MD3_VERSION: i32 = 15;
const MAX_QPATH: usize = 64;

pub type MD3Name = [u8; MAX_QPATH];

#[derive(Debug, Clone)]
pub struct MD3Model {
    pub _version: i32,
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

#[derive(Debug, Clone, Default, BinRead)]
#[br(little)]
pub struct MD3Frame {
    #[br(map(Vec3::from_array))]
    pub _min: Vec3,
    #[br(map(Vec3::from_array))]
    pub _max: Vec3,
    #[br(map(Vec3::from_array))]
    pub _origin: Vec3,
    pub radius: f32,
    pub _name: [u8; 16],
}

#[derive(Debug, Clone, BinRead)]
#[br(little)]
pub struct MD3FrameTag {
    pub name: MD3Name,
    #[br(map(Vec3::from_array))]
    pub origin: Vec3,
    #[br(map(|m: [f32; 9]| Mat3::from_cols_array(&m)))]
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
    pub _vertices: u32,
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
            (0..frames)
                .into_par_iter()
                .flat_map(|frame| {
                    let start = frame * vertices;
                    let end = start + vertices;
                    let by_slice = self.vertices[start..end]
                        .iter()
                        .map(|vert| vert.to_pixel().map(i32::to_ne_bytes))
                        .chain(iter::repeat([[0; 4]; 4]))
                        .take(pixels_per_frame)
                        .flatten()
                        .flatten()
                        .collect::<Vec<u8>>();
                    #[cfg(feature = "make_animation_is_bugged")]
                    {
                        let start = frame * pixels_per_frame;
                        let end = start + pixels_per_frame;
                        let by_index = (start..end)
                            .into_iter()
                            .flat_map(|vindex| {
                                let vindex = vindex % pixels_per_frame;
                                if vindex < vertices {
                                    let vindex = frame * vertices + vindex;
                                    self.vertices[vindex]
                                        .to_pixel()
                                        .map(i32::to_ne_bytes)
                                } else {
                                    [[0; 4]; 4]
                                }
                            })
                            .flatten()
                            .collect::<Vec<u8>>();
                        assert_eq!(by_slice.len(), by_index.len());
                        assert_eq!(by_slice, by_index);
                    }
                    by_slice
                })
                .collect::<Vec<u8>>()
                .into_boxed_slice()
        } else {
            let extra_count = pixels_per_frame - self.vertices.len();
            let by_slice = self
                .vertices
                .par_iter()
                .map(|vert| vert.to_pixel().map(i32::to_ne_bytes))
                .chain(riter::repeatn([[0; 4]; 4], extra_count))
                .flatten()
                .flatten()
                .collect::<Vec<u8>>()
                .into_boxed_slice();
            #[cfg(feature = "make_animation_is_bugged")]
            {
                let by_index = (0..pixels_per_frame)
                    .into_par_iter()
                    .flat_map(|vindex| {
                        let vindex = vindex % pixels_per_frame;
                        if vindex < vertices {
                            self.vertices[vindex]
                                .to_pixel()
                                .map(i32::to_ne_bytes)
                        } else {
                            [[0; 4]; 4]
                        }
                    })
                    .flatten()
                    .collect::<Vec<u8>>()
                    .into_boxed_slice();
                assert_eq!(by_slice.len(), by_index.len());
                assert_eq!(by_slice, by_index);
            }
            by_slice
        };
        assert_eq!(data.len(), width * height * channels);
        Animation {
            _vertices: vertices as u32,
            frames: frames as u32,
            rows_per_frame: rows_per_frame as u32,
            data,
        }
    }
}

#[derive(Debug, Clone, Copy, BinRead)]
#[br(little)]
pub struct MD3Shader {
    pub name: MD3Name,
    pub _index: u32,
}

#[derive(Debug, Clone, Copy, Default, BinRead)]
#[br(little)]
pub struct MD3Triangle(
    #[br(map(|tri: [u32; 3]| {
        [tri[2], tri[1], tri[0]]
    }))]
    pub [u32; 3]
);

#[derive(Debug, Clone, Copy, Default, BinRead)]
#[br(little)]
pub struct MD3TexCoord(#[br(map(Vec2::from_array))] pub Vec2);

#[derive(Debug, Clone, Copy, Default, BinRead)]
#[br(little)]
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

#[derive(Debug, Error)]
pub enum MD3ReadError {
    #[error("Wrong ID ({0:?} instead of IDP3)!")]
    WrongId([u8; 4]),
    #[error("Unsupported version (version is {0})")]
    UnsupportedVersion(i32),
    #[error("Reached end of file")]
    EOF,
    #[error("Reader is after end position (position is {0})!")]
    AfterEnd(u64),
    #[error("binrw error: {0}")]
    BinRead(binrw::Error)
}

impl From<binrw::Error> for MD3ReadError {
    fn from(value: binrw::Error) -> Self {
        MD3ReadError::BinRead(value)
    }
}

macro_rules! read_data_section {
    ($base_offset: ident, $data_offset: ident, $data: ident, $target: ident.$output: ident, $outtype: ty, $count: ident) => {
    let offset = $base_offset + $data_offset as u64;
    $data.seek(SeekFrom::Start(offset)).or(Err(EOF))?;
    $target.$output = Vec::<$outtype>::read_options(
        $data, Endian::Little, VecArgs {
            count: $count as usize,
            inner: Default::default(),
        }
    )?;
    }
}

// trait ReadStream : Read + Seek {}
type MD3Result<T> = Result<T, MD3ReadError>;

pub fn read_md3(data: &mut (impl Read + Seek)) -> MD3Result<MD3Model> {
    use MD3ReadError::*;
    let mut model = MD3Model {
        _version: MD3_VERSION,
        name: [0; MAX_QPATH],
        num_tags: 0,
        frames: vec![],
        tags: vec![],
        surfaces: vec![],
    };

    // This keeps the code clean with a declarative definition of MD3Header
    #[derive(BinRead)]
    #[br(little)]
    struct MD3Header {
        ident: [u8; 4],
        version: i32,
        name: [u8; MAX_QPATH],
        _flags: u32,
        num_frames: u32,
        num_tags: u32,
        num_surfs: u32,
        _num_skins: u32,
        offset_frames: u32,
        offset_tags: u32,
        offset_surfs: u32,
        offset_end: u32,
    }

    // Read the data
    let MD3Header {
        ident,
        version,
        name,
        _flags,
        num_frames,
        num_tags,
        num_surfs,
        _num_skins,
        offset_frames,
        offset_tags,
        offset_surfs,
        offset_end,
    } = MD3Header::read(data)?;
    let seek_offset = 0u64;

    // Process the data

    if ident != MD3_ID {
        return Err(WrongId(ident));
    }
    if version != MD3_VERSION {
        return Err(UnsupportedVersion(version));
    }

    model.name = name;
    model.num_tags = num_tags as usize;

    // Frames
    read_data_section!(seek_offset, offset_frames, data, model.frames, MD3Frame, num_frames);

    // Tags
    // num_tags in the header is the amount of tags per frame on the model
    // Actual amount of tag data is multiplied by the number of frames
    let num_tags = num_tags * num_frames;
    read_data_section!(seek_offset, offset_tags, data, model.tags, MD3FrameTag, num_tags);

    // Surfaces
    let offset_surfs = seek_offset + offset_surfs as u64;
    data.seek(SeekFrom::Start(offset_surfs)).or(Err(EOF))?;
    // MD3Surface is not BinRead for now
    model.surfaces = (0..num_surfs)
        .map(|_| read_surface(data))
        .collect::<MD3Result<Vec<MD3Surface>>>()?;

    // Ensure not past EOF
    let pos = data.stream_position().or(Err(EOF))?;
    if pos > offset_end as u64 {
        return Err(AfterEnd(pos));
    }
    Ok(model)
}

fn read_surface(data: &mut (impl Read + Seek)) -> MD3Result<MD3Surface> {
    use MD3ReadError::*;
    let mut surface = MD3Surface {
        name: [0; MAX_QPATH],
        num_verts: 0,
        num_frames: 0,
        shaders: vec![],
        triangles: vec![],
        texcoords: vec![],
        vertices: vec![],
    };
    let seek_offset = data.stream_position().or(Err(EOF))?;

    #[derive(BinRead)]
    #[br(little)]
    struct MD3SurfaceHeader {
        ident: [u8; 4],
        name: [u8; MAX_QPATH],
        _flags: u32,
        num_frames: u32,
        num_shaders: u32,
        num_verts: u32,
        num_tris: u32,
        offset_triangles: u32,
        offset_shaders: u32,
        offset_uvs: u32,
        offset_verts: u32,
        offset_end: u32,
    }

    // Read the data
    let MD3SurfaceHeader {
        ident,
        name,
        _flags,
        num_frames,
        num_shaders,
        num_verts,
        num_tris,
        offset_triangles,
        offset_shaders,
        offset_uvs,
        offset_verts,
        offset_end,
    } = MD3SurfaceHeader::read(data)?;

    // Process the data
    if ident != MD3_ID {
        return Err(WrongId(ident));
    }
    surface.name = name;
    // Sizes/counts
    surface.num_frames = num_frames as usize;
    surface.num_verts = num_verts as usize;

    // Shaders
    read_data_section!(seek_offset, offset_shaders, data, surface.shaders, MD3Shader, num_shaders);

    // Triangles
    read_data_section!(seek_offset, offset_triangles, data, surface.triangles, MD3Triangle, num_tris);

    // UVs
    read_data_section!(seek_offset, offset_uvs, data, surface.texcoords, MD3TexCoord, num_verts);

    // Vertices
    let num_verts = surface.num_verts * surface.num_frames;
    read_data_section!(seek_offset, offset_verts, data, surface.vertices, MD3FrameVertex, num_verts);

    let offset_end = seek_offset + offset_end as u64;
    let pos = data.stream_position().or(Err(EOF))?;
    if pos > offset_end {
        return Err(AfterEnd(pos));
    }
    Ok(surface)
}
