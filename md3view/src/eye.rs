use glam::{Mat4, Vec3};

pub trait Camera {
    fn view_projection(&self) -> Mat4;
}

#[derive(Debug, Clone, Copy)]
pub struct OrbitCamera {
    // x = longitude (horizontal), y = latitude (vertical), z = distance
    pub longtude: f32,
    pub latitude: f32,
    pub distance: f32,
    pub fov: f32,
    pub aspect: f32,
}

impl OrbitCamera {
    pub fn position(&self, distance: Option<f32>) -> Vec3 {
        Vec3::new(
            self.longtude.cos() * self.latitude.cos(),
            self.longtude.sin() * self.latitude.cos(),
            self.latitude.sin(),
        ) * -distance.unwrap_or(self.distance)
    }
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            longtude: 0.,
            latitude: 0.,
            distance: 0.,
            fov: 80f32.to_radians(),
            aspect: 1.,
        }
    }
}

impl Camera for OrbitCamera {
    fn view_projection(&self) -> Mat4 {
        let eye = self.position(None);
        let view = Mat4::look_at_lh(eye, Vec3::ZERO, Vec3::Z);
        let proj = Mat4::perspective_lh(self.fov, self.aspect, 0.25, 4096.);
        proj * view
    }
}
