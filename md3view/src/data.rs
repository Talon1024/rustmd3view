use glam::Vec2;
use bytemuck::{Pod, Zeroable};
use winit::dpi::LogicalSize;

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
    pub fn to_array(&self) -> [f32; 2] {
        [self.width, self.height]
    }
}

impl From<ScreenSize> for [f32; 2] {
    fn from(value: ScreenSize) -> Self {
        value.to_array()
    }
}

impl From<ScreenSize> for Vec2 {
    fn from(value: ScreenSize) -> Self {
        Vec2 { x: value.width, y: value.height }
    }
}

/*
#[derive(Debug, Clone, Copy, Default)]
pub struct BoundingBox {
    pub _min: Vec3,
    pub _max: Vec3,
}

impl From<MD3Frame> for BoundingBox {
    fn from(MD3Frame { min, max, .. }: MD3Frame) -> Self {
        BoundingBox { _min: min, _max: max }
    }
}

impl BoundingBox {
    pub fn to_lines(&self, camera: Mat4) -> Vec<Vec2> {
        let bbox_points: [Vec3; 8] = [
            self._min,
            Vec3 { x: self._max.x, y: self._min.y, z: self._min.z },
            Vec3 { x: self._min.x, y: self._max.y, z: self._min.z },
            Vec3 { x: self._min.x, y: self._min.y, z: self._max.z },
            Vec3 { x: self._min.x, y: self._max.y, z: self._max.z },
            Vec3 { x: self._max.x, y: self._min.y, z: self._max.z },
            Vec3 { x: self._max.x, y: self._max.y, z: self._min.z },
            self._max,
        ];
        let _closest = bbox_points.iter()
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
*/
