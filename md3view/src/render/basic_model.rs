use std::rc::Rc;
use anyhow::{Error as AError};
use bytemuck::Pod;
use glow::{Context, HasContext};
use crate::{err_util::GLError, render::{buffers::{IndexBuffer, VertexBuffer}, shader::ShaderProgram, traits::{IndexInteger, ShaderUniformLocations, ShaderUniforms}}};

pub struct BasicModel<I, U, L>
where
    I: IndexInteger + Pod,
    U: ShaderUniforms<L>,
    L: ShaderUniformLocations + Default,
{
    pub vertex: VertexBuffer,
    pub index: IndexBuffer<I>,
    pub shader: Rc<ShaderProgram<L>>,
    pub uniforms: U,
}

impl<I, U, L> BasicModel<I, U, L>
where
    I: IndexInteger + Pod,
    U: ShaderUniforms<L>,
    L: ShaderUniformLocations + Default,
{
    pub fn render<F>(
        &mut self,
        glc: &Context,
        modify_uniforms: F,
    ) -> Result<(), AError>
    where
        F: Fn(&mut U) -> (),
    {
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
