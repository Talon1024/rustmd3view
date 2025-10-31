
// Expected input:
//
// #[derive(Debug, Clone)]
// pub struct UniformsMD3 {
//     pub gzdoom: bool,
//     pub anim: Rc<Texture>,
//     pub eye: Mat4,
//     pub frame: f32,
//     pub mode: u32,
//     pub tex: Rc<Texture>,
//     pub rows_per_frame: i32,
// }
//
// Expected output:
//
// #[derive(Debug, Clone, Default)]
// pub struct UniformsMD3Locations {
//     gzdoom: Option<GLUniformLocation>,
//     anim: Option<GLUniformLocation>,
//     eye: Option<GLUniformLocation>,
//     frame: Option<GLUniformLocation>,
//     mode: Option<GLUniformLocation>,
//     tex: Option<GLUniformLocation>,
//     rows_per_frame: Option<GLUniformLocation>,
// }
//
// impl ShaderUniformLocations for UniformsMD3Locations {
//     fn get(
//         glc: &Context,
//         program: <Context as HasContext>::Program,
//     ) -> Self {
//         unsafe {
//             let gzdoom = glc.get_uniform_location(program, "gzdoom");
//             let anim = glc.get_uniform_location(program, "anim");
//             let eye = glc.get_uniform_location(program, "eye");
//             let frame = glc.get_uniform_location(program, "frame");
//             let mode = glc.get_uniform_location(program, "mode");
//             let tex = glc.get_uniform_location(program, "tex");
//             let rows_per_frame =
//                 glc.get_uniform_location(program, "rowsPerFrame");
//             UniformsMD3Locations {
//                 gzdoom,
//                 anim,
//                 eye,
//                 frame,
//                 mode,
//                 tex,
//                 rowsPerFrame: rows_per_frame,
//             }
//         }
//     }
// }
//
// impl ShaderUniforms<UniformsMD3Locations> for UniformsMD3 {
//     fn set(&self, glc: &Context, locations: &UniformsMD3Locations) -> () {
//         let mut texture = TextureUnit::default();
//         unsafe {
//             self.gzdoom.set_uniform(glc, locations.gzdoom.as_ref(), ());
//             self.anim.set_uniform(glc, locations.anim.as_ref(), texture);
//             texture.next();
//             self.eye.set_uniform(glc, locations.eye.as_ref(), ());
//             self.frame.set_uniform(glc, locations.frame.as_ref(), ());
//             self.mode.set_uniform(glc, locations.mode.as_ref(), ());
//             self.tex.set_uniform(glc, locations.tex.as_ref(), texture);
//             texture.next();
//             self.rowsPerFrame.set_uniform(glc, locations.rowsPerFrame.as_ref(), ());
//         }
//     }
// }

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;

pub fn shader_uniforms(v: TokenStream) -> TokenStream {
    let _input = parse_macro_input!(v as DeriveInput);
    quote! { compile_error!("ShaderUniforms derive macro is still WIP!") }.into()
    /*
    quote! {
        #[derive(Debug, Clone, Default)]
        pub struct #locations_struct {
            #uniform_location_fields
        }

        impl ShaderUniformLocations for #locations_struct {
            fn get(
                glc: &Context,
                program: <Context as HasContext>::Program,
            ) -> Self {
                unsafe {
                    #retrievals
                    #locations_struct {
                        #assignments
                    }
                }
            }
        }

        impl ShaderUniforms<#locations_struct> for #original_struct {
            fn set(&self, glc: &Context, locations: &#locations_struct) -> () {
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
    }.into()
    */
}