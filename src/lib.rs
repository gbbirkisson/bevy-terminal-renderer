use std::sync::Arc;

use bevy::app::AppExit;
use bevy::prelude::*;

use pancurses::{
    curs_set, endwin, getmouse, initscr, mousemask, nl, noecho, resize_term, Input, Window,
    ALL_MOUSE_EVENTS,
};

#[derive(Component)]
struct Terminal {
    window: Arc<Window>,
    wide: bool,
}

#[derive(Resource)]
struct TerminalContext {
    wide: bool,
}

unsafe impl Send for Terminal {}
unsafe impl Sync for Terminal {}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set(base)]
enum TerminalBaseSet {
    Handle,
}

#[derive(Component)]
pub struct Char(pub char);

#[derive(Component)]
pub struct ZBuffer(pub isize);

#[derive(Bundle)]
pub struct TerminalBundle {
    pub char: Char,
    pub z: ZBuffer,

    #[bundle]
    pub position: TransformBundle,
}

impl Default for TerminalBundle {
    fn default() -> Self {
        Self {
            char: Char('?'),
            position: TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)),
            z: ZBuffer(0),
        }
    }
}

#[derive(Component)]
pub struct Camera;

#[derive(Bundle)]
pub struct CameraBundle {
    pub camera: Camera,

    #[bundle]
    pub position: TransformBundle,
}

#[derive(Debug)]
pub enum TerminalInput {
    Mouse(i32, i32),
    Character(char),

    Left,
    Right,
    Up,
    Down,

    Space,
    BackSpace,
    Enter,
    Tab,
    Escape,

    Home,
    End,
    Insert,
    Delete,
    PageUp,
    PageDown,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

pub enum TerminalCommand {
    Exit,
}

pub struct TerminalPlugin {
    wide: bool,
}

impl TerminalPlugin {
    pub fn wide(wide: bool) -> Self {
        Self { wide }
    }
}

impl Plugin for TerminalPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerminalContext { wide: self.wide })
            .add_event::<TerminalInput>()
            .add_event::<TerminalCommand>()
            .add_plugin(TransformPlugin)
            .add_startup_system(create_terminal)
            .add_system(handle_terminal.in_base_set(TerminalBaseSet::Handle))
            .configure_set(TerminalBaseSet::Handle.before(CoreSet::PreUpdate));
    }
}

fn create_terminal(mut commands: Commands, context: Res<TerminalContext>) {
    let window = initscr();

    nl();
    noecho();
    curs_set(0);
    window.timeout(0);
    window.keypad(true);
    mousemask(ALL_MOUSE_EVENTS, None);

    commands.spawn(Terminal {
        window: Arc::new(window),
        wide: context.wide,
    });
}

fn handle_terminal(
    terminal: Query<&Terminal>,
    camera: Query<&GlobalTransform, With<Camera>>,
    entities: Query<(&GlobalTransform, &Char, Option<&ZBuffer>)>,
    mut ev_input: EventWriter<TerminalInput>,
    mut exit: EventWriter<AppExit>,
    mut ev_cmd: EventReader<TerminalCommand>,
) {
    // Get our window
    let terminal = terminal
        .get_single()
        .expect("We should always have a terminal");
    let window = &terminal.window;
    let wide = terminal.wide;

    let mut resize = false;

    // Look for commands for the terminal
    for ev in ev_cmd.iter() {
        match ev {
            TerminalCommand::Exit => {
                curs_set(1);
                endwin();
                exit.send(AppExit);
                return;
            }
        }
    }

    // Send events
    if let Some(ev) = terminal.window.getch() {
        match ev {
            Input::KeyBackspace => {
                ev_input.send(TerminalInput::BackSpace);
            }
            Input::Character(c) if c == ' ' => {
                ev_input.send(TerminalInput::Space);
            }
            Input::Character(c) if c == '\n' => {
                ev_input.send(TerminalInput::Enter);
            }
            Input::Character(c) if c == '\t' => {
                ev_input.send(TerminalInput::Tab);
            }
            Input::Character(c) if c == '\u{1b}' => {
                ev_input.send(TerminalInput::Escape);
            }
            Input::Character(c) => {
                ev_input.send(TerminalInput::Character(c));
            }
            Input::KeyMouse => {
                if let Ok(mouse_event) = getmouse() {
                    ev_input.send(TerminalInput::Mouse(mouse_event.x, mouse_event.y));
                };
            }
            Input::KeyResize => {
                resize = true;
            }
            Input::KeyLeft => {
                ev_input.send(TerminalInput::Left);
            }
            Input::KeyRight => {
                ev_input.send(TerminalInput::Right);
            }
            Input::KeyUp => {
                ev_input.send(TerminalInput::Up);
            }
            Input::KeyDown => {
                ev_input.send(TerminalInput::Down);
            }
            Input::KeyHome => {
                ev_input.send(TerminalInput::Home);
            }
            Input::KeyEnd => {
                ev_input.send(TerminalInput::End);
            }
            Input::KeyIC => {
                ev_input.send(TerminalInput::Insert);
            }
            Input::KeyDC => {
                ev_input.send(TerminalInput::Delete);
            }
            Input::KeyPPage => {
                ev_input.send(TerminalInput::PageUp);
            }
            Input::KeyNPage => {
                ev_input.send(TerminalInput::PageDown);
            }
            Input::KeyF1 => {
                ev_input.send(TerminalInput::F1);
            }
            Input::KeyF2 => {
                ev_input.send(TerminalInput::F2);
            }
            Input::KeyF3 => {
                ev_input.send(TerminalInput::F3);
            }
            Input::KeyF4 => {
                ev_input.send(TerminalInput::F4);
            }
            Input::KeyF5 => {
                ev_input.send(TerminalInput::F5);
            }
            Input::KeyF6 => {
                ev_input.send(TerminalInput::F6);
            }
            Input::KeyF7 => {
                ev_input.send(TerminalInput::F7);
            }
            Input::KeyF8 => {
                ev_input.send(TerminalInput::F8);
            }
            Input::KeyF9 => {
                ev_input.send(TerminalInput::F9);
            }
            Input::KeyF10 => {
                ev_input.send(TerminalInput::F10);
            }
            Input::KeyF11 => {
                ev_input.send(TerminalInput::F11);
            }
            Input::KeyF12 => {
                ev_input.send(TerminalInput::F12);
            }
            _ => {
                warn!("Unknown input: {:?}", ev);
            }
        }
    }

    // Resize terminal
    if resize {
        resize_term(0, 0);
        window.erase();
    }

    // Prepare drawing
    let c: usize = (window.get_max_x() - 2) as usize;
    let r: usize = (window.get_max_y()) as usize;
    let mut buffer = vec![vec![(' ', isize::MIN); r]; c];

    // Print resize
    if resize {
        info!("New terminal size: {}, {}", c, r);
    }

    let (camera_offset_x, camera_offset_y) = match camera.get_single() {
        Err(_) => (0, 0),
        Ok(camera) => {
            let camera_x = camera.translation().x.round() as isize;
            let camera_y = camera.translation().y.round() as isize;

            let camera_offset_x = (-camera_x + c as isize) / 2;
            let camera_offset_y = (camera_y + r as isize) / 2;

            (camera_offset_x, camera_offset_y)
        }
    };

    // Fill buffer
    for (transform, icon, z) in entities.iter() {
        let mut x = transform.translation().x.floor() as isize;
        let y = transform.translation().y.floor() as isize;
        let z = z.map(|z| z.0).unwrap_or(0);

        if wide {
            x *= 2;
        }

        let x = (x + camera_offset_x) as usize;
        let y = (-y + camera_offset_y) as usize;

        if x < c && y < r {
            let (_, oldz) = buffer[x][y];
            if z >= oldz {
                buffer[x][y] = (icon.0, z);
            }
        }
    }

    // Draw buffer
    for x in 0..c {
        for y in 0..r {
            window.mvaddstr(y as i32, x as i32, format!("{}", buffer[x][y].0));
        }
    }
}
