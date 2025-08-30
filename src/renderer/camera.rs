use cgmath::{Matrix4, Deg, Point3, Vector3, perspective, InnerSpace, SquareMatrix};

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            position: Point3::new(0.0, 1.0, 2.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::unit_y(),
            aspect: width as f32 / height as f32,
            fov_y: 45.0,
            z_near: 0.1,
            z_far: 100.0,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        // View matrix looks from position to target
        let view = Matrix4::look_at_rh(self.position, self.target, self.up);

        // Projection matrix with perspective projection
        let proj = perspective(
            Deg(self.fov_y),
            self.aspect,
            self.z_near,
            self.z_far,
        );

        // Combine for view-projection matrix
        proj * view
    }

    pub fn update_aspect(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn move_forward(&mut self, amount: f32) {
        let forward = (self.target - self.position).normalize();
        self.position += forward * amount;
        self.target += forward * amount;
    }

    pub fn move_right(&mut self, amount: f32) {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(self.up).normalize();
        self.position += right * amount;
        self.target += right * amount;
    }

    pub fn move_up(&mut self, amount: f32) {
        self.position += self.up * amount;
        self.target += self.up * amount;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
