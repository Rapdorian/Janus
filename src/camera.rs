pub struct Camera {
    pub eye: ultraviolet::Vec3,
    pub target: ultraviolet::Vec3,
    pub up: ultraviolet::Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(eye: ultraviolet::Vec3, target: ultraviolet::Vec3) -> Self {
        Self {
            eye,
            target,
            up: ultraviolet::Vec3::unit_y(),
            aspect: 16.0 / 9.0,
            fovy: f32::to_radians(59.0),
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn proj(&self) -> ultraviolet::Mat4 {
        let proj = ultraviolet::projection::perspective_infinite_z_wgpu_dx(
            self.fovy,
            self.aspect,
            self.znear,
        );
        let view = ultraviolet::Mat4::look_at(self.eye, self.target, self.up);
        proj * view
    }
}
