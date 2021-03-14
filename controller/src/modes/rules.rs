use std::f32::consts::{self, TAU};

use crate::{
    drawutils::{self, node},
    Globals, Transition, HEX_HEIGHT, HEX_WIDTH, NODE_RADIUS, WINDOW_WIDTH,
};

use macroquad::prelude::*;
use mofang_engine::Node;

pub struct ModeRules;

impl ModeRules {
    pub fn update(&mut self, globals: &mut Globals) -> Transition {
        if is_key_pressed(KeyCode::Escape) {
            Transition::Pop
        } else {
            Transition::None
        }
    }

    pub fn draw(&self, globals: &Globals) {
        drawutils::center_text(
            globals,
            "Mofang's Garden",
            32,
            WINDOW_WIDTH / 2.0,
            HEX_HEIGHT / 2.0,
        );

        let text_size_p = (HEX_HEIGHT / 4.0) as u16;

        drawutils::text(
            globals,
            r"Your goal is to empty the board, leaving one Qi node. 
Select combinations of free nodes to remove and transform them.
Usually, nodes are only free if they have three or more contiguous empty neighbors.
Fortunately, spaces off the board count as empty.
",
            text_size_p,
            HEX_WIDTH / 2.0,
            HEX_HEIGHT,
        );

        // Draw pentagram
        let canvas = render_target((HEX_WIDTH * 3.2) as _, (HEX_HEIGHT * 3.2) as _);
        set_texture_filter(canvas.texture, FilterMode::Linear);
        let mut camera = Camera2D::from_display_rect(Rect::new(
            0.0,
            0.0,
            canvas.texture.width(),
            canvas.texture.height(),
        ));
        camera.render_target = Some(canvas);
        set_camera(camera);
        drawutils::pentagram(
            globals,
            canvas.texture.width() / 2.0,
            canvas.texture.height() / 2.0,
        );
        set_default_camera();
        draw_texture_ex(
            canvas.texture,
            HEX_WIDTH / 2.0,
            HEX_HEIGHT * 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(HEX_WIDTH * 2.0, HEX_WIDTH * 2.0)),
                flip_y: true, // it's backwards for some reason...
                ..Default::default()
            },
        );

        drawutils::text(
            globals,
            r"Elemental nodes match and destroy each 
other along the black lines.
They also match with Change to become 
the next element in the cycle (gray lines).",
            text_size_p,
            HEX_WIDTH * 2.6,
            HEX_HEIGHT * 2.4,
        );
    }
}
