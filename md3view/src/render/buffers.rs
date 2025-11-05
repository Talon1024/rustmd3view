use glow::{Context, HasContext};
use std::{marker::PhantomData, sync::Arc, iter};
use glam::Vec2;
use crate::{md3::MD3Surface, render::{traits::IndexInteger, vertex_classes::VertexMD3}};
use bytemuck::Pod;

#[derive(Debug)]
pub struct VertexBuffer {
    pub(crate) glc: Arc<Context>,
    pub(crate) vao: <Context as HasContext>::VertexArray,
    pub(crate) vbo: <Context as HasContext>::Buffer,
    // size: i32,
}

impl VertexBuffer {
    pub fn from_surface(surf: &MD3Surface) -> Vec<VertexMD3> {
        let barycenter = iter::repeat([Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)]);
        let bary_iter = barycenter.flatten();
        surf.texcoords
            .iter()
            .zip(bary_iter)
            .enumerate()
            .map(|(index, (uv, barycenter))| VertexMD3 { index: index as u32, uv: uv.0, barycenter })
            .collect()
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

#[derive(Debug)]
pub struct IndexBuffer<I>
where
    I: IndexInteger + Pod,
{
    pub(crate) glc: Arc<Context>,
    pub(crate) ebo: <Context as HasContext>::Buffer,
    pub(crate) size: i32,
    // Used to access OpenGL constant for the index data type (GL_TYPE)
    itype: PhantomData<I>,
}

impl<I> IndexBuffer<I>
where
    I: IndexInteger + Pod,
{
    pub fn new(glc: Arc<Context>, buf: Vec<I>) -> Self {
        let ebo = unsafe {
            let ebo = glc.create_buffer().unwrap();
            glc.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            glc.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&buf),
                glow::STATIC_DRAW,
            );
            glc.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            ebo
        };
        let size = buf.len() as i32;
        Self { glc, ebo, size, itype: PhantomData }
    }
}

impl IndexBuffer<u32> {
    pub fn from_surface(surf: &MD3Surface) -> Vec<u32> {
        surf.triangles.iter().flat_map(|t| t.0).collect()
    }
}

impl<I> Drop for IndexBuffer<I>
where
    I: IndexInteger + Pod,
{
    fn drop(&mut self) {
        #[cfg(feature = "log_drop_gl_resources")]
        println!("Drop IndexBuffer");
        let glc = &self.glc;
        unsafe {
            glc.delete_buffer(self.ebo);
        }
    }
}
