use nalgebra::{Point3, Rotation3, Vector3, Isometry3, UnitQuaternion};

#[derive(Debug, Clone)]
pub struct Camera {
    position: Point3<f32>,
    rotation: (f32, f32, f32),
    fov: f32,
}

impl Camera {
    pub fn position(&self) -> &Point3<f32> {
        &self.position
    }
    pub fn rotation(&self) -> &(f32, f32, f32) {
        &self.rotation
    }
    pub fn fov(&self) -> f32 {
        self.fov
    }
}

impl Camera {
    pub fn new(
        position: Point3<f32>,
        rotation: (f32, f32, f32),
        fov: f32,
    ) -> Self {
        Camera { position, rotation, fov }
    }
    pub fn from_origin() -> Self {
        Camera::new(
            Point3::new(0.0, 0.0, 0.0),
            (0.0, 0.0, 0.0),
            60.0f32.to_radians(),
        )
    }
    pub fn with_fov(mut self, fov: f32) -> Self {
        self.fov = fov;
        self
    }
    pub fn move_at(self, x: f32, y: f32, z: f32) -> Self {
        let vector_to_move =
            Rotation3::from_euler_angles(self.rotation.0, self.rotation.1, self.rotation.2)
                .transform_vector(&Vector3::new(x, y, z));
        Camera::new(self.position + vector_to_move, self.rotation, self.fov)
    }
    pub fn rotate(self, roll: f32, pitch: f32, yaw: f32) -> Self {
        let (x, y, z) = self.rotation;
        Camera::new(self.position, (x + roll, y + pitch, z + yaw), self.fov)
    }
    pub fn into_uniform(self) -> CameraUniform {
        CameraUniform::from(self)
    }
}

#[derive(crevice::std140::AsStd140)]
pub struct CameraUniform {
    camera_to_world: mint::ColumnMatrix4<f32>,
    fov: f32,
}

impl From<Camera> for CameraUniform {
    fn from(cam: Camera) -> Self {
        Self {
            camera_to_world: Isometry3::from_parts(
                cam.position.coords.into(),
                UnitQuaternion::from_euler_angles(cam.rotation.0, cam.rotation.1, cam.rotation.2)
            ).to_matrix().into(),
            fov: cam.fov,
        }
    }
}
