use bevy::{app::ScheduleRunnerSettings, prelude::*, utils::Duration};
use bevy_rapier2d::prelude::*;
use rand::Rng;

use bevy_terminal_renderer::*;

const GROUND_SIZE: isize = 50;
const WALL_SIZE: isize = 10;

const NR_BALL_TYPES: usize = 7;
const BALLS: [char; NR_BALL_TYPES] = ['🔴', '🔵', '🟢', '🟡', '🟠', '🟣', '🟤'];

fn main() {
    // Initialize tracing_subscriber to write to a file
    let file_appender = tracing_appender::rolling::never(".", "debug.log");
    tracing_subscriber::fmt().with_writer(file_appender).init();

    // Initialize app
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0, // Run at 60 fps
        )))
        .add_plugins(MinimalPlugins)
        .add_plugin(TermPlugin::wide(true))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0))
        .add_startup_system(create_scene)
        .add_system(camera_control)
        .add_system(exit_control)
        .add_system(spawn_balls)
        .run();
}

fn create_scene(mut commands: Commands) {
    // Setup camera
    commands.spawn(TermCameraBundle {
        position: TransformBundle::from(Transform::from_xyz(0.0, 30.0, 0.0)),
        ..Default::default()
    });

    // Create ground
    commands
        .spawn(Collider::cuboid(GROUND_SIZE as f32, 1.005)) // Its a bit bigger than 1
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
    // Spawn balls when spacebar is pressed
    for i in input.iter() {
        match i {
            TermInput::SpaceBar => {
                let mut rng = rand::thread_rng();
                let rx = (rng.gen_range(-GROUND_SIZE..=GROUND_SIZE) / 2) as f32;
                let ry = ((WALL_SIZE * 3) + rng.gen_range(0..=WALL_SIZE)) as f32;
                let btype = BALLS[rng.gen_range(0..NR_BALL_TYPES)];
                commands
                    .spawn(RigidBody::Dynamic)
                    .insert(Collider::ball(1.0))
                    .insert(Restitution::coefficient(1.1))
                    .insert(TermSpriteBundle {
                        position: TransformBundle::from(Transform::from_xyz(rx, ry, 0.0)),
                        char: TermChar(btype),
                        ..Default::default()
                    });
            }
            _ => {}
        }
    }
}

fn camera_control(
    mut input: EventReader<TermInput>,
    mut camera: Query<&mut Transform, With<TermCamera>>,
) {
    // Move camera with arrow keys
    let mut camera = camera
        .get_single_mut()
        .expect("We should always have a camera");

    for i in input.iter() {
        match i {
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

fn exit_control(mut input: EventReader<TermInput>, mut command: EventWriter<TermCommand>) {
    // Exit when escape or q is pressed
    for i in input.iter() {
        match i {
            TermInput::Escape => {
                command.send(TermCommand::Exit);
            }
            TermInput::Character(c) if c == &'q' => {
                command.send(TermCommand::Exit);
            }
            _ => {}
        }
    }
}
