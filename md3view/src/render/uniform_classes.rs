use std::rc::Rc;
use glam::Mat4;
use glow::{Context, HasContext};
use crate::render::{texture::Texture, texture_unit::TextureUnit, traits::{GLUniformLocation, ShaderUniformLocations, ShaderUniforms, Uniform}};
use gl_macros::ShaderUniformLocations;

#[derive(Debug, Clone, ShaderUniformLocations)]
pub struct UniformsMD3 {
    pub gzdoom: bool,
    pub anim: Rc<Texture>,
    pub eye: Mat4,
    pub frame: f32,
    pub mode: u32,
    pub tex: Rc<Texture>,
    pub rows_per_frame: i32,
}

#[derive(Debug, Clone, Default, ShaderUniformLocations)]
pub struct UniformsRes {
    pub eye: Mat4,
    pub shaded: bool,
}

impl ShaderUniforms<UniformsMD3Locations> for UniformsMD3 {
    fn set(&self, glc: &Context, locations: &UniformsMD3Locations) -> () {
        /*
        let gzdoom_extra_data: <bool as Uniform>::ExtraData = Default::default();
        let anim_extra_data: <Rc<Texture> as Uniform>::ExtraData = Default::default();
        let eye_extra_data: <Mat4 as Uniform>::ExtraData = Default::default();
        let frame_extra_data: <f32 as Uniform>::ExtraData = Default::default();
        let mode_extra_data: <u32 as Uniform>::ExtraData = Default::default();
        let tex_extra_data: <Rc<Texture> as Uniform>::ExtraData = Default::default();
        let rowsPerFrame_extra_data: <i32 as Uniform>::ExtraData = Default::default();
        */
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
            self.rows_per_frame.set_uniform(glc, locations.rows_per_frame.as_ref(), ());
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