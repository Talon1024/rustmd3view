use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;

// Expected input:
//
// pub struct VertexMD3 {
//     index: u32,
//     uv: Vec2,
// }
//
// Expected output:
//
// impl InterleavedVertexAttributes for VertexMD3 {
//     unsafe fn setup_vertex_attrs(glc: &Context) {
//         let mut attrib_index = 0;
//         let mut offset = 0;
//         let stride = Self::STRIDE;
//
//         u32::enable(glc, attrib_index, stride, offset);
//         offset += i32::try_from(mem::size_of::<u32>()).unwrap();
//         attrib_index += 1;
//
//         Vec2::enable(glc, attrib_index, stride, offset);
//     }
// }

/*
struct Field {
    name: Option<syn::Ident>,
    ty: syn::Type
}

impl From<syn::Field> for Field {
    fn from(value: syn::Field) -> Self {
        Field { name: value.ident, ty: value.ty }
    }
}
*/

#[proc_macro_derive(InterleavedVertexAttributes)]
pub fn interleaved_vertex_attributes(v: TokenStream) -> TokenStream {
    let input = parse_macro_input!(v as DeriveInput);
    let type_name = input.ident;
    let attributes = match input.data {
        syn::Data::Struct(data_struct) => { 
            attributes_interleaved(data_struct)
        },
        syn::Data::Enum(_data_enum) => quote! {
            compile_error!("Enums are not supported")
        },
        syn::Data::Union(_data_union) => quote! {
            compile_error!("Unions are not supported")
        },
    };
    quote! {
        impl InterleavedVertexAttributes for #type_name {
            unsafe fn setup_vertex_attrs(glc: &Context) {
                let mut attrib_index: u32 = 0;
                let mut offset: i32 = 0;
                let stride = Self::STRIDE;

                #attributes
            }
        }
    }.into()
}

fn attributes_interleaved(data_struct: syn::DataStruct) -> proc_macro2::TokenStream {
    match data_struct.fields {
        syn::Fields::Named(fields_named) => {
            let last_index = fields_named.named.len().max(1) - 1;
            fields_named.named
                .into_pairs()
                .enumerate()
                .map(|(index, pair)| {
                    let field = pair.into_value();
                    single_field_interleaved(field, index == last_index)
                })
                .collect()
        },
        syn::Fields::Unnamed(fields_unnamed) => {
            let last_index = fields_unnamed.unnamed.len().max(1) - 1;
            fields_unnamed.unnamed
                .into_pairs()
                .enumerate()
                .map(|(index, pair)| {
                    let field = pair.into_value();
                    single_field_interleaved(field, index == last_index)
                })
                .collect()
        },
        syn::Fields::Unit => {
            proc_macro2::TokenStream::default()
        },
    }
}

fn single_field_interleaved(field: syn::Field, last: bool) -> proc_macro2::TokenStream {
    let fty = field.ty;
    let mut tokens = quote! {
        <#fty as VertexAttribute>::enable(glc, attrib_index, stride, offset);
    };
    if !last {
        tokens.extend(quote! {
            offset += i32::try_from(mem::size_of::<#fty>).unwrap();
            attrib_index += 1;
        });
    }
    tokens
}


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
//         let count = i32::try_from(data.len()).unwrap();
// 
//         let attr_continuous: Vec<u8> = data.iter()
//             .flat_map(|d| SmallVec::<[u8; 128]>::from_slice(bytemuck::bytes_of(&d.index)))
//             .chain(data.iter().flat_map(|d| SmallVec::<[u8; 128]>::from_slice(bytemuck::bytes_of(&d.uv))))
//             .collect();
// 
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
//         let stride = i32::try_from(mem::size_of::<u32>()).unwrap();
//         u32::enable(&glc, attrib_index, stride, offset);
//         attrib_index += 1;
//         offset += stride * count;
// 
//         let stride = i32::try_from(mem::size_of::<Vec2>()).unwrap();
//         Vec2::enable(&glc, attrib_index, stride, offset);
//         attrib_index += 1;
//         offset += stride * count;
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

#[proc_macro_derive(SeparateVertexAttributes)]
pub fn separate_vertex_attributes(v: TokenStream) -> TokenStream {
    let input = parse_macro_input!(v as DeriveInput);
    let type_name = input.ident;
    let mut fields = vec![];
    let type_error_if_applicable = match input.data {
        syn::Data::Struct(data_struct) => {
            match data_struct.fields {
                syn::Fields::Named(fields_named) => {
                    fields = fields_named.named
                        .into_pairs()
                        .map(|pair| pair.into_value())
                        .collect();
                    proc_macro2::TokenStream::default()
                },
                syn::Fields::Unnamed(fields_unnamed) => {
                    fields = fields_unnamed.unnamed
                        .into_pairs()
                        .map(|pair| pair.into_value())
                        .collect();
                    proc_macro2::TokenStream::default()
                },
                syn::Fields::Unit => proc_macro2::TokenStream::default(),
            }
        },
        syn::Data::Enum(_data_enum) => quote! {
            compile_error!("Enums are not supported");
        },
        syn::Data::Union(_data_union) => quote! {
            compile_error!("Unions are not supported");
        },
    };
    let conversion = {
        if fields.len() == 0 {
            proc_macro2::TokenStream::default()
        } else {
            let mut field_conversions: Vec<proc_macro2::TokenStream> = fields.iter().enumerate().map(|(index, field)| {
                let access = match field.ident {
                    Some(ref id) => quote! { &d.#id },
                    None => {
                        let id = syn::Index::from(index);
                        quote! { &d.#id }
                    }
                };
                quote! { data.iter().flat_map(|d| SmallVec::<[u8; 128]>::from_slice(bytemuck::bytes_of(#access))) }
            })
            .collect();
            let first_field = field_conversions.remove(0);
            let subsequent_fields = field_conversions.into_iter().map(|conv| {
                quote! { .chain(#conv) }
            }).collect::<proc_macro2::TokenStream>();
            quote! {
                let attr_continuous: Vec<u8> = #first_field
                #subsequent_fields
                .collect();
            }
        }
    };
    let attributes = {
        let num_fields = fields.len();
        fields.iter().enumerate().map(|(index, field)| {
            let ty = &field.ty;
            let mut attribute = quote! {
                let stride = i32::try_from(mem::size_of::<#ty>()).unwrap();
                <#ty as VertexAttribute>::enable(&glc, attrib_index, stride, offset);
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
                let mut attrib_index: u32 = 0;
                let mut offset: i32 = 0;
                let count = i32::try_from(data.len()).unwrap();

                #type_error_if_applicable
                #conversion

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

