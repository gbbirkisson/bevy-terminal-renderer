use bevy::{app::ScheduleRunnerSettings, prelude::*, utils::Duration};
use bevy_terminal_renderer::*;

const DIAMOND_SIZE: isize = 12;
const ROTATION_SPEED: f32 = 2.0;
const DIAMOND_CHAR: char = '+';

#[derive(Component)]
pub struct Diamond;

#[derive(Bundle)]
pub struct DiamondBundle {
    pub diamond: Diamond,

    #[bundle]
    pub transform: TransformBundle,
}

fn main() {
    App::new()
        // Add bevy bare necessities
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0, // Run at 60 fps
        )))
        .add_plugins(MinimalPlugins) // The absolute basics
        .add_plugin(TransformPlugin) // This is needed to update global transforms
        // Add our plugin and a physics engine
        .add_plugin(TermPlugin {
            wide: true,
            minz: 0.0,
        })
        .add_startup_system(create_scene)
        .add_system(spin_controls)
        .add_system(exit_control)
        .run();
}

fn create_scene(mut commands: Commands) {
    // Setup camera
    commands.spawn(TermCameraBundle {
        transform: TransformBundle::from(Transform::from_xyz(0.0, -3.0, 0.0)),
        ..Default::default()
    });

    // Create diamonds
    create_diamond(&mut commands, Transform::from_xyz(0.0, 0.0, 0.0));
    create_diamond(
        &mut commands,
        Transform::from_xyz(-2.0 * DIAMOND_SIZE as f32, 8.0, 0.0)
            .with_scale(Vec3::new(0.5, 0.5, 0.5)),
    );
    create_diamond(
        &mut commands,
        Transform::from_xyz(2.0 * DIAMOND_SIZE as f32, -5.0, 0.0)
            .with_scale(Vec3::new(0.3, 0.3, 0.3)),
    );

    // Create text
    commands
        .spawn(TransformBundle::from(Transform::from_xyz(
            0.0,
            -(DIAMOND_SIZE + 3) as f32,
            0.0,
        )))
        .with_children(|p| {
            p.spawn(TermTextBundle {
                text: TermText::from("Rotate on x-axis: ↑ ↓"),
                transform: TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)),
                ..Default::default()
            });
            p.spawn(TermTextBundle {
                text: TermText::from("      Exit: q"),
                transform: TransformBundle::from(Transform::from_xyz(0.0, -1.0, 0.0)),
                ..Default::default()
            });
        });
}

fn create_diamond(commands: &mut Commands, transform: Transform) {
    commands
        .spawn(DiamondBundle {
            diamond: Diamond,
            transform: TransformBundle::from(transform),
        })
        .with_children(|p| {
            for i in 0..DIAMOND_SIZE {
                p.spawn(TermSpriteBundle {
                    transform: TransformBundle::from(Transform::from_xyz(
                        i as f32,
                        (DIAMOND_SIZE - i) as f32,
                        0.0,
                    )),
                    char: TermChar(DIAMOND_CHAR),
                });
                p.spawn(TermSpriteBundle {
                    transform: TransformBundle::from(Transform::from_xyz(
                        (-DIAMOND_SIZE + i) as f32,
                        i as f32,
                        0.0,
                    )),
                    char: TermChar(DIAMOND_CHAR),
                });
                p.spawn(TermSpriteBundle {
                    transform: TransformBundle::from(Transform::from_xyz(
                        -i as f32,
                        -(DIAMOND_SIZE - i) as f32,
                        0.0,
                    )),
                    char: TermChar(DIAMOND_CHAR),
                });
                p.spawn(TermSpriteBundle {
                    transform: TransformBundle::from(Transform::from_xyz(
                        -(-DIAMOND_SIZE + i) as f32,
                        -i as f32,
                        0.0,
                    )),
                    char: TermChar(DIAMOND_CHAR),
                });

                p.spawn(TermSpriteBundle {
                    transform: TransformBundle::from(Transform::from_xyz(
                        0.0,
                        (DIAMOND_SIZE - i) as f32,
                        i as f32,
                    )),
                    char: TermChar(DIAMOND_CHAR),
                });
                p.spawn(TermSpriteBundle {
                    transform: TransformBundle::from(Transform::from_xyz(
                        0.0,
                        i as f32,
                        (-DIAMOND_SIZE + i) as f32,
                    )),
                    char: TermChar(DIAMOND_CHAR),
                });
                p.spawn(TermSpriteBundle {
                    transform: TransformBundle::from(Transform::from_xyz(
                        0.0,
                        -(DIAMOND_SIZE - i) as f32,
                        -i as f32,
                    )),
                    char: TermChar(DIAMOND_CHAR),
                });
                p.spawn(TermSpriteBundle {
                    transform: TransformBundle::from(Transform::from_xyz(
                        0.0,
                        -i as f32,
                        -(-DIAMOND_SIZE + i) as f32,
                    )),
                    char: TermChar(DIAMOND_CHAR),
                });

                p.spawn(TermSpriteBundle {
                    transform: TransformBundle::from(Transform::from_xyz(
                        (DIAMOND_SIZE - i) as f32,
                        0.0,
                        i as f32,
                    )),
                    char: TermChar(DIAMOND_CHAR),
                });
                p.spawn(TermSpriteBundle {
                    transform: TransformBundle::from(Transform::from_xyz(
                        i as f32,
                        0.0,
                        (-DIAMOND_SIZE + i) as f32,
                    )),
                    char: TermChar(DIAMOND_CHAR),
                });
                p.spawn(TermSpriteBundle {
                    transform: TransformBundle::from(Transform::from_xyz(
                        -(DIAMOND_SIZE - i) as f32,
                        0.0,
                        -i as f32,
                    )),
                    char: TermChar(DIAMOND_CHAR),
                });
                p.spawn(TermSpriteBundle {
                    transform: TransformBundle::from(Transform::from_xyz(
                        -i as f32,
                        0.0,
                        -(-DIAMOND_SIZE + i) as f32,
                    )),
                    char: TermChar(DIAMOND_CHAR),
                });
            }
        });
}

fn spin_controls(
    time: Res<Time>,
    mut query: Query<(&mut Transform, With<Diamond>)>,
    mut input: EventReader<TermInput>,
) {
    let mut x_rotation = 0.0;
    let y_rotation = ROTATION_SPEED / 2.0;
    for i in input.iter() {
        match i {
            TermInput::Up => {
                x_rotation = -ROTATION_SPEED;
            }
            TermInput::Down => {
                x_rotation = ROTATION_SPEED;
            }
            _ => {}
        }
    }

    for (mut transform, _) in query.iter_mut() {
        transform.rotate(
            Quat::from_rotation_x(time.delta_seconds() * x_rotation)
                * Quat::from_rotation_y(time.delta_seconds() * y_rotation),
        );
    }
}

fn exit_control(mut input: EventReader<TermInput>, mut command: EventWriter<TermCommand>) {
    // Exit when q is pressed
    for i in input.iter() {
        match i {
            TermInput::Character(c) if c == &'q' => {
                command.send(TermCommand::Exit);
            }
            _ => {}
        }
    }
}
