use super::render::RenderPlugin;
use super::term::TermDrawPlugin;
use bevy::prelude::*;

pub struct TermPlugin {
    pub minz: f32,
}

impl Default for TermPlugin {
    fn default() -> Self {
        Self { minz: f32::MIN }
    }
}

impl Plugin for TermPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TermDrawPlugin {}, RenderPlugin { minz: self.minz }));
    }
}
