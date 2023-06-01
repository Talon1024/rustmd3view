use glam::{Vec2, Vec3, Mat4};
use bytemuck::{Pod, Zeroable};
use winit::dpi::LogicalSize;

use crate::md3::MD3Frame;

#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct ScreenSize {
    pub width: f32,
    pub height: f32
}

impl From<LogicalSize<f32>> for ScreenSize {
    fn from(value: LogicalSize<f32>) -> Self {
        ScreenSize { width: value.width, height: value.height }
    }
}

impl ScreenSize {
    pub fn aspect_ratio(&self) -> f32 {
        self.width / self.height
    }
}

impl From<ScreenSize> for [f32; 2] {
    fn from(value: ScreenSize) -> Self {
        [value.width, value.height]
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl From<MD3Frame> for BoundingBox {
    fn from(MD3Frame { min, max, .. }: MD3Frame) -> Self {
        BoundingBox { min, max }
    }
}

impl BoundingBox {
    pub fn to_lines(&self, camera: Mat4) -> Vec<Vec2> {
        let bbox_points: [Vec3; 8] = [
            self.min,
            Vec3 { x: self.max.x, y: self.min.y, z: self.min.z },
            Vec3 { x: self.min.x, y: self.max.y, z: self.min.z },
            Vec3 { x: self.min.x, y: self.min.y, z: self.max.z },
            Vec3 { x: self.min.x, y: self.max.y, z: self.max.z },
            Vec3 { x: self.max.x, y: self.min.y, z: self.max.z },
            Vec3 { x: self.max.x, y: self.max.y, z: self.min.z },
            self.max,
        ];
        let closest = bbox_points.iter()
            .map(|pt| camera.project_point3(*pt))
            .reduce(|a, b| {
                let dast = a.length();
                let dist = b.length();
                let closer = f32::min(dast, dist);
                match (closer == dast, closer == dist) {
                    (true, false) => a,
                    (false, true) => b,
                    _ => a
                }
            }).unwrap();
        // 9 lines are needed
        vec![]
    }
}
