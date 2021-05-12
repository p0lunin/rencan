use nalgebra::{Point3, Point4, UnitQuaternion, Vector3, Isometry3, Translation3};
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
    let mut models = Vec::new();
    models.push(models::make_room([0.0, 2.5, 0.0].into(), 5.0));
    Scene::new(
        device,
        models,
        vec![
            SphereModel::new(Point3::new(0.0, 1.5, 0.0), 0.3),
            SphereModel::new(Point3::new(0.0, 0.5, 0.3), 0.45),
        ],
        DirectionLight::new(
            LightInfo::new(Point4::new(1.0, 0.98, 0.96, 0.0), 0.0),
            Vector3::new(0.2, -0.4, -0.3).normalize(),
        ),
        vec![
            PointLight::new(
                LightInfo::new(Point4::new(1.0, 0.98, 0.96, 0.0), 800.0),
                Point3::new(0.0, 2.3, 0.0),
            ),
        ],
        Camera::from_origin().move_at(4.185082,
                1.1902695,
                4.007931,).rotate(-0.34,
        0.8000001,
        0.0,),
    )
}

use rapier3d::dynamics::{JointSet, RigidBodySet, IntegrationParameters, RigidBodyBuilder, RigidBody};
use rapier3d::geometry::{BroadPhase, NarrowPhase, ColliderSet, Collider, ColliderBuilder, InteractionGroups};
use rapier3d::pipeline::PhysicsPipeline;

struct PShere {
    sphere: SphereModel,
    rigid_body: RigidBody,
    collider: Collider
}

fn main() {
    let app = AnimationApp::new(Screen::new(1280, 720), 5, 3);
    let device = app.vulkan_device();

    let mut renderer = Renderer::new(app, 30, &"some.mp4");
    let mut scene = init_scene(device);

    let mut pipeline = PhysicsPipeline::new();
    let gravity = Vector3::new(0.0, -9.81, 0.0);
    let mut integration_parameters = IntegrationParameters::default();
    integration_parameters.set_inv_dt(30.0);
    let mut broad_phase = BroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();
    let mut joints = JointSet::new();
    let event_handler = ();

    const COL_GROUP: InteractionGroups = InteractionGroups::new(0b10, 0b10);

    for (i, sphere) in scene.data.sphere_models.state().iter().enumerate() {
        let rigid_body = RigidBodyBuilder::new_dynamic()
            .translation(sphere.center.x, sphere.center.y, sphere.center.z)
            .user_data(i as u128)
            .build();
        let collider = ColliderBuilder::ball(sphere.radius)
            .collision_groups(COL_GROUP)
            .build();
        let parent_handle = bodies.insert(rigid_body);
        colliders.insert(collider, parent_handle, &mut bodies);
    }

    let floor_rb_handle = bodies.insert(
        RigidBodyBuilder::new_static()
            .translation(0.0, -2.5, 0.0)
            .user_data(300000)
            .build()
    );
    colliders.insert(
        ColliderBuilder::cuboid(5.0, 0.1, 5.0)
            .collision_groups(COL_GROUP)
            .build(),
        floor_rb_handle,
        &mut bodies,
    );

    let floor_rb_handle = bodies.insert(
        RigidBodyBuilder::new_static()
            .translation(0.0, 0.0, -5.0)
            .user_data(300000)
            .build()
    );
    colliders.insert(
        ColliderBuilder::cuboid(5.0, 5.0, 0.1)
            .collision_groups(COL_GROUP)
            .build(),
        floor_rb_handle,
        &mut bodies,
    );

    for i in 0..120 {
        println!("Render frame {}", i);
        renderer.render_frame_to_video(&mut scene, i);

        pipeline.step(
            &gravity,
            &integration_parameters,
            &mut broad_phase,
            &mut narrow_phase,
            &mut bodies,
            &mut colliders,
            &mut joints,
            None,
            None,
            &event_handler
        );
        scene.data.sphere_models = scene.data.sphere_models.change(|mut spheres| {
            for (_, body) in bodies.iter() {
                if body.user_data == 300000 {
                    continue;
                }
                spheres[body.user_data as usize].center = Point3::new(
                    body.position().translation.vector.x,
                    body.position().translation.vector.y,
                    body.position().translation.vector.z,
                );
            }
            spheres
        });
    }
    renderer.end_video();

    //renderer.render_frame_to_image(&mut scene);
}
