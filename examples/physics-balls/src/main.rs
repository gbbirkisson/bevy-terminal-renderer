use rand::Rng;
use std::time::Duration;

use avian2d::{parry::shape::SharedShape, prelude::*};
use bevy::{app::ScheduleRunnerPlugin, prelude::*, scene::ScenePlugin};
use bevy_terminal_renderer::prelude::*;

const GROUND_SIZE: isize = 20;
const WALL_SIZE: isize = 5;
const CAMERA_SPEED: f32 = 10.0;

const NR_BALL_TYPES: usize = 3;
const BALLS: [char; NR_BALL_TYPES] = ['0', 'O', '*'];

// Uncomment to use emojis
// const NR_BALL_TYPES: usize = 7;
// const BALLS: [char; NR_BALL_TYPES] = ['🔴', '🔵', '🟢', '🟡', '🟠', '🟣', '🟤'];

#[derive(Component)]
pub struct Ball;

fn main() {
    // Initialize tracing_subscriber to write to a file
    let file_appender = tracing_appender::rolling::never("../../", "debug.log");
    tracing_subscriber::fmt().with_writer(file_appender).init();

    App::new()
        // Add absolute basics
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0, // Run at 60fps
            ))),
            TransformPlugin, // Needed to update global transforms
        ))
        // Add a physics engine
        .add_plugins((
            (AssetPlugin::default(), ScenePlugin), // Needed for aiven physics
            PhysicsPlugins::default(),
        ))
        // Add our plugin and a physics engine
        .add_plugins(TermPlugin::default())
        // Add our systems
        .add_systems(Startup, create_scene)
        .add_systems(
            Update,
            (camera_control, exit_control, spawn_balls, despawn_balls),
        )
        .run();
}

fn create_scene(mut commands: Commands) {
    // Setup camera
    commands.spawn(TermCameraBundle {
        transform: Transform::from_xyz(0.0, 10.0, 0.0),
        ..Default::default()
    });

    // Create ground
    commands
        .spawn((
            RigidBody::Static,
            <SharedShape as Into<Collider>>::into(SharedShape::cuboid(GROUND_SIZE as f32, 1.0)),
            Transform::from_xyz(0.0, 0.0, 1.0),
        ))
        .with_children(|p| {
            for i in -GROUND_SIZE..=GROUND_SIZE {
                p.spawn(TermSpriteBundle {
                    transform: Transform::from_xyz(i as f32, 0.0, 0.0),
                    char: TermChar('-'),
                });
            }
        });

    // Create walls
    for i in [-GROUND_SIZE, GROUND_SIZE] {
        commands
            .spawn((
                RigidBody::Static,
                <SharedShape as Into<Collider>>::into(SharedShape::cuboid(1.0, WALL_SIZE as f32)),
                Transform::from_xyz(i as f32, WALL_SIZE as f32, 1.0),
            ))
            .with_children(|p| {
                for i in -WALL_SIZE..=WALL_SIZE {
                    p.spawn(TermSpriteBundle {
                        transform: Transform::from_xyz(0.0, i as f32, 0.0),
                        char: TermChar('|'),
                    });
                }
            });
    }

    // Create text
    commands
        .spawn(Transform::from_xyz(0.0, -1.0, 0.0))
        .with_children(|p| {
            p.spawn(TermTextBundle {
                text: TermText::from("        Move camera: ↑ ↓ ← →"),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..Default::default()
            });
            p.spawn(TermTextBundle {
                text: TermText::from("Spawn balls: spacebar"),
                transform: Transform::from_xyz(0.0, -1.0, 0.0),
                ..Default::default()
            });
            p.spawn(TermTextBundle {
                text: TermText::from(" Exit: q"),
                transform: Transform::from_xyz(0.0, -2.0, 0.0),
                ..Default::default()
            });
        });
}

fn spawn_balls(mut input: EventReader<TermInput>, mut commands: Commands) {
    // Spawn balls when spacebar is pressed
    for i in input.read() {
        if let TermInput::SpaceBar = i {
            let mut rng = rand::thread_rng();
            let rx = (rng.gen_range(-GROUND_SIZE..=GROUND_SIZE) / 2) as f32;
            let ry = ((WALL_SIZE * 3) + rng.gen_range(0..=WALL_SIZE)) as f32;
            let btype = BALLS[rng.gen_range(0..NR_BALL_TYPES)];

            commands
                .spawn(Ball)
                .insert(RigidBody::Dynamic)
                .insert(Collider::circle(1.0))
                .insert(Restitution::new(1.1))
                .insert(TermSpriteBundle {
                    transform: Transform::from_xyz(rx, ry, 0.0),
                    char: TermChar(btype),
                });
        }
    }
}

fn despawn_balls(mut commands: Commands, query: Query<(Entity, &GlobalTransform), With<Ball>>) {
    // Despawn balls that fall off the screen
    for (entity, transform) in query.iter() {
        if transform.translation().y < -20.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn camera_control(
    mut input: EventReader<TermInput>,
    mut camera: Query<&mut Transform, With<TermCamera>>,
) {
    // Move camera with arrow keys
    let mut camera = camera.single_mut().expect("We should always have a camera");

    for i in input.read() {
        match i {
            TermInput::Left => {
                camera.translation -= Vec3::X * CAMERA_SPEED;
            }
            TermInput::Right => {
                camera.translation += Vec3::X * CAMERA_SPEED;
            }
            TermInput::Up => {
                camera.translation += Vec3::Y * CAMERA_SPEED / 2.0;
            }
            TermInput::Down => {
                camera.translation -= Vec3::Y * CAMERA_SPEED / 2.0;
            }
            _ => {}
        }
    }
}

fn exit_control(mut input: EventReader<TermInput>, mut command: EventWriter<TermCommand>) {
    // Exit when q is pressed
    for i in input.read() {
        if let TermInput::Character('q') = i {
            command.write(TermCommand::Exit);
        }
    }
}
