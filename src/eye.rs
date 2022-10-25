use glam::{Vec3, Mat4};

pub trait Camera {
	fn view_projection(&self) -> Mat4;
}

#[derive(Debug, Clone, Copy)]
pub struct OrbitCamera {
	// x = longitude (horizontal), y = latitude (vertical), z = distance
	pub position: Vec3,
	pub fov: f32,
	pub aspect: f32,
}

impl Default for OrbitCamera {
	fn default() -> Self {
		Self {
			position: Vec3::ZERO,
			fov: 80f32.to_radians(),
			aspect: 1.,
		}
	}
}

impl Camera for OrbitCamera {
	fn view_projection(&self) -> Mat4 {
		let eye = Vec3::new(
			self.position.x.cos() * self.position.y.cos(),
			self.position.x.sin() * self.position.y.cos(),
			self.position.y.sin(),
		) * self.position.z;
		let view = Mat4::look_at_lh(eye, Vec3::ZERO, Vec3::Z);
		let proj = Mat4::perspective_lh(self.fov, self.aspect, 0.25, 512.);
		proj * view
	}
}
