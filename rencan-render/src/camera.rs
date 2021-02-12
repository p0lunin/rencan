use nalgebra::{Point3, Rotation3};

#[derive(Debug, Clone)]
pub struct Camera {
    position: Point3<f32>,
    rotation: Rotation3<f32>,
}

impl Camera {
    pub fn position(&self) -> &Point3<f32> {
        &self.position
    }
    pub fn rotation(&self) -> &Rotation3<f32> {
        &self.rotation
    }
}

impl Camera {
    pub fn new(position: Point3<f32>, rotation: Rotation3<f32>) -> Self {
        Camera { position, rotation }
    }
    pub fn from_origin() -> Self {
        Camera::new(Point3::new(0.0, 0.0, 0.0), Rotation3::from_euler_angles(0.0, 0.0, 0.0))
    }
    pub fn into_uniform(self) -> CameraUniform {
        CameraUniform::from(self)
    }
}

#[derive(crevice::std140::AsStd140)]
pub struct CameraUniform {
    position: mint::Vector3<f32>,
    rotation: mint::ColumnMatrix3<f32>,
}

impl From<Camera> for CameraUniform {
    fn from(cam: Camera) -> Self {
        Self {
            position: cam.position.coords.into(),
            rotation: cam.rotation.matrix().clone().into(),
        }
    }
}