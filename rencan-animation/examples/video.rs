use nalgebra::{Point3, Point4, UnitQuaternion, Vector3};
use rencan_animation::{AnimationApp, Renderer};
use rencan_render::core::{
    camera::Camera,
    light::{DirectionLight, LightInfo, PointLight},
    model::SphereModel,
    Scene, Screen,
};
use std::sync::Arc;
use vulkano::device::Device;

mod models {
    use nalgebra::{Point3, UnitQuaternion, Vector3};
    use rencan_render::core::{
        model::{AppModel, Material},
        Model,
    };

    macro_rules! indices {
    ($($x:ident, $y:ident, $z:ident,)*) => {
        vec![
            $([$x as u32, $y as u32, $z as u32, 0].into(),)*
        ]
    };
}

    pub fn make_desk(position: Point3<f32>, scale: f32) -> Vec<AppModel> {
        let mut models = vec![];
        models.push(make_top_desk(position.clone(), scale));
        models.push(make_desk_leg(
            position.clone() + Vector3::new(-0.4 * scale, 0.0, -0.4 * scale),
            scale / 3.0,
        ));
        models.push(make_desk_leg(
            position.clone() + Vector3::new(0.4 * scale, 0.0, -0.4 * scale),
            scale / 3.0,
        ));
        models.push(make_desk_leg(
            position.clone() + Vector3::new(-0.4 * scale, 0.0, 0.4 * scale),
            scale / 3.0,
        ));
        models.push(make_desk_leg(
            position.clone() + Vector3::new(0.4 * scale, 0.0, 0.4 * scale),
            scale / 3.0,
        ));

        models
    }

    fn make_top_desk(position: Point3<f32>, scale: f32) -> AppModel {
        enum Vert {
            A1 = 0,
            B1 = 1,
            C1 = 2,
            D1 = 3,
            A2 = 4,
            B2 = 5,
            C2 = 6,
            D2 = 7,
        }
        use Vert::*;

        let mut model = Model::new(
            vec![
                [-0.4, 0.0, -0.4, 0.0].into(),  // A1 - 0
                [0.4, 0.0, -0.4, 0.0].into(),   // B1 - 1
                [0.4, 0.0, 0.4, 0.0].into(),    // C1 - 2
                [-0.4, 0.0, 0.4, 0.0].into(),   // D1 - 3
                [-0.4, -0.1, -0.4, 0.0].into(), // A2 - 4
                [0.4, -0.1, -0.4, 0.0].into(),  // B2 - 5
                [0.4, -0.1, 0.4, 0.0].into(),   // C2 - 6
                [-0.4, -0.1, 0.4, 0.0].into(),  // D2 - 7
            ],
            indices![
                A1, C1, B1, A1, D1, C1, D2, B2, C2, D2, A2, B2, B2, A1, B1, B2, A2, A1, A2, D1, A1,
                A2, D2, D1, D2, C1, D1, D2, C2, C1, C2, B1, C1, C2, B2, B1,
            ],
        );
        model.position = position;
        model.scaling = scale;

        AppModel::new(model)
    }

    fn make_desk_leg(position: Point3<f32>, scale: f32) -> AppModel {
        enum Vert {
            A1 = 0,
            B1 = 1,
            C1 = 2,
            D1 = 3,
            A2 = 4,
            B2 = 5,
            C2 = 6,
            D2 = 7,
        }
        use Vert::*;

        let mut model = Model::new(
            vec![
                [-0.1, 0.0, -0.1, 0.0].into(),  // A1 - 0
                [0.1, 0.0, -0.1, 0.0].into(),   // B1 - 1
                [0.1, 0.0, 0.1, 0.0].into(),    // C1 - 2
                [-0.1, 0.0, 0.1, 0.0].into(),   // D1 - 3
                [-0.1, -1.0, -0.1, 0.0].into(), // A2 - 4
                [0.1, -1.0, -0.1, 0.0].into(),  // B2 - 5
                [0.1, -1.0, 0.1, 0.0].into(),   // C2 - 6
                [-0.1, -1.0, 0.1, 0.0].into(),  // D2 - 7
            ],
            indices![
                A1, C1, B1, A1, D1, C1, D2, B2, C2, D2, A2, B2, B2, A1, B1, B2, A2, A1, A2, D1, A1,
                A2, D2, D1, D2, C1, D1, D2, C2, C1, C2, B1, C1, C2, B2, B1,
            ],
        );
        model.position = position;
        model.scaling = scale;

        AppModel::new(model)
    }

    pub fn make_room(position: Point3<f32>, scale: f32) -> AppModel {
        enum Vert {
            A1 = 0,
            B1 = 1,
            C1 = 2,
            D1 = 3,
            A2 = 4,
            B2 = 5,
            C2 = 6,
            D2 = 7,
        }
        use Vert::*;

        let mut model = Model::new(
            vec![
                [-1.0, 0.0, -1.0, 0.0].into(),  // A1 - 0
                [1.0, 0.0, -1.0, 0.0].into(),   // B1 - 1
                [1.0, 0.0, 1.0, 0.0].into(),    // C1 - 2
                [-1.0, 0.0, 1.0, 0.0].into(),   // D1 - 3
                [-1.0, -1.0, -1.0, 0.0].into(), // A2 - 4
                [1.0, -1.0, -1.0, 0.0].into(),  // B2 - 5
                [1.0, -1.0, 1.0, 0.0].into(),   // C2 - 6
                [-1.0, -1.0, 1.0, 0.0].into(),  // D2 - 7
            ],
            indices![
                A2, C2, B2, A2, D2,
                C2,
                                D1, B1, C1,
                                D1, A1, B1,

                B1, A2, B2, B1, A1, A2, A1, D2, A2, A1, D1, D2, D1, C2, D2, D1, C1, C2, C1, B2, C2,
                C1, B1, B2,
            ],
        );
        model.position = position;
        model.scaling = scale;

        AppModel::new(model)
    }

    pub fn make_mirror(
        position: Point3<f32>,
        rotation: UnitQuaternion<f32>,
        scale: f32,
    ) -> AppModel {
        enum Vert {
            A = 0,
            B = 1,
            C = 2,
            D = 3,
        }
        use Vert::*;

        #[rustfmt::skip]
    let mut model = Model::new(
        vec![
            [-1.0, -1.0, 0.0, 0.0].into(),  // A - 0
            [1.0, -1.0, 0.0, 0.0].into(),   // B - 1
            [1.0, 1.0, 0.0, 0.0].into(),    // C - 2
            [-1.0, 1.0, 0.0, 0.0].into(),   // D - 3
        ],
        indices![
            A, C, D,
            A, B, C,
        ],
    );
        model.rotation = rotation;
        model.position = position;
        model.scaling = scale;
        model.material = Material::Mirror;

        AppModel::new(model)
    }
}

fn init_scene(device: Arc<Device>) -> Scene {
    let mut models = models::make_desk(Point3::new(0.0, -1.5, 0.0), 3.0);
    models.push(models::make_room([0.0, 2.5, 0.0].into(), 5.0));
    Scene::new(
        device,
        models,
        vec![SphereModel::new(Point3::new(0.0, -1.2, 0.0), 0.3)],
        DirectionLight::new(
            LightInfo::new(Point4::new(1.0, 0.98, 0.96, 0.0), 0.0),
            Vector3::new(0.2, -0.4, -0.3).normalize(),
        ),
        vec![
            PointLight::new(
                LightInfo::new(Point4::new(1.0, 0.98, 0.96, 0.0), 600.0),
                Point3::new(0.0, 2.3, 0.0),
            ),
        ],
        Camera::from_origin().move_at(4.185082,
                1.1902695,
                4.007931,).rotate(-0.24999996,
        0.8000001,
        0.0,),
    )
}

fn main() {
    let app = AnimationApp::new(Screen::new(1920, 1080), 2);
    let device = app.vulkan_device();

    let mut renderer = Renderer::new(app, 30, 2, &"some.png");
    let mut scene = init_scene(device);
/*
    for i in 0..1 {
        println!("Render frame {}", i);
        renderer.render_frame_to_video(&mut scene);

        scene.update_camera(|camera| {
            camera.rotate(0.0, 0.01, 0.0)
        });
    }
    renderer.end_video();
*/
    renderer.render_frame_to_image(&mut scene);

}
