
// Expected input:
//
// pub struct VertexMD3 {
//     index: u32,
//     uv: Vec2,
// }
//
// Expected output:
//
// impl SeparateVertexAttributes for VertexMD3 {
//     unsafe fn setup_vertex_attrs(glc: Arc<Context>, data: &[Self]) -> VertexBuffer {
//         let mut attrib_index = 0;
//         let mut offset = 0;
//         // Needs to be i32 so it can be passed to OpenGL functions without the
//         // hassle that comes with conversions between types.
//         let count = i32::try_from(data.len()).unwrap();
// 
//         let strides = [mem::size_of::<u32>(), mem::size_of::<Vec2>()];
//         let total_bytes = strides.iter().sum::<usize>() * data.len();
// 
//         let mut attr_continuous: Vec<u8> = Vec::with_capacity(total_bytes);
//         #[cfg(feature = "log_conversion_time")]
//         let timer_start = Instant::now();
//         unsafe {
//             let uv_start = strides[0] * data.len();
//             let allocated = attr_continuous.spare_capacity_mut();
//             let (mut index_slice, mut uv_slice) = allocated.split_at_mut(uv_start);
//             data.iter().for_each(|datum| {
//                 datum.index.write_to_buf(&mut index_slice);
//                 datum.uv.write_to_buf(&mut uv_slice);
//             });
//             // write_to_buf uses bytes::BufMut, which shortens the slice.
//             assert_eq!(index_slice.len(), 0);
//             assert_eq!(uv_slice.len(), 0);
//             attr_continuous.set_len(total_bytes);
//         }
//         #[cfg(feature = "log_conversion_time")]
//         {
//         let convert_time = timer_start.elapsed().as_micros();
//         println!("Conversion to batched took {} microseconds.", convert_time);
//         }
// 
//         let strides = strides.map(|stride| i32::try_from(stride).unwrap());
//         let vbo = glc.create_buffer().unwrap();
//         glc.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
// 
//         glc.buffer_data_u8_slice(
//             glow::ARRAY_BUFFER,
//             &attr_continuous,
//             glow::STATIC_DRAW,
//         );
// 
//         let vao = glc.create_vertex_array().unwrap();
//         glc.bind_vertex_array(Some(vao));
// 
//         let stride = strides[0];
//         u32::enable(&glc, attrib_index, stride, offset);
//         attrib_index += 1;
//         offset += stride * count;
// 
//         let stride = strides[1];
//         Vec2::enable(&glc, attrib_index, stride, offset);
//         // attrib_index += 1;
//         // offset += stride * count;
// 
//         glc.bind_vertex_array(None);
//         glc.bind_buffer(glow::ARRAY_BUFFER, None);
// 
//         VertexBuffer {
//             glc,
//             vao,
//             vbo,
//         }
//     }
// }

const ALPHABET: &'static str = "abcdefghijklmnopqrstuvwxyz";

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, Span};
use syn::{parse_macro_input, DeriveInput};
use quote::{quote, format_ident, ToTokens};

pub fn separate_vertex_attributes(v: TokenStream) -> TokenStream {
    let input = parse_macro_input!(v as DeriveInput);
    let type_name = input.ident;
    // let mut fields = vec![];
    let fields: Result<Vec<syn::Field>, TokenStream2> = match input.data {
        syn::Data::Struct(data_struct) => {
            match data_struct.fields {
                syn::Fields::Named(fields_named) => {
                    Ok(fields_named.named
                        .into_pairs()
                        .map(|pair| pair.into_value())
                        .collect())
                },
                syn::Fields::Unnamed(fields_unnamed) => {
                    Ok(fields_unnamed.unnamed
                        .into_pairs()
                        .map(|pair| pair.into_value())
                        .collect())
                },
                syn::Fields::Unit => Err(quote! {
                    compile_error!("Marker structs are not supported")
                }),
            }
        },
        syn::Data::Enum(_data_enum) => Err(quote! {
            compile_error!("Enums are not supported")
        }),
        syn::Data::Union(_data_union) => Err(quote! {
            compile_error!("Unions are not supported")
        }),
    };
    let fields = match fields {
        Ok(fields) => fields,
        Err(error) => {
            return error.into();
        }
    };
    let strides = {
        let strides: Vec<TokenStream2> = fields.iter().map(|field| {
            let fty = &field.ty;
            quote! { mem::size_of::<#fty>() }
        }).collect();
        quote! { let strides = [ #(#strides),* ]; }
    };
    let conversion = {
        let field_names = fields.iter().enumerate().map(|(index, field)| {
            field.ident.clone().unwrap_or_else(|| {
                format_ident!("field{index}")
            })
        }).collect::<Vec<_>>();

        /////////////////////////////////////////////////////////////////////

        let field_byte_lengths: TokenStream2 = field_names.iter()
            .enumerate()
            .map(|(index, name)| {
            let name = format_ident!("{name}_byte_len");
            quote! { let #name = strides[#index] * data.len(); }
        }).collect();

        /////////////////////////////////////////////////////////////////////

        let field_splits: TokenStream2 = match fields.len() {
            1 => {
                let slice_name = field_names.first().unwrap();
                quote! {
                    let mut #slice_name = allocated;
                }
            },
            _ => {
                let first_field_name = field_names.first().unwrap();
                let last_field_name = field_names.last().unwrap();
                field_names.windows(2)
                    .map(|field_pair| {
                    let rest_slice_ident = syn::Ident::new("rest_slice", Span::call_site());
                    let [a_name, b_name] = field_pair else {
                        return quote! { compile_error!("What?!") };
                    };
                    let slice_to_split = if a_name == first_field_name {
                        syn::Ident::new("allocated", Span::call_site())
                    } else {
                        rest_slice_ident.clone()
                    };
                    let start_name = format_ident!("{a_name}_byte_len");
                    let a_slice_name = format_ident!("{a_name}_slice");
                    let b_slice_name = if b_name == last_field_name {
                        format_ident!("{b_name}_slice")
                    } else {
                        rest_slice_ident.clone()
                    };
                    quote! {
                        let (mut #a_slice_name, mut #b_slice_name) = #slice_to_split.split_at_mut(#start_name);
                    }
                }).collect()
            }
        };

        /////////////////////////////////////////////////////////////////////

        let field_writes: TokenStream2 = fields.iter()
            .zip(field_names.iter())
            .enumerate()
            .map(|(index, (field, name))| {
            let slice_name = format_ident!("{name}_slice");
            let accessor: TokenStream2 = field.ident.as_ref()
                .map(<syn::Ident as ToTokens>::to_token_stream)
                .unwrap_or_else(|| {
                syn::Index::from(index).to_token_stream()
            });
            quote! { datum.#accessor.write_to_buf(&mut #slice_name); }
        }).collect();

        /////////////////////////////////////////////////////////////////////

        let asserts: TokenStream2 = field_names.iter()
            .map(|name| {
                let slice_name = format_ident!("{name}_slice");
                quote! { assert_eq!(#slice_name.len(), 0); }
            }).collect();


        /////////////////////////////////////////////////////////////////////

        quote! {
            let mut attr_continuous: Vec<u8> = Vec::with_capacity(total_bytes);
            #[cfg(feature = "log_conversion_time")]
            let timer_start = Instant::now();
            unsafe {
                let allocated = attr_continuous.spare_capacity_mut();
                #field_byte_lengths
                #field_splits
                data.iter().for_each(|datum| {
                    #field_writes
                });
                // write_to_buf uses bytes::BufMut, which shortens the slice.
                #asserts
                attr_continuous.set_len(total_bytes);
            }
            #[cfg(feature = "log_conversion_time")]
            {
            let convert_time = timer_start.elapsed().as_micros();
            println!("Conversion to batched took {} microseconds.", convert_time);
            }
        }
    };
    let attributes = {
        let num_fields = fields.len();
        fields.iter().enumerate().map(|(index, field)| {
            let ty = &field.ty;
            let mut attribute = quote! {
                let stride = strides[#index];
                #ty::enable(&glc, attrib_index, stride, offset);
            };
            if index != num_fields {
                attribute.extend(quote! {
                    attrib_index += 1;
                    offset += stride * count;
                });
            }
            attribute
        }).collect::<proc_macro2::TokenStream>()
    };
    quote! {
        impl SeparateVertexAttributes for #type_name {
            unsafe fn setup_vertex_attrs(glc: Arc<Context>, data: &[Self]) -> VertexBuffer {
                let mut attrib_index = 0;
                let mut offset = 0;
                // Needs to be i32 so it can be passed to OpenGL functions without the
                // hassle that comes with conversions between types.
                let count = i32::try_from(data.len()).unwrap();

                #strides
                let total_bytes = strides.iter().sum::<usize>() * data.len();

                #conversion

                let strides = strides.map(|stride| i32::try_from(stride).unwrap());
                let vbo = glc.create_buffer().unwrap();
                glc.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

                glc.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    &attr_continuous,
                    glow::STATIC_DRAW,
                );

                let vao = glc.create_vertex_array().unwrap();
                glc.bind_vertex_array(Some(vao));

                #attributes

                glc.bind_vertex_array(None);
                glc.bind_buffer(glow::ARRAY_BUFFER, None);

                VertexBuffer {
                    glc,
                    vao,
                    vbo,
                }
            }
        }
    }.into()
}
