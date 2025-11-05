use glam::{Vec2, Vec3};
use glow::{Context, HasContext};
use crate::render::{buffers::VertexBuffer, traits::{SeparateVertexAttributes, VertexAttribute}};
use gl_macros::SeparateVertexAttributes;
use std::{sync::Arc, mem};

#[cfg(feature = "log_conversion_time")]
use std::time::Instant;

#[derive(Debug, Clone, Copy, Default, SeparateVertexAttributes)]
pub struct VertexMD3 {
    pub index: u32,
    pub uv: Vec2,
    pub barycenter: Vec2,
}

#[derive(Debug, Clone, Copy, Default, SeparateVertexAttributes)]
pub struct VertexSprite {
    pub position: Vec2,
    pub size: Vec2,
}

#[derive(Debug, Clone, Copy, Default, SeparateVertexAttributes)]
pub struct VertexRes {
    pub position: Vec3,
    pub colour: Vec3,
    pub normal: Vec3,
}
