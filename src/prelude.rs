use bevy::prelude::*;

pub use super::plugin::TermPlugin;

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
