use super::prelude::{TermCommand, TermInput};
use bevy::prelude::*;
use pancurses::{
    curs_set, endwin, getmouse, initscr, mousemask, nl, noecho, resize_term, Input, Window,
    ALL_MOUSE_EVENTS,
};
use tracing::warn;

const CLEAR: (char, f32) = (' ', f32::MIN);

#[derive(Resource)]
pub(crate) struct TermBuffer {
    buffer: Vec<Vec<(char, f32)>>,
    c: usize,
    r: usize,
}

impl TermBuffer {
    fn new(c: i32, r: i32) -> Self {
        Self {
            buffer: vec![vec![CLEAR; c as usize]; r as usize],
            c: c as usize,
            r: r as usize,
        }
    }

    pub(crate) fn size(&self) -> (usize, usize) {
        (self.c, self.r)
    }

    pub(crate) fn write(&mut self, c: usize, r: usize, v: char, z: f32) {
        if z > self.buffer[r][c].1 {
            self.buffer[r][c] = (v, z)
        }
    }

    pub(crate) fn prune(&mut self) {
        // TODO: Prune buffer so it does not overflow if sum of char.len_utf8() is greater than
        // buffer length
    }
}

#[derive(Resource)]
struct Term {
    window: Window,
}

// SAFETY: Window cannot be passed between threads, but there is only ever 1 thread that uses it at
// a time. We just have to be careful with our system ordering. I have no idea if this is a good
// idea or not, but it works on my machine :D
unsafe impl Send for Term {}
unsafe impl Sync for Term {}

pub(crate) struct TermDrawPlugin {}

impl Plugin for TermDrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TermInput>()
            .add_event::<TermCommand>()
            .add_systems(Startup, term_create)
            .add_systems(PreUpdate, term_events)
            .add_systems(Last, (term_draw, term_commands.after(term_draw)));
    }
}

fn term_create(mut commands: Commands) {
    let window = initscr();

    // Configure window
    nl();
    noecho();
    curs_set(0);
    window.timeout(0);
    window.keypad(true);
    mousemask(ALL_MOUSE_EVENTS, None);

    // Create components
    commands.insert_resource(TermBuffer::new(window.get_max_x(), window.get_max_y()));
    commands.insert_resource(Term { window });
}

fn term_events(
    terminal: Res<Term>,
    terminal_buffer: ResMut<TermBuffer>,
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
        let terminal_buffer = terminal_buffer.into_inner();
        *terminal_buffer =
            TermBuffer::new(terminal.window.get_max_x(), terminal.window.get_max_y());
    }
}

fn term_draw(terminal: Res<Term>, terminal_buffer: ResMut<TermBuffer>) {
    let terminal_buffer = terminal_buffer.into_inner();
    for (r, v) in terminal_buffer.buffer.iter_mut().enumerate() {
        for (c, v) in v.iter_mut().enumerate() {
            // Using the string method here to handle emojis
            terminal
                .window
                .mvaddstr(r as i32, c as i32, format!("{}", v.0));

            // Clear the char after write
            *v = CLEAR;
        }
    }
}

fn term_commands(mut ev_cmd: EventReader<TermCommand>, mut exit: EventWriter<AppExit>) {
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

fn map_and_send_events(ev: Input, ev_input: &mut EventWriter<TermInput>) {
    if let Some(ev) = match ev {
        Input::KeyBackspace => Some(TermInput::BackSpace),
        Input::Character(' ') => Some(TermInput::SpaceBar),
        Input::Character('\n') => Some(TermInput::Enter),
        Input::Character('\t') => Some(TermInput::Tab),
        Input::Character('\u{1b}') => Some(TermInput::Escape),
        Input::Character(c) => Some(TermInput::Character(c)),
        Input::KeyLeft => Some(TermInput::Left),
        Input::KeyRight => Some(TermInput::Right),
        Input::KeyUp => Some(TermInput::Up),
        Input::KeyDown => Some(TermInput::Down),
        Input::KeyHome => Some(TermInput::Home),
        Input::KeyEnd => Some(TermInput::End),
        Input::KeyIC => Some(TermInput::Insert),
        Input::KeyDC => Some(TermInput::Delete),
        Input::KeyPPage => Some(TermInput::PageUp),
        Input::KeyNPage => Some(TermInput::PageDown),
        Input::KeyF1 => Some(TermInput::F1),
        Input::KeyF2 => Some(TermInput::F2),
        Input::KeyF3 => Some(TermInput::F3),
        Input::KeyF4 => Some(TermInput::F4),
        Input::KeyF5 => Some(TermInput::F5),
        Input::KeyF6 => Some(TermInput::F6),
        Input::KeyF7 => Some(TermInput::F7),
        Input::KeyF8 => Some(TermInput::F8),
        Input::KeyF9 => Some(TermInput::F9),
        Input::KeyF10 => Some(TermInput::F10),
        Input::KeyF11 => Some(TermInput::F11),
        Input::KeyF12 => Some(TermInput::F12),
        Input::KeyMouse => match getmouse() {
            Ok(m_ev) => Some(TermInput::Mouse(m_ev.x, m_ev.y)),
            Err(e) => {
                warn!("Failed getting mouse event: {}", e);
                None
            }
        },
        _ => {
            warn!("Unknown input: {:?}", ev);
            None
        }
    } {
        ev_input.write(ev);
    }
}
