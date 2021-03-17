use enum_map::EnumMap;
use hex2d::{Coordinate, Spacing};
use macroquad::prelude::*;
use mofang_engine::{Board, Node, PartialResult};
use mofang_games::MofangNode;

use crate::{drawutils, Globals, Mode, Transition, HEX_HEIGHT, HEX_SIZE, HEX_WIDTH, NODE_RADIUS};

use super::rules::ModeRules;

pub struct ModeGame {
    board: Board<MofangNode>,
    hovered_slot: Option<Coordinate>,
    selected_slots: Vec<Coordinate>,
    node_count: EnumMap<MofangNode, u32>,

    won: bool,
}

impl ModeGame {
    pub fn new_game() -> Self {
        let mut this = Self {
            board: MofangNode::new_game_random(),
            hovered_slot: None,
            selected_slots: Vec::new(),
            node_count: EnumMap::new(),

            won: false,
        };
        this.update_node_count();
        this
    }

    pub fn update(&mut self, _globals: &mut Globals) -> Transition {
        if self.won {
            // Forbid interacting with the board
            self.hovered_slot = None;
            self.selected_slots.clear();
        } else if is_key_pressed(KeyCode::H) {
            return Transition::Push(Mode::Rules(ModeRules));
        }

        let mouse_raw = mouse_position();
        if is_mouse_button_released(MouseButton::Left)
            && new_game_button().contains(mouse_raw.into())
        {
            return Transition::Swap(Mode::Game(ModeGame::new_game()));
        } else if self.won {
            return Transition::None;
        }

        let dmouse_x = mouse_raw.0 - self.board_origin_x();
        let dmouse_y = mouse_raw.1 - self.board_origin_y();

        let hovered_coord =
            Coordinate::from_pixel(dmouse_x, dmouse_y, Spacing::PointyTop(HEX_SIZE));
        self.hovered_slot = Some(hovered_coord).filter(|c| self.board.in_bounds(*c));

        if let Some(hovered) = self.hovered_slot {
            if is_mouse_button_released(MouseButton::Left) {
                // hey we're trying to select something!
                if let Some(idx) = self.selected_slots.iter().position(|c| c == &hovered) {
                    // Oops we know this already
                    if idx == self.selected_slots.len() - 1 {
                        self.selected_slots.pop();
                    } else {
                        self.selected_slots.clear();
                    }
                } else if self.is_selectable(hovered) {
                    self.selected_slots.push(hovered);
                    // See if we have a WOMBO COMBO
                    let combo: Vec<_> = self
                        .selected_slots
                        .iter()
                        .flat_map(|&c| self.board.get_node(c))
                        .collect();
                    if let PartialResult::Success(change) = Node::select(&combo) {
                        // nice!
                        for (update, &slot) in change.into_iter().zip(self.selected_slots.iter()) {
                            self.board.set_node(slot, update);
                        }
                        self.selected_slots.clear();
                        self.update_node_count();

                        if self.node_count.iter().all(|(_node, &count)| count == 0) {
                            // poggers
                            self.won = true;
                        }
                    }
                } else if self.board.get_node(hovered).is_none() {
                    // Click off a piece to clear your selection
                    self.selected_slots.clear();
                }
            }
        }

        Transition::None
    }

    pub fn draw(&self, globals: &Globals) {
        // Draw counting UI
        let ui_center_x = screen_width() - HEX_WIDTH * 2.3;

        let pent_x = ui_center_x;
        let pent_y = HEX_WIDTH * 2.5;
        let mouse_pos = mouse_position();
        let mut hovered_node = None;
        drawutils::pentagram(globals, pent_x, pent_y, |x, y, angle, node| {
            let (dx, dy) = (mouse_pos.0 - x, mouse_pos.1 - y);
            if dx * dx + dy * dy < NODE_RADIUS * NODE_RADIUS {
                hovered_node = Some(node.clone());
            }

            let count = self.node_count[node];
            let (dy, dx) = angle.sin_cos();
            let (x, y) = (x + dx * NODE_RADIUS, y - dy * NODE_RADIUS);
            draw_circle(x, y, NODE_RADIUS * 0.3, WHITE);
            draw_circle_lines(x, y, NODE_RADIUS * 0.3, 1.2, BLACK);
            drawutils::center_text(globals, count.to_string().as_str(), 14, x, y);
        });

        // Draw new game button
        let new_game_button = new_game_button();
        draw_rectangle_lines(
            new_game_button.x,
            new_game_button.y,
            new_game_button.w,
            new_game_button.h,
            2.0,
            BLACK,
        );
        drawutils::center_text(
            globals,
            "New Game",
            20,
            new_game_button.x + new_game_button.w / 2.0,
            new_game_button.y + new_game_button.h / 2.0,
        );

        // Draw board
        for hex_coord in Coordinate::new(0, 0).range_iter(self.board.radius()) {
            let zero_coords = hex_coord.to_pixel(Spacing::PointyTop(HEX_SIZE));
            let coords = (
                zero_coords.0 + self.board_origin_x(),
                zero_coords.1 + self.board_origin_y(),
            );
            draw_texture(
                globals.assets.textures.hex,
                coords.0 - HEX_WIDTH / 2.0,
                coords.1 - HEX_HEIGHT / 2.0,
                WHITE,
            );

            let unfaded_node = if let Some(node) = self.board.get_node(hex_coord) {
                let unfaded = match hovered_node {
                    Some(ref hnode) => *hnode == *node,
                    None => self.is_selectable(hex_coord),
                };
                drawutils::node(globals, node, coords.0, coords.1, !unfaded);
                unfaded
            } else {
                false
            };

            let center_x = coords.0 - NODE_RADIUS;
            let center_y = coords.1 - NODE_RADIUS;

            if self.selected_slots.contains(&hex_coord) {
                draw_texture(globals.assets.textures.select, center_x, center_y, WHITE);
            } else if self.hovered_slot == Some(hex_coord) && unfaded_node {
                draw_texture(globals.assets.textures.highlight, center_x, center_y, WHITE);
            }

            if is_key_down(KeyCode::LeftShift) {
                let open_count = self.board.max_open_neighbors(&hex_coord);
                draw_text(
                    open_count.to_string().as_str(),
                    coords.0,
                    coords.1,
                    25.0,
                    LIME,
                );
            }
        }
    }

    /// TODO: This function should be part of MofangNode.
    /// We shouldn't trust the controller to do stuff like this.
    fn is_selectable(&self, coord: Coordinate) -> bool {
        let node = match self.board.get_node(coord) {
            Some(it) => it,
            None => return false,
        };

        node.can_select(&self.board, &coord, self.selected_slots.as_slice()) && {
            // check to see if this is an allowed combo
            let potential_select: Vec<_> = self
                .selected_slots
                .iter()
                .flat_map(|c| self.board.get_node(*c))
                .chain(Some(node))
                .collect();
            Node::select(&potential_select).is_valid()
        }
    }

    fn update_node_count(&mut self) {
        self.node_count.clear();
        for node in self.board.nodes_iter().flat_map(|(_, node)| node) {
            self.node_count[node.clone()] += 1;
        }
    }

    fn board_origin_x(&self) -> f32 {
        (self.board.radius() + 1) as f32 * HEX_WIDTH
    }
    fn board_origin_y(&self) -> f32 {
        (self.board.radius() + 1) as f32 * HEX_HEIGHT * 0.75
    }
}

fn new_game_button() -> Rect {
    let ui_center_x = screen_width() - HEX_WIDTH * 2.3;
    Rect::new(
        ui_center_x - HEX_WIDTH * 1.5,
        HEX_HEIGHT * 7.0,
        HEX_WIDTH * 3.0,
        HEX_HEIGHT,
    )
}
