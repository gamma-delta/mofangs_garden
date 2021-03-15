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
        Node::Creation => globals.assets.textures.creation,
        Node::Destruction => globals.assets.textures.destruction,
        Node::Qi => globals.assets.textures.qi,
    };

    if faded {
        gl_use_material(globals.assets.fade_shader);
    }

    draw_texture(
        tex,
        (cx - NODE_RADIUS).round(),
        (cy - NODE_RADIUS).round(),
        WHITE,
    );

    if faded {
        gl_use_default_material();
    }
}

pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Draw the text with the given size at the given position.
pub fn text(globals: &Globals, text: &str, size: u16, cx: f32, cy: f32, align: TextAlign) {
    let params = TextParams {
        font_size: size,
        font: globals.assets.font,
        color: BLACK,
        ..Default::default()
    };
    for (idx, line) in text.lines().enumerate() {
        let offset = match align {
            TextAlign::Left => 0.0,
            TextAlign::Center => 0.5,
            TextAlign::Right => 1.0,
        };
        let width = measure_text(line, Some(globals.assets.font), size, 1.).width;
        draw_text_ex(
            line,
            cx - offset * width,
            cy + size as f32 * idx as f32,
            params,
        );
    }
}

pub fn center_text(globals: &Globals, text: &str, size: u16, cx: f32, cy: f32) {
    let center_y = cy - size as f32 * (text.lines().count() as f32 - 5. / 3.) * 0.5;
    self::text(globals, text, size, cx, center_y, TextAlign::Center);
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

pub fn node_arrow(from: (f32, f32), to: (f32, f32), padding: f32, skew_angle: f32, color: Color) {
    let arrow_pad = (1. + padding) * NODE_RADIUS;
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let angle = dy.atan2(dx);
    // length is distance between centers minus twice the padding
    let len = (dx * dx + dy * dy).sqrt() - 2. * arrow_pad * skew_angle.cos();
    arrow(
        from.0 + arrow_pad * (skew_angle + angle).cos(),
        from.1 + arrow_pad * (skew_angle + angle).sin(),
        angle,
        len,
        2.0,
        NODE_RADIUS * 0.2,
        NODE_RADIUS * 0.2,
        color,
    );
}

pub fn draw_centered(tex: Texture2D, (x, y): (f32, f32)) {
    // we take the floor to align pixels perfectly
    draw_texture(
        tex,
        (x - tex.width() * 0.5).floor(),
        (y - tex.height() * 0.5).floor(),
        WHITE,
    );
}

pub fn pentagram<C>(globals: &Globals, pent_x: f32, pent_y: f32, mut continuation: C)
where
    C: FnMut(f32, f32, f32, Node) -> (),
{
    let offset = |angle: f32, rad| {
        let (dx, dy) = (angle * TAU).sin_cos();
        (pent_x + rad * dx, pent_y - rad * dy)
    };
    let node_pos = (0..5)
        .map(|idx| offset(idx as f32 * 0.2, HEX_HEIGHT * 1.2))
        .collect::<Vec<_>>();
    self::node(globals, Node::Destruction, pent_x, pent_y, false);
    continuation(pent_x, pent_y, TAU * 0.125, Node::Destruction);
    draw_poly_lines(pent_x, pent_y, 40, HEX_HEIGHT * 1.24, 0., 1.2, GRAY);
    draw_poly_lines(pent_x, pent_y, 40, HEX_HEIGHT * 1.3, 0., 1.2, GRAY);
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
        let pos = node_pos[idx];

        // Arrows and transmute annotation
        self::node_arrow(pos, node_pos[(idx + 1) % 5], 0.25, -0.3, GRAY);
        self::node_arrow(pos, node_pos[(idx + 2) % 5], 0.1, -0.25, BLACK);
        draw_centered(
            globals.assets.textures.create_base,
            offset(idx as f32 * 0.2 + 0.5, HEX_HEIGHT * 0.95),
        );

        self::node(globals, node, pos.0, pos.1, false);
        continuation(pos.0, pos.1, (0.25 - idx as f32 * 0.2) * TAU, node);
    }
}
