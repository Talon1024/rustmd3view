use proc_macro::TokenStream;

mod interleaved_vertex_attribute;

#[proc_macro_derive(InterleavedVertexAttributes)]
pub fn interleaved_vertex_attributes(v: TokenStream) -> TokenStream {
    interleaved_vertex_attribute::interleaved_vertex_attributes(v)
}

mod separate_vertex_attribute;

#[proc_macro_derive(SeparateVertexAttributes)]
pub fn separate_vertex_attributes(v: TokenStream) -> TokenStream {
    separate_vertex_attribute::separate_vertex_attributes(v)
}

mod shader_uniforms;

#[proc_macro_derive(ShaderUniforms)]
pub fn shader_uniforms(v: TokenStream) -> TokenStream {
    shader_uniforms::shader_uniforms(v)
}

mod shader_uniform_locations;

#[proc_macro_derive(ShaderUniformLocations)]
pub fn shader_uniform_locations(v: TokenStream) -> TokenStream {
    shader_uniform_locations::shader_uniform_locations(v)
}

pub(crate) fn get_fields(data_struct: &syn::DataStruct) -> Vec<syn::Field> {
    match data_struct.fields {
        syn::Fields::Named(ref fields_named) => {
            fields_named.named.iter().cloned().collect()
        },
        syn::Fields::Unnamed(ref fields_unnamed) => {
            fields_unnamed.unnamed.iter().cloned().collect()
        },
        syn::Fields::Unit => vec![],
    }
}
