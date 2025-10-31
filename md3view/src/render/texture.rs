use std::sync::Arc;
use anyhow::{Error as AError};
use glow::{Context, HasContext};
use crate::{err_util::GLError, md3::MD3Surface, render::MAX_TEXTURE_POT, res::{Surface, SurfaceType}};

#[derive(Debug)]
pub struct Texture {
    glc: Arc<Context>,
    tex: <Context as HasContext>::Texture,
}

impl Drop for Texture {
    fn drop(&mut self) {
        #[cfg(feature = "log_drop_gl_resources")]
        println!("Drop Texture");
        let glc = &self.glc;
        unsafe {
            glc.delete_texture(self.tex);
        }
    }
}

impl Texture {
    pub fn try_from_surface(
        glc: Arc<Context>,
        tex: &Surface,
    ) -> Result<Self, AError> {
        unsafe {
            let texture = glc.create_texture().map_err(AError::msg)?;
            glc.bind_texture(glow::TEXTURE_2D, Some(texture));
            // NOTE: 16-bit images are untested!
            let tex_iformat: i32 = match tex.texture_type {
                SurfaceType::U8RGBA => glow::RGBA32F,
                SurfaceType::U8RGB => glow::RGB32F,
                SurfaceType::U16RGB => glow::RGB32F,
                SurfaceType::U16RGBA => glow::RGBA32F,
                SurfaceType::F32RGB => glow::RGB32F,
                SurfaceType::F32RGBA => glow::RGBA32F,
            }
            .try_into()
            .unwrap();
            let tex_format = match tex.texture_type {
                SurfaceType::U8RGBA => glow::RGBA,
                SurfaceType::U8RGB => glow::RGB,
                SurfaceType::U16RGB => glow::RGB16UI,
                SurfaceType::U16RGBA => glow::RGBA16UI,
                SurfaceType::F32RGB => glow::RGB32F,
                SurfaceType::F32RGBA => glow::RGBA32F,
            };
            let data_type = match tex.texture_type {
                SurfaceType::U8RGBA => glow::UNSIGNED_BYTE,
                SurfaceType::U8RGB => glow::UNSIGNED_BYTE,
                SurfaceType::U16RGB => glow::UNSIGNED_SHORT,
                SurfaceType::U16RGBA => glow::UNSIGNED_SHORT,
                SurfaceType::F32RGB => glow::FLOAT,
                SurfaceType::F32RGBA => glow::FLOAT,
            };
            let min_filter = glow::LINEAR as i32;
            let mag_filter = glow::LINEAR as i32;
            glc.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                tex_iformat,
                tex.width as i32,
                tex.height as i32,
                0,
                tex_format,
                data_type,
                Some(&tex.data),
            );
            GLError::get(&glc)?;
            glc.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::REPEAT as i32,
            );
            glc.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::REPEAT as i32,
            );
            glc.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                min_filter,
            );
            glc.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                mag_filter,
            );
            glc.bind_texture(glow::TEXTURE_2D, None);
            Ok(Texture { tex: texture, glc })
        }
    }
    pub fn try_from_md3(
        glc: Arc<Context>,
        surf: &MD3Surface,
    ) -> Result<(Self, u32), AError> {
        // Animations may need some additional processing
        enum UploadError {
            GLError(GLError),
            Message(String),
            TooBig,
        }
        fn try_upload(
            glc: &Context,
            width: i32,
            height: i32,
            data: &[u8],
        ) -> Result<<Context as HasContext>::Texture, UploadError> {
            let internal_format = glow::RGBA32I as i32;
            let tex_format = glow::RGBA_INTEGER;
            let data_type = glow::INT;
            let target = glow::TEXTURE_2D;
            unsafe {
                let texture =
                    glc.create_texture().map_err(UploadError::Message)?;
                glc.bind_texture(target, Some(texture));
                glc.tex_image_2d(
                    target,
                    0,
                    internal_format,
                    width,
                    height,
                    0,
                    tex_format,
                    data_type,
                    Some(data),
                );
                match GLError::get(&glc) {
                    Ok(_) => Ok(texture),
                    Err(err) => {
                        glc.delete_texture(texture);
                        match err {
                            GLError::InvalidValue => Err(UploadError::TooBig),
                            e => Err(UploadError::GLError(e)),
                        }
                    }
                }
            }
        }
        let mut width = surf.num_verts as i32;
        let mut two_power = (1..MAX_TEXTURE_POT.get().copied().unwrap())
            .rev()
            .filter(|&i| 2i32.pow(i) <= width)
            .next()
            .unwrap_or(0);
        let mut rows_per_frame;
        let tex_handle = loop {
            let an = surf.make_animation(Some(width as usize));
            rows_per_frame = an.rows_per_frame;
            let height = (an.rows_per_frame * an.frames) as i32;
            match try_upload(&glc, width, height, &an.data) {
                Ok(tex) => {
                    break Ok(Texture { glc: Arc::clone(&glc), tex });
                }
                Err(e) => {
                    match e {
                        UploadError::GLError(e) => break Err(AError::from(e)),
                        UploadError::Message(m) => break Err(AError::msg(m)),
                        UploadError::TooBig => {
                            if two_power > 0 {
                                width = 2i32.pow(two_power);
                                two_power -= 1;
                            } else {
                                break Err(AError::msg("Animation is too big to upload to the GPU!"));
                            }
                        }
                    }
                }
            }
        }?;
        let wrapping = glow::REPEAT as i32;
        let filter = glow::NEAREST as i32;
        let target = glow::TEXTURE_2D;
        unsafe {
            glc.tex_parameter_i32(target, glow::TEXTURE_WRAP_S, wrapping);
            glc.tex_parameter_i32(target, glow::TEXTURE_WRAP_T, wrapping);
            glc.tex_parameter_i32(target, glow::TEXTURE_MIN_FILTER, filter);
            glc.tex_parameter_i32(target, glow::TEXTURE_MAG_FILTER, filter);
        }
        Ok((tex_handle, rows_per_frame))
    }
    pub fn tex(&self) -> <Context as HasContext>::Texture {
        self.tex
    }
}
