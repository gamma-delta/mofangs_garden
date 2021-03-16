pub mod nodes;
pub use nodes::{Node, PartialResult};

use itertools::Itertools;

use hex2d::{Angle, Coordinate, Direction, Spin};

use std::collections::HashMap;

/// The hexagonal board the game is played on.
///
/// The board uses pointy-topped hexes.
/// Coordinates are stored as qr, with q increasing to the right
/// and r increasing to the down-right.
pub struct Board {
    nodes: HashMap<Coordinate, Option<Node>>,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    // https://www.drking.org.uk/hexagons/misc/numbers.html
    pub const DIAMETER: i32 = 11;
    pub const RADIUS: i32 = Board::DIAMETER / 2;
    pub const HEX_COUNT: i32 = 3 * Board::RADIUS * (Board::RADIUS + 1) + 1;

    /// Create a new empty grid
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        for coord in Coordinate::new(0, 0).range_iter(Board::RADIUS) {
            nodes.insert(coord, None);
        }

        Self { nodes }
    }

    /// Create a new empty grid and fill it with a random game.
    pub fn new_game() -> Self {
        let mut bank = Board::standard_game();
        fastrand::shuffle(&mut bank);

        let mut out = Self::new();
        out.nodes
            .insert(Coordinate::new(0, 0), Some(Node::Destruction));

        let mut try_insert = |coord, node, req_neighbor| {
            // Fail if:
            // - there's something here
            // - it's out of bounds
            let failure = !out.in_bounds(coord) || out.get_node(coord).is_some()
                // - there are no neighbors and we want some
                || (req_neighbor
                    && !coord
                        .neighbors()
                        .iter()
                        .any(|&c| out.get_node(c).is_some()))
                    // - this is qi and we have 2 neighbor qi
                    || (matches!(node, Node::Qi)
                        && Direction::all().iter().filter(|&&dir| {
                            matches!(out.get_node(coord + dir), Some(Node::Qi))
                        }).count() >= 2)
                /*
                    // - i am sandwiched by qi
                    || Direction::all().iter().take(3).any(|&dir| {
                        matches!(out.get_node(coord + dir), Some(Node::Qi))
                            && matches!(out.get_node(coord - dir), Some(Node::Qi))
                    })
                */
                ;
            if failure {
                Some(node)
            } else {
                out.nodes.insert(coord, Some(node));
                None
            }
        };

        // +1 => right
        // -1 => left
        let chirality = if fastrand::bool() { 1 } else { -1 };
        // make the non-spokes surrounding the center always be qi
        for idx in 0..3 {
            let pos = Coordinate::new(0, -chirality).rotate_around_zero(Angle::from_int(idx * 2));
            try_insert(pos, Node::Qi, false);
        }

        'outer: for radius in 1..=Board::RADIUS {
            let prob = if radius == 1 {
                1.0
            } else if radius % 2 == 1 {
                0.8
            } else {
                0.0
            };
            // try each ring this many times
            for _ in 0..3 {
                let mut ring = Coordinate::new(0i32, 0)
                    .ring_iter(radius, Spin::CW(Direction::XZ))
                    .collect_vec();
                fastrand::shuffle(&mut ring);
                for coord in ring {
                    let on_spoke = (coord.x == 0 && coord.y.signum() == chirality)
                        || (coord.y == 0 && coord.z().signum() == chirality)
                        || (coord.z() == 0 && coord.x.signum() == chirality);

                    if on_spoke && radius == 0 {
                        // Just place Qi
                        try_insert(coord, Node::Qi, false);
                    }
                    let prob = if on_spoke { 1.0 } else { prob };
                    if fastrand::f32() <= prob {
                        if let Some(node) = bank.pop() {
                            let neighbor_req = radius as f32 / ((Board::RADIUS - 1) as f32);
                            if let Some(failed_to_insert) =
                                try_insert(coord, node, fastrand::f32() <= neighbor_req)
                            {
                                bank.push(failed_to_insert);
                            }
                        } else {
                            break 'outer;
                        }
                    }
                }
            }
        }
        println!("remaining: {:?}", &bank);

        let coord_options = Coordinate::new(0, 0)
            .range_iter(Board::RADIUS)
            .collect_vec();
        while let Some(node) = bank.pop() {
            // TODO: Not sure how to do this nicely since `try_insert` eats `node`,
            // so I gave up and did it with a fold
            let result = (0..1000).try_fold(node, |node, _| {
                let rand_coord = coord_options[fastrand::usize(..coord_options.len())];
                try_insert(rand_coord, node, true)
            });
            if result.is_some() {
                // just give up and try again
                println!("giving up...");
                return Self::new_game();
            }
        }
        println!("remaining (must be empty): {:?}", &bank);

        out
    }

    /// Get the node at the given coordinate, or `None` if it's out of bounds or doesn't exist.
    pub fn get_node(&self, coord: Coordinate) -> Option<&Node> {
        self.nodes.get(&coord).map(Option::as_ref).flatten()
    }
    /// Check whether the given coordinate is on the grid.
    pub fn in_bounds(&self, coord: Coordinate) -> bool {
        self.nodes.get(&coord).is_some()
    }
    /// Set the node at the given spot. Return Some with the old value if something was clobbered.
    pub fn set_node(&mut self, coord: Coordinate, node: Option<Node>) -> Option<Node> {
        self.nodes.insert(coord, node).flatten()
    }
    /// Iterator through all slots on the board in no particular order.
    pub fn nodes_iter(&self) -> impl Iterator<Item = (Coordinate, Option<&Node>)> + '_ {
        Coordinate::new(0, 0)
            .range_iter(Board::RADIUS)
            .map(move |c| (c, self.get_node(c)))
    }

    /// Return the standard game sans 1 Destruction to go in the center
    /// and 3 qi to surround it.
    fn standard_game() -> Vec<Node> {
        let mut game = vec![];

        for element in &[
            Node::Wood,
            Node::Fire,
            Node::Earth,
            Node::Metal,
            Node::Water,
        ] {
            for _ in 0..7 {
                game.push(element.clone());
            }
        }

        for node in &[Node::Heavenly, Node::Earthly, Node::Human] {
            for _ in 0..4 {
                game.push(node.clone());
            }
        }

        game.push(Node::Yin);
        game.push(Node::Yang);

        for _ in 0..3 {
            game.push(Node::Qi);
        }

        game
    }
}
