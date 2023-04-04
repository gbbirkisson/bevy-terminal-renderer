use bevy::{app::ScheduleRunnerSettings, prelude::*, utils::Duration};
use bevy_rapier2d::prelude::*;
use rand::Rng;

use bevy_terminal_renderer::{
    TerminalPlugin, Camera, CameraBundle, Char, TerminalBundle, TerminalCommand, TerminalInput, ZBuffer,
};

fn main() {
    // Initialize tracing_subscriber to write to a file
    let file_appender = tracing_appender::rolling::never(".", "debug.log");
    tracing_subscriber::fmt().with_writer(file_appender).init();

    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugins(MinimalPlugins)
        .add_plugin(TerminalPlugin::wide(true))
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
    commands.spawn(CameraBundle {
        position: TransformBundle::from(Transform::from_xyz(0.0, 20.0, 0.0)),
        camera: Camera,
    });

    // Create ground
    commands
        .spawn(Collider::cuboid(GROUND_SIZE as f32, 1.005)) // Make it a bit bigger than 1
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)))
        .with_children(|p| {
            for i in -GROUND_SIZE..=GROUND_SIZE {
                p.spawn(TerminalBundle {
                    position: TransformBundle::from(Transform::from_xyz(i as f32, 0.0, 0.0)),
                    char: Char('-'),
                    z: ZBuffer(1),
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
                    p.spawn(TerminalBundle {
                        position: TransformBundle::from(Transform::from_xyz(0.0, i as f32, 0.0)),
                        char: Char('|'),
                        z: ZBuffer(1),
                    });
                }
            });
    }

    // // Create ball
    // commands
    //     .spawn(RigidBody::Dynamic)
    //     .insert(Collider::ball(1.0))
    //     .insert(Restitution::coefficient(1.5))
    //     .insert(TerminalBundle {
    //         position: TransformBundle::from(Transform::from_xyz(-0.5, 20.0, 0.0)),
    //         char: Char('⚽'),
    //     });
    //
    // commands
    //     .spawn(RigidBody::Dynamic)
    //     .insert(Collider::ball(1.0))
    //     .insert(Restitution::coefficient(1.5))
    //     .insert(TerminalBundle {
    //         position: TransformBundle::from(Transform::from_xyz(0.5, 30.0, 0.0)),
    //         char: Char('⚽'),
    //     });
    // commands.spawn(TerminalBundle {
    //     position: TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)),
    //     char: Char('\u{1F378}'),
    // });
    // commands.spawn(TerminalBundle {
    //     position: TransformBundle::from(Transform::from_xyz(1.0, 0.0, 0.0)),
    //     char: Char('\u{1F378}'),
    // });
}

fn spawn_balls(mut input: EventReader<TerminalInput>, mut commands: Commands) {
    for i in input.iter() {
        match i {
            TerminalInput::Space => {
                let mut rng = rand::thread_rng();
                let rx = (rng.gen_range(-GROUND_SIZE..=GROUND_SIZE) / 2) as f32;
                let ry = ((WALL_SIZE * 3) + rng.gen_range(0..=WALL_SIZE)) as f32;
                commands
                    .spawn(RigidBody::Dynamic)
                    .insert(Collider::ball(1.0))
                    .insert(Restitution::coefficient(1.2))
                    .insert(TerminalBundle {
                        position: TransformBundle::from(Transform::from_xyz(rx, ry, 0.0)),
                        char: Char('🔴'),
                        ..Default::default()
                    });
            }
            _ => {}
        }
    }
}

fn controls(
    mut input: EventReader<TerminalInput>,
    mut command: EventWriter<TerminalCommand>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    let mut camera = camera
        .get_single_mut()
        .expect("We should always have a camera");

    for i in input.iter() {
        match i {
            TerminalInput::Escape => {
                command.send(TerminalCommand::Exit);
            }
            TerminalInput::Character(c) if c == &'q' => {
                command.send(TerminalCommand::Exit);
            }
            TerminalInput::Left => {
                camera.translation -= Vec3::X * 2.0;
            }
            TerminalInput::Right => {
                camera.translation += Vec3::X * 2.0;
            }
            TerminalInput::Up => {
                camera.translation += Vec3::Y;
            }
            TerminalInput::Down => {
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
