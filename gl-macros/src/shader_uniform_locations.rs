
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

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::{quote, format_ident};
use heck::ToLowerCamelCase;
use proc_macro2::Literal;

pub fn shader_uniform_locations(v: TokenStream) -> TokenStream {
    // let original = proc_macro2::TokenStream::from(v.clone());
    let input = parse_macro_input!(v as DeriveInput);
    let locations_struct_ident = format_ident!("{}Locations", input.ident);

    let fields = match input.data {
        syn::Data::Struct(data_struct) => {
            Ok(crate::get_fields(&data_struct))
        },
        syn::Data::Enum(_data_enum) => {
            Err(quote!{ compile_error!("Cannot retrieve the fields of an enum") })
        },
        syn::Data::Union(_data_union) => {
            Err(quote!{ compile_error!("Cannot retrieve the fields of a union") })
        },
    };

    let uniform_location_fields = match fields.clone().map(|fields| {
        fields.iter().map(|field| {
            match field.ident {
                Some(ref field_name) => quote!{ #field_name: Option<GLUniformLocation>, },
                None => quote!{ compile_error!("Shader uniform struct fields must be named!") },
            }
        }).collect::<proc_macro2::TokenStream>()
    }) {
        Ok(t) => t,
        Err(e) => e
    };

    let retrievals = match fields.clone().map(|fields| {
        fields.iter().map(|field| {
            match field.ident {
                Some(ref field_name) => {
                    let field_name_glsl = Literal::string(&field_name.to_string().to_lower_camel_case());
                    quote! { let #field_name = glc.get_uniform_location(program, #field_name_glsl); }
                },
                None => quote!{ compile_error!("Shader uniform struct fields must be named!") },
            }
        }).collect::<proc_macro2::TokenStream>()
    }) {
        Ok(t) => t,
        Err(e) => e
    };

    let assignments = match fields.map(|fields| {
        fields.iter().map(|field| {
            match field.ident {
                Some(ref field_name) => {
                    quote! { #field_name, }
                },
                None => quote!{ compile_error!("Shader uniform struct fields must be named!") },
            }
        }).collect::<proc_macro2::TokenStream>()
    }) {
        Ok(t) => t,
        Err(e) => e
    };

    let locations_struct = quote! {
        #[derive(Debug, Clone, Default)]
        pub struct #locations_struct_ident {
            #uniform_location_fields
        }
    };

    quote! {
        #locations_struct

        impl ShaderUniformLocations for #locations_struct_ident {
            fn get(
                glc: &Context,
                program: <Context as HasContext>::Program,
            ) -> Self {
                unsafe {
                    #retrievals
                    #locations_struct_ident {
                        #assignments
                    }
                }
            }
        }
    }.into()
}