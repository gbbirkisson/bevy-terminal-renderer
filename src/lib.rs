use bevy::app::AppExit;
use bevy::prelude::*;
use tracing::warn;

use pancurses::{
    curs_set, endwin, getmouse, initscr, mousemask, nl, noecho, resize_term, Input, Window,
    ALL_MOUSE_EVENTS,
};

#[derive(Resource)]
struct Term {
    window: Window,
    wide: bool,
    minz: f32,
}

// SAFETY: Window cannot be passed between threads, but there is only ever 1 thread that uses it at
// a time. We just have to be careful with our system ordering. I have no idea if this is a good
// idea or not, but it works on my machine :D
unsafe impl Send for Term {}
unsafe impl Sync for Term {}

#[derive(Resource)]
struct TermContext {
    wide: bool,
    minz: f32,
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
    pub transform: Transform,
}

impl Default for TermSpriteBundle {
    fn default() -> Self {
        Self {
            char: TermChar('?'),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
        }
    }
}

#[derive(Bundle)]
pub struct TermTextBundle {
    pub text: TermText,
    pub align: TermTextAlign,
    pub transform: Transform,
}

impl Default for TermTextBundle {
    fn default() -> Self {
        Self {
            text: TermText::from("?"),
            align: TermTextAlign::CENTER,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
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
    pub transform: Transform,
}

impl Default for TermCameraBundle {
    fn default() -> Self {
        Self {
            camera: TermCamera,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
        }
    }
}

#[derive(Debug, Event)]
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

#[derive(Event)]
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
        .add_systems(Startup, create_terminal)
        .add_systems(PreUpdate, handle_terminal_events)
        .add_systems(
            PostUpdate,
            (
                handle_terminal_draw,
                handle_terminal_commands.after(handle_terminal_draw),
            ),
        );
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
    commands.insert_resource(Term {
        window,
        wide: context.wide,
        minz: context.minz,
    });
}

fn handle_terminal_events(
    terminal: Res<Term>,
    mut terminal_size: Query<&mut TermSize>,
    mut ev_input: EventWriter<TermInput>,
) {
    let mut resize = false;

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
        terminal.window.erase();
    }

    let (c, r) = get_window_size(&terminal.window, terminal.wide);

    // Update terminal size
    let mut terminal_size = terminal_size
        .single_mut()
        .expect("We should always have a terminal");
    terminal_size.0 = c;
    terminal_size.1 = r;
}

fn handle_terminal_commands(mut ev_cmd: EventReader<TermCommand>, mut exit: EventWriter<AppExit>) {
    // Look for commands for the terminal
    if let Some(ev) = ev_cmd.read().next() {
        match ev {
            TermCommand::Exit => {
                curs_set(1);
                endwin();
                exit.write(AppExit::Success);
            }
        }
    }
}

fn handle_terminal_draw(
    terminal: Res<Term>,
    terminal_size: Query<&TermSize>,
    camera: Query<&GlobalTransform, With<TermCamera>>,
    chars: Query<(&GlobalTransform, &TermChar)>,
    texts: Query<(&GlobalTransform, &TermText, &TermTextAlign)>,
) {
    // Get our data
    let terminal = terminal.into_inner();
    let terminal_size = terminal_size
        .single()
        .expect("We should always have a terminal");

    // Prepare drawing
    let c = terminal_size.0 as usize;
    let r = terminal_size.1 as usize;
    let mut buffer = vec![vec![(' ', terminal.minz); r]; c];

    // Calculate camera offset
    let (camera_offset_x, camera_offset_y) = match camera.single() {
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

        if terminal.wide {
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
            terminal
                .window
                .mvaddstr(y as i32, x as i32, format!("{}", buffer[x][y].0));
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
            ev_input.write(TermInput::BackSpace);
        }
        Input::Character(' ') => {
            ev_input.write(TermInput::SpaceBar);
        }
        Input::Character('\n') => {
            ev_input.write(TermInput::Enter);
        }
        Input::Character('\t') => {
            ev_input.write(TermInput::Tab);
        }
        Input::Character('\u{1b}') => {
            ev_input.write(TermInput::Escape);
        }
        Input::Character(c) => {
            ev_input.write(TermInput::Character(c));
        }
        Input::KeyMouse => {
            if let Ok(mouse_event) = getmouse() {
                ev_input.write(TermInput::Mouse(mouse_event.x, mouse_event.y));
            };
        }
        Input::KeyLeft => {
            ev_input.write(TermInput::Left);
        }
        Input::KeyRight => {
            ev_input.write(TermInput::Right);
        }
        Input::KeyUp => {
            ev_input.write(TermInput::Up);
        }
        Input::KeyDown => {
            ev_input.write(TermInput::Down);
        }
        Input::KeyHome => {
            ev_input.write(TermInput::Home);
        }
        Input::KeyEnd => {
            ev_input.write(TermInput::End);
        }
        Input::KeyIC => {
            ev_input.write(TermInput::Insert);
        }
        Input::KeyDC => {
            ev_input.write(TermInput::Delete);
        }
        Input::KeyPPage => {
            ev_input.write(TermInput::PageUp);
        }
        Input::KeyNPage => {
            ev_input.write(TermInput::PageDown);
        }
        Input::KeyF1 => {
            ev_input.write(TermInput::F1);
        }
        Input::KeyF2 => {
            ev_input.write(TermInput::F2);
        }
        Input::KeyF3 => {
            ev_input.write(TermInput::F3);
        }
        Input::KeyF4 => {
            ev_input.write(TermInput::F4);
        }
        Input::KeyF5 => {
            ev_input.write(TermInput::F5);
        }
        Input::KeyF6 => {
            ev_input.write(TermInput::F6);
        }
        Input::KeyF7 => {
            ev_input.write(TermInput::F7);
        }
        Input::KeyF8 => {
            ev_input.write(TermInput::F8);
        }
        Input::KeyF9 => {
            ev_input.write(TermInput::F9);
        }
        Input::KeyF10 => {
            ev_input.write(TermInput::F10);
        }
        Input::KeyF11 => {
            ev_input.write(TermInput::F11);
        }
        Input::KeyF12 => {
            ev_input.write(TermInput::F12);
        }
        _ => {
            warn!("Unknown input: {:?}", ev);
        }
    }
}
