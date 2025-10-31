use glow::{Context, HasContext, UniformLocation};
use std::{rc::Rc, sync::Arc, mem};
use glam::{Vec2, Vec3, Mat4};
use crate::render::{buffers::VertexBuffer, texture::Texture, texture_unit::TextureUnit};
use crate::data::ScreenSize;

pub type GLUniformLocation = <Context as HasContext>::UniformLocation;

pub trait InterleavedVertexAttributes : Sized {
    unsafe fn setup_vertex_attrs(glc: &Context);
    const STRIDE: i32 = mem::size_of::<Self>() as i32;
}

pub trait SeparateVertexAttributes : Sized {
    unsafe fn setup_vertex_attrs(glc: Arc<Context>, data: &[Self]) -> VertexBuffer;
}

pub trait VertexAttribute : Sized {
    unsafe fn enable(glc: &Context, attrib_index: u32, stride: i32, offset: i32);
}

impl VertexAttribute for u32 {
    unsafe fn enable(glc: &Context, attrib_index: u32, stride: i32, offset: i32) {
        glc.vertex_attrib_pointer_i32(
            attrib_index,
            1,
            glow::UNSIGNED_INT,
            stride,
            offset,
        );
        glc.enable_vertex_attrib_array(attrib_index);
    }
}

impl VertexAttribute for Vec2 {
    unsafe fn enable(glc: &Context, attrib_index: u32, stride: i32, offset: i32) {
        glc.vertex_attrib_pointer_f32(
            attrib_index,
            2,
            glow::FLOAT,
            false,
            stride,
            offset,
        );
        glc.enable_vertex_attrib_array(attrib_index);
    }
}

impl VertexAttribute for Vec3 {
    unsafe fn enable(glc: &Context, attrib_index: u32, stride: i32, offset: i32) {
        glc.vertex_attrib_pointer_f32(
            attrib_index,
            3,
            glow::FLOAT,
            false,
            stride,
            offset
        );
        glc.enable_vertex_attrib_array(attrib_index);
    }
}

pub trait IndexInteger {
    const GL_TYPE: u32;
}
impl IndexInteger for u8 {
    const GL_TYPE: u32 = glow::UNSIGNED_BYTE;
}
impl IndexInteger for u16 {
    const GL_TYPE: u32 = glow::UNSIGNED_SHORT;
}
impl IndexInteger for u32 {
    const GL_TYPE: u32 = glow::UNSIGNED_INT;
}

pub trait Uniform {
    type ExtraData; // e.g. Texture unit
    unsafe fn set_uniform(&self, glc: &Context, loc: Option<&UniformLocation>, extra: Self::ExtraData);
}

impl Uniform for Mat4 {
    type ExtraData = ();
    unsafe fn set_uniform(&self, glc: &Context, loc: Option<&UniformLocation>, _extra: Self::ExtraData) {
        glc.uniform_matrix_4_f32_slice(
            loc,
            false,
            &self.to_cols_array(),
        );
    }
}

impl Uniform for Rc<Texture> {
    type ExtraData = TextureUnit;

    unsafe fn set_uniform(&self, glc: &Context, loc: Option<&UniformLocation>, extra: Self::ExtraData) {
        let texture_unit = extra;
        glc.active_texture(texture_unit.slot());
        glc.bind_texture(glow::TEXTURE_2D, Some(self.tex()));
        glc.uniform_1_i32(loc, texture_unit.uniform());
    }
}

impl Uniform for u32 {
    type ExtraData = ();

    unsafe fn set_uniform(&self, glc: &Context, loc: Option<&UniformLocation>, _extra: Self::ExtraData) {
        glc.uniform_1_u32(loc, *self);
    }
}

impl Uniform for bool {
    type ExtraData = ();

    unsafe fn set_uniform(&self, glc: &Context, loc: Option<&UniformLocation>, _extra: Self::ExtraData) {
        glc.uniform_1_u32(loc, *self as u32);
    }
}

impl Uniform for f32 {
    type ExtraData = ();

    unsafe fn set_uniform(&self, glc: &Context, loc: Option<&UniformLocation>, _extra: Self::ExtraData) {
        glc.uniform_1_f32(loc, *self);
    }
}

impl Uniform for i32 {
    type ExtraData = ();

    unsafe fn set_uniform(&self, glc: &Context, loc: Option<&UniformLocation>, _extra: Self::ExtraData) {
        glc.uniform_1_i32(loc, *self);
    }
}

impl Uniform for ScreenSize {
    type ExtraData = ();

    unsafe fn set_uniform(&self, glc: &Context, loc: Option<&UniformLocation>, _extra: Self::ExtraData) {
        // &<[f32; 2]>::from(self.window_resolution)
        glc.uniform_2_f32_slice(loc, &<[f32; 2]>::from(*self));
    }
}

pub trait ShaderUniformLocations {
    fn get(
        glc: &Context,
        program: <Context as HasContext>::Program,
    ) -> Self;
}

pub trait ShaderUniforms<L>
where
    L: ShaderUniformLocations,
{
    fn set(&self, glc: &Context, locations: &L) -> ();
}