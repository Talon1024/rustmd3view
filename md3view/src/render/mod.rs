use once_cell::race::OnceBox;

pub mod traits;
pub mod texture_unit;
pub mod vertex_classes;
pub mod uniform_classes;
pub mod texture;
pub mod buffers;
pub mod shader;
pub mod thick_lines;
pub mod basic_model;

pub static MAX_TEXTURE_UNITS: OnceBox<u8> = OnceBox::new();
pub static MAX_TEXTURE_POT: OnceBox<u32> = OnceBox::new();

/*

*/

