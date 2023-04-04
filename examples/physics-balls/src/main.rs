use bevy::{app::ScheduleRunnerSettings, prelude::*, utils::Duration};
use bevy_rapier2d::prelude::*;
use rand::Rng;

use bevy_terminal_renderer::*;

fn main() {
    // Initialize tracing_subscriber to write to a file
    let file_appender = tracing_appender::rolling::never(".", "debug.log");
    tracing_subscriber::fmt().with_writer(file_appender).init();

    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugins(MinimalPlugins)
        .add_plugin(TermPlugin::wide(true))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0))
        .add_startup_system(create_entities)
        .add_system(controls)
        .add_system(spawn_balls)
        // .add_system(move_around)
        .run();
}

const GROUND_SIZE: isize = 50;
const WALL_SIZE: isize = 10;

fn create_entities(mut commands: Commands) {
    // Setup camera
    commands.spawn(TermCameraBundle {
        position: TransformBundle::from(Transform::from_xyz(0.0, 30.0, 0.0)),
        ..Default::default()
    });

    // Create ground
    commands
        .spawn(Collider::cuboid(GROUND_SIZE as f32, 1.005)) // Make it a bit bigger than 1
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)))
        .with_children(|p| {
            for i in -GROUND_SIZE..=GROUND_SIZE {
                p.spawn(TermSpriteBundle {
                    position: TransformBundle::from(Transform::from_xyz(i as f32, 0.0, 0.0)),
                    char: TermChar('-'),
                    z: TermZBuffer(1),
                });
            }
        });

    // Create walls
    for i in [-GROUND_SIZE, GROUND_SIZE] {
        commands
            .spawn(Collider::cuboid(1.0, WALL_SIZE as f32))
            .insert(TransformBundle::from(Transform::from_xyz(
                i as f32,
                WALL_SIZE as f32,
                0.0,
            )))
            .with_children(|p| {
                for i in -WALL_SIZE..=WALL_SIZE {
                    p.spawn(TermSpriteBundle {
                        position: TransformBundle::from(Transform::from_xyz(0.0, i as f32, 0.0)),
                        char: TermChar('|'),
                        z: TermZBuffer(1),
                    });
                }
            });
    }
}

fn spawn_balls(mut input: EventReader<TermInput>, mut commands: Commands) {
    for i in input.iter() {
        match i {
            TermInput::Space => {
                let mut rng = rand::thread_rng();
                let rx = (rng.gen_range(-GROUND_SIZE..=GROUND_SIZE) / 2) as f32;
                let ry = ((WALL_SIZE * 3) + rng.gen_range(0..=WALL_SIZE)) as f32;
                commands
                    .spawn(RigidBody::Dynamic)
                    .insert(Collider::ball(1.0))
                    .insert(Restitution::coefficient(1.2))
                    .insert(TermSpriteBundle {
                        position: TransformBundle::from(Transform::from_xyz(rx, ry, 0.0)),
                        char: TermChar('🔴'),
                        ..Default::default()
                    });
            }
            _ => {}
        }
    }
}

fn controls(
    mut input: EventReader<TermInput>,
    mut command: EventWriter<TermCommand>,
    mut camera: Query<&mut Transform, With<TermCamera>>,
) {
    let mut camera = camera
        .get_single_mut()
        .expect("We should always have a camera");

    for i in input.iter() {
        match i {
            TermInput::Escape => {
                command.send(TermCommand::Exit);
            }
            TermInput::Character(c) if c == &'q' => {
                command.send(TermCommand::Exit);
            }
            TermInput::Left => {
                camera.translation -= Vec3::X * 2.0;
            }
            TermInput::Right => {
                camera.translation += Vec3::X * 2.0;
            }
            TermInput::Up => {
                camera.translation += Vec3::Y;
            }
            TermInput::Down => {
                camera.translation -= Vec3::Y;
            }
            _ => {}
        }
    }
}

// fn move_around(mut query: Query<&mut Position>) {
//     for mut pos in query.iter_mut() {
//         // if pos.0 > 151 {
//         //     pos.0 = 0;
//         // } else {
//         //     pos.0 += 1;
//         // }
//         if pos.1 > 30 {
//             pos.1 = -30;
//         } else {
//             pos.1 += 1;
//         }
//     }
// }
