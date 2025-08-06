use bevy::prelude::*;

use crate::prelude::{TermCamera, TermChar, TermText, TermTextAlign};
use crate::term::TermBuffer;

#[derive(Resource)]
struct RenderContext {
    minz: f32,
}

pub(crate) struct RenderPlugin {
    pub minz: f32,
}

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RenderContext { minz: self.minz })
            .add_systems(PostUpdate, render);
    }
}

// fn debug_render(terminal_buffer: ResMut<TermBuffer>) {
//     let terminal_buffer = terminal_buffer.into_inner();
//     let mut terminal_buffer = terminal_buffer.wide();
//     let (c, r) = terminal_buffer.size();
//     warn!("{c}, {r}");
//     terminal_buffer.write(0, 0, 'a', 0.0);
//     terminal_buffer.write(c - 1, 0, 'b', 0.0);
//     terminal_buffer.write(0, r - 1, 'c', 0.0);
//     terminal_buffer.write(c - 1, r - 1, 'd', 0.0);
// }

fn render(
    context: Res<RenderContext>,
    terminal_buffer: ResMut<TermBuffer>,
    camera: Query<&GlobalTransform, With<TermCamera>>,
    chars: Query<(&GlobalTransform, &TermChar)>,
    texts: Query<(&GlobalTransform, &TermText, &TermTextAlign)>,
) {
    // Prepare drawing
    let ctx = context.into_inner();
    let terminal_buffer = terminal_buffer.into_inner();
    let (c, r) = terminal_buffer.size();

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
        let x = transform.translation().x.floor() as isize;
        let y = transform.translation().y.floor() as isize;
        let z = transform.translation().z;

        let x = x + camera_offset_x;
        let y = -y + camera_offset_y;

        if x < 0 || y < 0 {
            // This char is not in view
            continue;
        }

        let x = x as usize;
        let y = y as usize;

        if x < c && y < r && z > ctx.minz {
            terminal_buffer.write(x, y, char.0, z);
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

            if x < c && y < r && z > ctx.minz {
                terminal_buffer.write(x, y, char, z);
            }
        }
    }

    terminal_buffer.prune()
}
