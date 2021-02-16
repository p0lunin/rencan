use nalgebra::{Point3, Rotation3, Vector3};

#[derive(Debug, Clone)]
pub struct Camera {
    position: Point3<f32>,
    rotation: Rotation3<f32>,
    x_angle: f32,
    y_angle: f32,
}

impl Camera {
    pub fn position(&self) -> &Point3<f32> {
        &self.position
    }
    pub fn rotation(&self) -> &Rotation3<f32> {
        &self.rotation
    }
    pub fn angles(&self) -> (f32, f32) {
        (self.x_angle, self.y_angle)
    }
}

impl Camera {
    pub fn new(
        position: Point3<f32>,
        rotation: Rotation3<f32>,
        x_angle: f32,
        y_angle: f32,
    ) -> Self {
        Camera { position, rotation, x_angle, y_angle }
    }
    pub fn from_origin() -> Self {
        Camera::new(
            Point3::new(0.0, 0.0, 0.0),
            Rotation3::from_euler_angles(0.0, 0.0, 0.0),
            60.0f32.to_radians(),
            60.0f32.to_radians(),
        )
    }
    pub fn move_at(self, x: f32, y: f32, z: f32) -> Self {
        let vector_to_move = self.rotation.transform_vector(&Vector3::new(x, y, z));
        Camera::new(self.position + vector_to_move, self.rotation, self.x_angle, self.y_angle)
    }
    pub fn rotate(self, roll: f32, pitch: f32, yaw: f32) -> Self {
        let (x, y, z) = self.rotation.euler_angles();
        Camera::new(
            self.position,
            Rotation3::from_euler_angles(x + roll, y + pitch, z + yaw),
            self.x_angle,
            self.y_angle,
        )
    }
    pub fn into_uniform(self) -> CameraUniform {
        CameraUniform::from(self)
    }
}

#[derive(crevice::std140::AsStd140)]
pub struct CameraUniform {
    position: mint::Vector3<f32>,
    rotation: mint::ColumnMatrix3<f32>,
    x_angle: f32,
    y_angle: f32,
}

impl From<Camera> for CameraUniform {
    fn from(cam: Camera) -> Self {
        Self {
            position: cam.position.coords.into(),
            rotation: cam.rotation.matrix().clone().into(),
            x_angle: cam.x_angle,
            y_angle: cam.y_angle,
        }
    }
}
