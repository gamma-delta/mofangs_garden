pub mod nodes;
pub use nodes::{Node, PartialResult};

use hex2d::Coordinate;

use std::collections::HashMap;

/// The hexagonal board the game is played on.
///
/// The board uses pointy-topped hexes.
/// Coordinates are stored as qr, with q increasing to the right
/// and r increasing to the down-right.
///
/// Members of this struct are public only for the benefit of people making games.
/// If you are making a controller, please don't access these and instead use the methods.
pub struct Board<N: Node> {
    pub nodes: HashMap<Coordinate, Option<N>>,
    pub diameter: i32,
}

impl<N: Node> Board<N> {
    // https://www.drking.org.uk/hexagons/misc/numbers.html
    pub fn radius(&self) -> i32 {
        self.diameter / 2
    }
    pub fn hex_count(&self) -> i32 {
        3 * self.radius() * (self.radius() + 1) + 1
    }
    pub fn diameter(&self) -> i32 {
        self.diameter
    }

    /// Create a new empty board.
    pub fn new(diameter: i32) -> Self {
        let mut nodes = HashMap::new();
        for coord in Coordinate::new(0, 0).range_iter(diameter / 2) {
            nodes.insert(coord, None);
        }

        Self { nodes, diameter }
    }

    /// Get the node at the given coordinate, or `None` if it's out of bounds or doesn't exist.
    pub fn get_node(&self, coord: Coordinate) -> Option<&N> {
        self.nodes.get(&coord).map(Option::as_ref).flatten()
    }
    /// Check whether the given coordinate is on the grid.
    pub fn in_bounds(&self, coord: Coordinate) -> bool {
        self.nodes.get(&coord).is_some()
    }
    /// Set the node at the given spot. Return Some with the old value if something was clobbered.
    pub fn set_node(&mut self, coord: Coordinate, node: Option<N>) -> Option<N> {
        self.nodes.insert(coord, node).flatten()
    }
    /// Iterator through all slots on the board in no particular order.
    pub fn nodes_iter(&self) -> impl Iterator<Item = (Coordinate, Option<&N>)> + '_ {
        Coordinate::new(0, 0)
            .range_iter(self.radius())
            .map(move |c| (c, self.get_node(c)))
    }

    /// Convenience method:
    /// How many open neighbors are there around the coord?
    pub fn max_open_neighbors(&self, at: &Coordinate) -> usize {
        match at
            .neighbors()
            .iter()
            .position(|&coord| self.get_node(coord).is_some())
        {
            Some(pos) => {
                // At least one neighbor exists, iter around it
                at.neighbors()
                    .iter()
                    .cycle()
                    .skip(pos + 1)
                    .take(6)
                    .fold((0, 0), |(maxrun, run), &neighbor| {
                        if self.get_node(neighbor).is_some() {
                            (maxrun.max(run), 0)
                        } else {
                            (maxrun, run + 1)
                        }
                    })
                    .0
            }
            // Everything is empty and fine
            None => 6,
        }
    }
}

/// Convenient helper function for your game implementations.
pub fn all_unique<I, E>(mut iter: I) -> bool
where
    I: Iterator<Item = E>,
    E: PartialEq,
{
    iter.next()
        .map(|val| {
            iter.try_fold(val, |acc, next| if acc == next { None } else { Some(next) })
                .is_some()
        })
        .unwrap_or(true)
}
