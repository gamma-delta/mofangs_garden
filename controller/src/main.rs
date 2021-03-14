mod assets;
mod drawutils;
mod modes;

use assets::Assets;
use macroquad::prelude::*;
use modes::{game::ModeGame, rules::ModeRules};
use mofang_engine::Board;

const HEX_SIZE: f32 = 40.0;
const HEX_WIDTH: f32 = HEX_SIZE * 1.732_050_8; // sqrt(3)
const HEX_HEIGHT: f32 = HEX_SIZE * 2.0;
const NODE_RADIUS: f32 = 32.0;

const WINDOW_WIDTH: f32 = HEX_WIDTH * (Board::DIAMETER + 5) as f32;
const WINDOW_HEIGHT: f32 = HEX_HEIGHT * (Board::DIAMETER + 1) as f32 * 0.75;

fn window_conf() -> Conf {
    Conf {
        window_title: String::from("Mofang's Garden"),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut mode_stack = vec![Mode::Game(ModeGame::new_game())];
    let mut globals = Globals {
        assets: Assets::init().await,
    };

    loop {
        clear_background(WHITE);

        let transition = match mode_stack.last_mut().unwrap() {
            Mode::Game(game) => game.update(&mut globals),

            Mode::Rules(rules) => rules.update(&mut globals),
        };
        match transition {
            Transition::None => {}
            Transition::Push(m) => mode_stack.push(m),
            Transition::Pop => {
                mode_stack.pop();
            }
            Transition::Swap(m) => *mode_stack.last_mut().unwrap() = m,
        }

        match mode_stack.last().unwrap() {
            Mode::Game(game) => game.draw(&globals),
            Mode::Rules(rules) => rules.draw(&globals),
        }

        next_frame().await
    }
}

pub enum Mode {
    Game(ModeGame),
    Rules(ModeRules),
}

pub struct Globals {
    assets: Assets,
}

pub enum Transition {
    None,
    Push(Mode),
    Pop,
    Swap(Mode),
}
