use std::rc::Rc;
use glam::Mat4;
use glow::{Context, HasContext};
use crate::render::{texture::Texture, texture_unit::TextureUnit, traits::{GLUniformLocation, ShaderUniformLocations, ShaderUniforms, Uniform}};


// TODO: Macro-ize!
#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub struct UniformsMD3 {
    pub gzdoom: bool,
    pub anim: Rc<Texture>,
    pub eye: Mat4,
    pub frame: f32,
    pub mode: u32,
    pub tex: Rc<Texture>,
    pub rowsPerFrame: i32,
}

// TODO: Macro-ize!
#[derive(Debug, Clone, Default)]
pub struct UniformsRes {
    pub eye: Mat4,
    pub shaded: bool,
}

// Derive output for UniformsMD3

#[allow(non_snake_case)]
#[derive(Debug, Clone, Default)]
pub struct UniformsMD3Locations {
    gzdoom: Option<GLUniformLocation>,
    anim: Option<GLUniformLocation>,
    eye: Option<GLUniformLocation>,
    frame: Option<GLUniformLocation>,
    mode: Option<GLUniformLocation>,
    tex: Option<GLUniformLocation>,
    rowsPerFrame: Option<GLUniformLocation>,
}

impl ShaderUniformLocations for UniformsMD3Locations {
    fn get(
        glc: &Context,
        program: <Context as HasContext>::Program,
    ) -> Self {
        unsafe {
            let gzdoom = glc.get_uniform_location(program, "gzdoom");
            let anim = glc.get_uniform_location(program, "anim");
            let eye = glc.get_uniform_location(program, "eye");
            let frame = glc.get_uniform_location(program, "frame");
            let mode = glc.get_uniform_location(program, "mode");
            let tex = glc.get_uniform_location(program, "tex");
            let rows_per_frame =
                glc.get_uniform_location(program, "rowsPerFrame");
            UniformsMD3Locations {
                gzdoom,
                anim,
                eye,
                frame,
                mode,
                tex,
                rowsPerFrame: rows_per_frame,
            }
        }
    }
}

impl ShaderUniforms<UniformsMD3Locations> for UniformsMD3 {
    fn set(&self, glc: &Context, locations: &UniformsMD3Locations) -> () {
        let mut texture = TextureUnit::default();
        unsafe {
            self.gzdoom.set_uniform(glc, locations.gzdoom.as_ref(), ());
            self.anim.set_uniform(glc, locations.anim.as_ref(), texture);
            texture.next();
            self.eye.set_uniform(glc, locations.eye.as_ref(), ());
            self.frame.set_uniform(glc, locations.frame.as_ref(), ());
            self.mode.set_uniform(glc, locations.mode.as_ref(), ());
            self.tex.set_uniform(glc, locations.tex.as_ref(), texture);
            texture.next();
            self.rowsPerFrame.set_uniform(glc, locations.rowsPerFrame.as_ref(), ());
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UniformsResLocations {
    eye: Option<GLUniformLocation>,
    shaded: Option<GLUniformLocation>,
}

impl ShaderUniformLocations for UniformsResLocations {
    fn get(
        glc: &Context,
        program: <Context as HasContext>::Program,
    ) -> Self {
        unsafe {
            let eye = glc.get_uniform_location(program, "eye");
            let shaded = glc.get_uniform_location(program, "shaded");
            UniformsResLocations { eye, shaded }
        }
    }
}

impl ShaderUniforms<UniformsResLocations> for UniformsRes {
    fn set(&self, glc: &Context, locations: &UniformsResLocations) -> () {
        let mut _texture = TextureUnit::default();
        unsafe {
            self.eye.set_uniform(glc, locations.eye.as_ref(), ());
            self.shaded.set_uniform(glc, locations.shaded.as_ref(), ());
        }
    }
}