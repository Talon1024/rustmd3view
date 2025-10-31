use std::sync::Arc;

use anyhow::{Error as AError};
use glam::{Vec2, Vec4};
use glow::{Context, HasContext};
use crate::{data::ScreenSize, err_util::GLError, render::{shader::{ShaderProgram, ShaderProgramBuilder, ShaderStage}, traits::{GLUniformLocation, ShaderUniformLocations, ShaderUniforms}}, res::AppResources};

trait AngleOfVector {
    fn to_angle(&self) -> f32;
}

impl AngleOfVector for Vec2 {
    fn to_angle(&self) -> f32 {
        self.y.atan2(self.x)
    }
}

pub struct ThickLineInstanceInfo {
    pub a: Vec2,
    pub b: Vec2,
    pub colour: Option<Vec4>,
    pub res: Option<ScreenSize>,
}

impl ThickLineInstanceInfo {
    fn validate(self) -> Option<Self> {
        if self.a == self.b {
            None
        } else {
            Some(self)
        }
    }
    pub fn add_to(self, vec: &mut Vec<ThickLineInstanceRaw>) {
        if let Some(line) = self.validate() {
            vec.push(ThickLineInstanceRaw::from(line));
        }
    }
}

impl From<ThickLineInstanceInfo> for ThickLineInstanceRaw {
    fn from(ThickLineInstanceInfo {
        a, b, colour, res
    }: ThickLineInstanceInfo) -> Self {
        let colour_rgba = colour.unwrap_or(Vec4::ONE);
        let mut a = a;
        let diff = b - a;
        let length_px = diff.length();
        if length_px == 0.0 {
            return ThickLineInstanceRaw::default();
        }
        let angle_rad_ccw = -diff.to_angle();
        if let Some(res) = res {
            let add = Vec2::new(-1., 1.);
            let fac = 2. / Vec2::from(res) * -add;
            a = a.mul_add(fac, add);
            // b = b.mul_add(fac, add);
        }
        ThickLineInstanceRaw::new(
            a, length_px,
            angle_rad_ccw,
            colour_rgba,
        )
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ThickLineInstanceRaw {
    offset_norm_length_px_angle_rad_ccw: [f32; 4],
    colour_rgba: [f32; 4],
}

impl ThickLineInstanceRaw {
    fn new(offset_norm: Vec2, length_px: f32, angle_rad_ccw: f32, colour_rgba: Vec4) -> Self {
        ThickLineInstanceRaw {
            offset_norm_length_px_angle_rad_ccw: [
                offset_norm.x, offset_norm.y,
                length_px, angle_rad_ccw
            ],
            colour_rgba: colour_rgba.to_array()
        }
    }
}

const MAX_LINES_INSTANCES: usize = 128;

#[derive(Debug, Clone)]
pub struct ThickLinesInstanceUniformLocations {
    offset_norm_length_px_angle_rad_ccw: Option<GLUniformLocation>,
    colour_rgb: Option<GLUniformLocation>,
}

#[derive(Debug, Clone)]
pub struct ThickLinesUniformLocations {
    window_resolution: Option<GLUniformLocation>,
    line_instances: [ThickLinesInstanceUniformLocations; MAX_LINES_INSTANCES],
}

impl ShaderUniformLocations for ThickLinesUniformLocations {
    fn get(
        glc: &Context,
        program: <Context as HasContext>::Program,
    ) -> Self {
        unsafe {
            let window_resolution = Some(glc.get_uniform_location(program, "windowResolution").unwrap());
            let array = std::array::from_fn(|i| i);
            let line_instances = array.map(|index| {
                let name = format!("lineInstances[{index}].offset_norm_length_px_angle_rad_ccw");
                let offset_norm_length_px_angle_rad_ccw = glc.get_uniform_location(program, &name);
                let name = format!("lineInstances[{index}].colour_rgb");
                let colour_rgb = glc.get_uniform_location(program, &name);
                ThickLinesInstanceUniformLocations {
                    offset_norm_length_px_angle_rad_ccw, colour_rgb
                }
            });
            ThickLinesUniformLocations { window_resolution, line_instances }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ThickLinesUniforms {
    pub window_resolution: ScreenSize,
}

impl ShaderUniforms<ThickLinesUniformLocations> for ThickLinesUniforms {
    fn set(&self, glc: &Context, locations: &ThickLinesUniformLocations) -> () {
        unsafe {
            glc.uniform_2_f32_slice(locations.window_resolution.as_ref(), &<[f32; 2]>::from(self.window_resolution));
        }
    }
}

pub struct ThickLines {
    glc: Arc<Context>,
    ebo: <Context as HasContext>::Buffer,
    vao: <Context as HasContext>::VertexArray,
    vbo: <Context as HasContext>::Buffer,
    uniforms: ThickLinesUniforms,
    locations: ThickLinesUniformLocations,
    shader: ShaderProgram<ThickLinesUniformLocations>,
    pub instances: Vec<ThickLineInstanceRaw>,
}

impl ThickLines {
    const THICK_LINE_INDEX: [u16; 4] = [3, 0, 2, 1];
    const THICK_LINE_VERTEX: [Vec2; 4] = [
        Vec2 {
            x: 0.0,
            y: 1.0,
        },                  //  0-------3   y=1
        Vec2 {              //  |       |
            x: 0.0,         //  |       |
            y: -1.0,        //  1-------2   y=-1
        },
        Vec2 {
            x: 2.0, // Total NDC range
            y: -1.0,
        },
        Vec2 {
            x: 2.0,
            y: 1.0,
        },
    ];
    pub fn new(glc: Arc<Context>, res: &AppResources) -> Self {
        let stride = std::mem::size_of::<Vec2>() as i32;
        let (ebo, vao, vbo) = unsafe {
            let glc = &glc;
            let vao = glc.create_vertex_array().unwrap();
            glc.bind_vertex_array(Some(vao));
            let vbo = glc.create_buffer().unwrap();
            let ebo = glc.create_buffer().unwrap();
            glc.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            glc.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&Self::THICK_LINE_INDEX),
                glow::STATIC_DRAW,
            );
            glc.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            glc.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&Self::THICK_LINE_VERTEX),
                glow::STATIC_DRAW,
            );
            // T::setup_vertex_attrs(glc);
            glc.vertex_attrib_pointer_f32(
                0, // attrib_index,
                2,
                glow::FLOAT,
                false,
                stride,
                0, // offset,
            );
            glc.enable_vertex_attrib_array(0);
            // end T::setup_vertex_attrs(glc);
            glc.bind_vertex_array(None);
            glc.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            glc.bind_buffer(glow::ARRAY_BUFFER, None);
            (ebo, vao, vbo)
        };
        let shader = ShaderProgramBuilder::new()
            .add_shader(ShaderStage::Vertex, &res.lines_vertex_shader)
            .add_shader(ShaderStage::Fragment, &res.lines_pixel_shader)
            .build(Arc::clone(&glc)).unwrap();
        let uniforms = ThickLinesUniforms::default();
        let locations = ThickLinesUniformLocations::get(&glc, shader.prog);
        Self {
            glc,
            vao,
            vbo,
            ebo,
            instances: Vec::with_capacity(32),
            shader,
            uniforms,
            locations,
        }
    }

    pub fn render<F>(
        &mut self,
        modify_uniforms: F,
    ) -> Result<(), AError>
    where
        F: Fn(&mut ThickLinesUniforms) -> (),
    {
        let glc = &self.glc;
        self.shader.activate()?;
        modify_uniforms(&mut self.uniforms);
        self.instances.chunks(MAX_LINES_INSTANCES).try_for_each(|group| {
            self.uniforms.set(glc, &self.locations);
            group.iter()
                .zip(self.locations.line_instances.as_ref().iter())
                .for_each(|(inst, locations)| {
                unsafe {
                    glc.uniform_4_f32_slice(locations.offset_norm_length_px_angle_rad_ccw.as_ref(), &inst.offset_norm_length_px_angle_rad_ccw);
                    glc.uniform_4_f32_slice(locations.colour_rgb.as_ref(), &inst.colour_rgba);
                }
            });
            unsafe {
                glc.bind_vertex_array(Some(self.vao));
                glc.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
                glc.draw_elements_instanced(
                    glow::TRIANGLE_STRIP, // mode
                    Self::THICK_LINE_INDEX.len() as i32, // count
                    glow::UNSIGNED_SHORT, // type (u16)
                    0, // offset
                    group.len() as i32 // instances
                );
            }
            GLError::get(glc)
        })?;
        self.instances.clear();
        Ok(())
    }
}

impl Drop for ThickLines {
    fn drop(&mut self) {
        unsafe {
            self.glc.delete_vertex_array(self.vao);
            self.glc.delete_buffer(self.ebo);
            self.glc.delete_buffer(self.vbo);
        }
    }
}
