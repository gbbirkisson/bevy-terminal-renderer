use bevy::app::AppExit;
use bevy::prelude::*;

use pancurses::{
    curs_set, endwin, getmouse, initscr, mousemask, nl, noecho, resize_term, Input, Window,
    ALL_MOUSE_EVENTS,
};

#[derive(Component)]
struct Term {
    window: Window,
    wide: bool,
    minz: f32,
}

// Window cannot be passed between threads, but there is only ever 1 thread at a time that uses it
// at a time. Because there is only 1 system that uses it. I have no idea if this is a good idea or
// not, but it works on my machine :D
unsafe impl Send for Term {}
unsafe impl Sync for Term {}

#[derive(Resource)]
struct TermContext {
    wide: bool,
    minz: f32,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set(base)]
enum TermBaseSet {
    Handle,
}

#[derive(Component)]
pub struct TermChar(pub char);

#[derive(Component)]
pub struct TermText(pub String);

impl From<&str> for TermText {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Component)]
pub enum TermTextAlign {
    LEFT,
    CENTER,
    RIGHT,
}

#[derive(Bundle)]
pub struct TermSpriteBundle {
    pub char: TermChar,

    #[bundle]
    pub transform: TransformBundle,
}

impl Default for TermSpriteBundle {
    fn default() -> Self {
        Self {
            char: TermChar('?'),
            transform: TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)),
        }
    }
}

#[derive(Bundle)]
pub struct TermTextBundle {
    pub text: TermText,
    pub align: TermTextAlign,

    #[bundle]
    pub transform: TransformBundle,
}

impl Default for TermTextBundle {
    fn default() -> Self {
        Self {
            text: TermText::from("?"),
            align: TermTextAlign::CENTER,
            transform: TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)),
        }
    }
}

#[derive(Component)]
pub struct TermCamera;

#[derive(Component)]
pub struct TermSize(i32, i32);

#[derive(Bundle)]
pub struct TermCameraBundle {
    pub camera: TermCamera,

    #[bundle]
    pub transform: TransformBundle,
}

impl Default for TermCameraBundle {
    fn default() -> Self {
        Self {
            camera: TermCamera,
            transform: TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)),
        }
    }
}

#[derive(Debug)]
pub enum TermInput {
    Mouse(i32, i32),
    Character(char),

    Left,
    Right,
    Up,
    Down,

    SpaceBar,
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

pub enum TermCommand {
    Exit,
}

pub struct TermPlugin {
    pub wide: bool,
    pub minz: f32,
}

impl Default for TermPlugin {
    fn default() -> Self {
        Self {
            wide: false,
            minz: f32::MIN,
        }
    }
}

impl Plugin for TermPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TermContext {
            wide: self.wide,
            minz: self.minz,
        })
        .add_event::<TermInput>()
        .add_event::<TermCommand>()
        .add_startup_system(create_terminal)
        .add_system(handle_terminal.in_base_set(TermBaseSet::Handle))
        .configure_set(TermBaseSet::Handle.before(CoreSet::PreUpdate));
    }
}

fn create_terminal(mut commands: Commands, context: Res<TermContext>) {
    let window = initscr();

    // Configure window
    nl();
    noecho();
    curs_set(0);
    window.timeout(0);
    window.keypad(true);
    mousemask(ALL_MOUSE_EVENTS, None);

    // Create components
    let (x, y) = get_window_size(&window, context.wide);
    commands.spawn(TermSize(x, y));
    commands.spawn(Term {
        window,
        wide: context.wide,
        minz: context.minz,
    });
}

#[allow(clippy::too_many_arguments)]
fn handle_terminal(
    terminal: Query<&Term>,
    mut terminal_size: Query<&mut TermSize>,
    camera: Query<&GlobalTransform, With<TermCamera>>,
    chars: Query<(&GlobalTransform, &TermChar)>,
    texts: Query<(&GlobalTransform, &TermText, &TermTextAlign)>,
    mut ev_input: EventWriter<TermInput>,
    mut exit: EventWriter<AppExit>,
    mut ev_cmd: EventReader<TermCommand>,
) {
    // Get our data
    let terminal = terminal
        .get_single()
        .expect("We should always have a terminal");
    let mut terminal_size = terminal_size
        .get_single_mut()
        .expect("We should always have a terminal");
    let window = &terminal.window;
    let wide = terminal.wide;
    let minz = terminal.minz;

    let mut resize = false;

    // Look for commands for the terminal
    #[allow(clippy::never_loop)]
    for ev in ev_cmd.iter() {
        match ev {
            TermCommand::Exit => {
                curs_set(1);
                endwin();
                exit.send(AppExit);
                return;
            }
        }
    }

    // Handle events
    if let Some(ev) = terminal.window.getch() {
        match ev {
            Input::KeyResize => {
                resize = true;
            }
            _ => map_and_send_events(ev, &mut ev_input),
        }
    }

    // Resize terminal
    if resize {
        resize_term(0, 0);
        window.erase();
    }

    let (c, r) = get_window_size(window, wide);

    // Update terminal size
    terminal_size.0 = c;
    terminal_size.1 = r;

    // Prepare drawing
    let c = c as usize;
    let r = r as usize;
    let mut buffer = vec![vec![(' ', minz); r]; c];

    // Calculate camera offset
    let (camera_offset_x, camera_offset_y) = match camera.get_single() {
        Err(_) => (0, 0),
        Ok(camera) => {
            let camera_x = camera.translation().x.round() as isize;
            let camera_y = camera.translation().y.round() as isize;

            let camera_offset_x = (-camera_x + (c as isize)) / 2;
            let camera_offset_y = (camera_y + (r as isize)) / 2;

            (camera_offset_x, camera_offset_y)
        }
    };

    // Fill buffer with chars
    for (transform, char) in chars.iter() {
        let mut x = transform.translation().x.floor() as isize;
        let y = transform.translation().y.floor() as isize;
        let z = transform.translation().z;

        if wide {
            x *= 2;
        }

        let x = x + camera_offset_x;
        let y = -y + camera_offset_y;

        if x < 0 || y < 0 {
            // This char is not in view
            continue;
        }

        let x = x as usize;
        let y = y as usize;

        if x < c && y < r {
            let (_, oldz) = buffer[x][y];
            if z >= oldz {
                buffer[x][y] = (char.0, z);
            }
        }
    }

    // Fill buffer with text
    for (transform, text, align) in texts.iter() {
        let text = &text.0;
        let text_len = text.len();
        let x = transform.translation().x.floor() as isize;
        let y = transform.translation().y.floor() as isize;
        let z = transform.translation().z.floor();

        let x = x + camera_offset_x;
        let y = -y + camera_offset_y;

        let x = match align {
            TermTextAlign::LEFT => x,
            TermTextAlign::CENTER => x - (text_len as isize / 2),
            TermTextAlign::RIGHT => x - text_len as isize,
        };

        if x + (text_len as isize) < 0 || y < 0 {
            // This string is not in view
            continue;
        }

        let x = x as usize;
        let y = y as usize;

        if x > c || y > r {
            // This string is not in view
            continue;
        }

        for (i, char) in text.chars().enumerate() {
            let x = x + i;

            if x < c && y < r {
                let (_, oldz) = buffer[x][y];
                if z >= oldz {
                    buffer[x][y] = (char, z);
                }
            }
        }
    }

    // Draw buffer
    #[allow(clippy::needless_range_loop)]
    for x in 0..c {
        for y in 0..r {
            // Using the string method here to handle emojis
            window.mvaddstr(y as i32, x as i32, format!("{}", buffer[x][y].0));
        }
    }
}

fn get_window_size(window: &Window, wide: bool) -> (i32, i32) {
    let sub = if wide { 2 } else { 1 };
    ((window.get_max_x() - sub), window.get_max_y())
}

fn map_and_send_events(ev: Input, ev_input: &mut EventWriter<TermInput>) {
    match ev {
        Input::KeyBackspace => {
            ev_input.send(TermInput::BackSpace);
        }
        Input::Character(c) if c == ' ' => {
            ev_input.send(TermInput::SpaceBar);
        }
        Input::Character(c) if c == '\n' => {
            ev_input.send(TermInput::Enter);
        }
        Input::Character(c) if c == '\t' => {
            ev_input.send(TermInput::Tab);
        }
        Input::Character(c) if c == '\u{1b}' => {
            ev_input.send(TermInput::Escape);
        }
        Input::Character(c) => {
            ev_input.send(TermInput::Character(c));
        }
        Input::KeyMouse => {
            if let Ok(mouse_event) = getmouse() {
                ev_input.send(TermInput::Mouse(mouse_event.x, mouse_event.y));
            };
        }
        Input::KeyLeft => {
            ev_input.send(TermInput::Left);
        }
        Input::KeyRight => {
            ev_input.send(TermInput::Right);
        }
        Input::KeyUp => {
            ev_input.send(TermInput::Up);
        }
        Input::KeyDown => {
            ev_input.send(TermInput::Down);
        }
        Input::KeyHome => {
            ev_input.send(TermInput::Home);
        }
        Input::KeyEnd => {
            ev_input.send(TermInput::End);
        }
        Input::KeyIC => {
            ev_input.send(TermInput::Insert);
        }
        Input::KeyDC => {
            ev_input.send(TermInput::Delete);
        }
        Input::KeyPPage => {
            ev_input.send(TermInput::PageUp);
        }
        Input::KeyNPage => {
            ev_input.send(TermInput::PageDown);
        }
        Input::KeyF1 => {
            ev_input.send(TermInput::F1);
        }
        Input::KeyF2 => {
            ev_input.send(TermInput::F2);
        }
        Input::KeyF3 => {
            ev_input.send(TermInput::F3);
        }
        Input::KeyF4 => {
            ev_input.send(TermInput::F4);
        }
        Input::KeyF5 => {
            ev_input.send(TermInput::F5);
        }
        Input::KeyF6 => {
            ev_input.send(TermInput::F6);
        }
        Input::KeyF7 => {
            ev_input.send(TermInput::F7);
        }
        Input::KeyF8 => {
            ev_input.send(TermInput::F8);
        }
        Input::KeyF9 => {
            ev_input.send(TermInput::F9);
        }
        Input::KeyF10 => {
            ev_input.send(TermInput::F10);
        }
        Input::KeyF11 => {
            ev_input.send(TermInput::F11);
        }
        Input::KeyF12 => {
            ev_input.send(TermInput::F12);
        }
        _ => {
            warn!("Unknown input: {:?}", ev);
        }
    }
}
