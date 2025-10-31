use std::{marker::PhantomData, sync::Arc};
use anyhow::{Error as AError};
use glow::{Context, HasContext};
use crate::render::traits::ShaderUniformLocations;

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
where
    L: ShaderUniformLocations,
{
    glc: Arc<Context>,
    pub(crate) prog: <Context as HasContext>::Program,
    // Make sure uniform structs match
    pub(crate) locations: L,
}

impl<L> ShaderProgram<L>
where
    L: ShaderUniformLocations,
{
    pub fn activate(&self) -> Result<(), AError> {
        let glc = &self.glc;
        unsafe {
            glc.use_program(Some(self.prog));
        }
        Ok(())
    }
}

impl<L> Drop for ShaderProgram<L>
where
    L: ShaderUniformLocations,
{
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
where
    L: ShaderUniformLocations,
{
    shaders: Vec<Shader<'a>>,
    location_type: PhantomData<L>,
}

impl<'a, L> ShaderProgramBuilder<'a, L>
where
    L: ShaderUniformLocations,
{
    pub fn new() -> Self {
        Self { shaders: vec![], location_type: PhantomData }
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
                let gl_shader = glc
                    .create_shader(shader.stage.into())
                    .map_err(AError::msg)?;
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
        let locations = L::get(&glc, prog);
        Ok(ShaderProgram { glc, prog, locations })
    }
}
