
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

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;

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
