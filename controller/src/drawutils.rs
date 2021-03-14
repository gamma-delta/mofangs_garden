use std::f32::consts::TAU;

use mofang_engine::Node;

use macroquad::prelude::*;

use crate::{Globals, HEX_HEIGHT, NODE_RADIUS};

/// Draw the Node centered at that position.
pub fn node(globals: &Globals, node: Node, cx: f32, cy: f32, faded: bool) {
    let tex = match node {
        Node::Wood => globals.assets.textures.wood,
        Node::Fire => globals.assets.textures.fire,
        Node::Earth => globals.assets.textures.earth,
        Node::Metal => globals.assets.textures.metal,
        Node::Water => globals.assets.textures.water,
        Node::Heavenly => globals.assets.textures.heavenly,
        Node::Earthly => globals.assets.textures.earthly,
        Node::Human => globals.assets.textures.human,
        Node::Yin => globals.assets.textures.yin,
        Node::Yang => globals.assets.textures.yang,
        Node::Change => globals.assets.textures.change,
        Node::Qi => globals.assets.textures.qi,
    };

    draw_texture(
        tex,
        cx - NODE_RADIUS,
        cy - NODE_RADIUS,
        if faded {
            Color::new(1.0, 1.0, 1.0, 0.3)
        } else {
            WHITE
        },
    );
}

/// Draw the text with the given size at the given position.
pub fn text(globals: &Globals, text: &str, size: u16, cx: f32, cy: f32) {
    for (idx, line) in text.lines().enumerate() {
        draw_text_ex(
            line,
            cx,
            cy + size as f32 * idx as f32 * 1.1,
            TextParams {
                font_size: size,
                font: globals.assets.font,
                color: BLACK,
                ..Default::default()
            },
        );
    }
}

pub fn center_text(globals: &Globals, text: &str, size: u16, cx: f32, cy: f32) {
    // Estimate center
    let center_x = cx - text.len() as f32 / 2.0 * size as f32 * 0.5;
    let center_y = cy + size as f32 * 0.5;
    self::text(globals, text, size, center_x, center_y);
}

// shut up
#[allow(clippy::too_many_arguments)]
pub fn arrow(
    cx: f32,
    cy: f32,
    angle: f32,
    length: f32,
    width: f32,
    tip_width: f32,
    tip_length: f32,
    color: Color,
) {
    let line_start = vec2(0.0, 0.0);
    let line_end = vec2(length - tip_length, 0.0);
    let point_end = vec2(length, 0.0);
    let point_top = vec2(length - tip_length, -tip_width / 2.0);
    let point_bottom = vec2(length - tip_length, tip_width / 2.0);

    // linear algebra is pogchamp
    let t = Mat3::from_scale_angle_translation(vec2(1.0, 1.0), angle, vec2(cx, cy));
    let line_start = t.transform_point2(line_start);
    let line_end = t.transform_point2(line_end);
    let point_end = t.transform_point2(point_end);
    let point_top = t.transform_point2(point_top);
    let point_bottom = t.transform_point2(point_bottom);

    draw_line(
        line_start.x,
        line_start.y,
        line_end.x,
        line_end.y,
        width,
        color,
    );
    draw_triangle(point_end, point_top, point_bottom, color);
}

pub fn pentagram(globals: &Globals, pent_x: f32, pent_y: f32) {
    for (idx, &node) in [
        Node::Wood,
        Node::Fire,
        Node::Earth,
        Node::Metal,
        Node::Water,
    ]
    .iter()
    .enumerate()
    {
        let wrap = idx as f32 / 5.0;
        let angle = wrap * TAU;
        let theta = angle - TAU / 4.0;

        let (dy, dx) = theta.sin_cos();
        let radius = HEX_HEIGHT;
        let x = dx * radius + pent_x;
        let y = dy * radius + pent_y;
        self::node(globals, node, x, y, false);

        // Draw the two arrows.
        let arrow_origin_x = dx * radius * 1.1 + pent_x;
        let arrow_origin_y = dy * radius * 1.1 + pent_y;
        let arrow_pad = 1.2;

        let short_angle = 0.1 * TAU + angle;
        let (dy, dx) = short_angle.sin_cos();
        let arrow_x = arrow_origin_x + dx * NODE_RADIUS * arrow_pad;
        let arrow_y = arrow_origin_y + dy * NODE_RADIUS * arrow_pad;
        let side = 1.176 * radius;
        let len = side - NODE_RADIUS * arrow_pad * 1.8;
        arrow(
            arrow_x,
            arrow_y,
            short_angle,
            len,
            2.0,
            NODE_RADIUS * 0.2,
            NODE_RADIUS * 0.2,
            GRAY,
        );

        let long_angle = 0.2 * TAU + angle;
        let (dy, dx) = long_angle.sin_cos();
        let arrow_pad = 1.3;
        let arrow_x = arrow_origin_x + dx * NODE_RADIUS * arrow_pad;
        let arrow_y = arrow_origin_y + dy * NODE_RADIUS * arrow_pad;
        let span = 1.618 * radius;
        let len = span - NODE_RADIUS * arrow_pad * 1.1;
        arrow(
            arrow_x,
            arrow_y,
            long_angle,
            len,
            2.0,
            NODE_RADIUS * 0.2,
            NODE_RADIUS * 0.2,
            BLACK,
        );
    }
}
