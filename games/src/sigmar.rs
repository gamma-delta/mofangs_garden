use enum_map::Enum;
use hex2d::{Coordinate, Direction, Spin};
use itertools::Itertools;
use mofang_engine::{all_unique, Board, Node, PartialResult};

/// One of the marbles on the game board.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Enum)]
pub enum SigmarNode {
    Salt,
    Aether,
    // ATLA order haha yes
    Water,
    Earth,
    Fire,
    Air,

    Quicksilver,
    Lead,
    Tin,
    Iron,
    Copper,
    Silver,
    Gold,

    Vitae,
    Mors,
}

impl Node for SigmarNode {
    fn name() -> &'static str {
        "sigmar"
    }
    fn texture_name(&self) -> &'static str {
        match self {
            SigmarNode::Salt => "salt",
            SigmarNode::Aether => "aether",
            SigmarNode::Water => "water",
            SigmarNode::Earth => "earth",
            SigmarNode::Fire => "fire",
            SigmarNode::Air => "air",

            SigmarNode::Quicksilver => "quicksilver",
            SigmarNode::Lead => "lead",
            SigmarNode::Tin => "tin",
            SigmarNode::Iron => "iron",
            SigmarNode::Copper => "copper",
            SigmarNode::Silver => "silver",
            SigmarNode::Gold => "gold",

            SigmarNode::Vitae => "vitae",
            SigmarNode::Mors => "mors",
        }
    }
    fn can_select(
        &self,
        board: &Board<SigmarNode>,
        coord: &Coordinate,
        _selected: &[Coordinate],
    ) -> bool {
        board.max_open_neighbors(coord) >= 3 && self.downgrade().and_then(|d| board.nodes_iter().find(|(_, n)| *n == Some(&d))).is_none()
    }

    /// Given a list of Nodes, see whether this pattern could exist
    /// and, if so, what to replace each Node with.
    fn select(nodes: &[&SigmarNode]) -> PartialResult<Vec<Option<SigmarNode>>> {
        match nodes.iter().sorted_unstable().as_slice() {
            [] => unreachable!("You can't select 0 nodes!"),
            [_] => PartialResult::Continue,
            [left, right] if left.cancels_with(right) => PartialResult::Success(vec![None, None]),
            [SigmarNode::Aether, rest @ ..]
                if rest.iter().all(|&n| n.is_prime()) && all_unique(rest.iter()) => {
                    if rest.len() == 4 {
                        PartialResult::Success(vec![None; 5])
                    } else {
                        PartialResult::Continue
                    }
                }
            _ => PartialResult::Failure,
        }
    }

    fn new_game(seed: u64) -> Board<SigmarNode> {
        let radius = 11;
        let rand = fastrand::Rng::new();

        let mut bank = Self::standard_game(false);
        rand.shuffle(&mut bank);

        let mut out = Board::new(11);
        out.nodes
            .insert(Coordinate::new(0, 0), Some(SigmarNode::Gold));

        let mut try_insert = |coord, node, req_neighbor| {
            // Fail if:
            // - there's something here
            // - it's out of bounds
            let failure = !out.in_bounds(coord) || out.get_node(coord).is_some()
                || (req_neighbor
                    && !coord
                        .neighbors()
                        .iter()
                        .any(|&c| out.get_node(c).is_some()));
            if failure {
                Some(node)
            } else {
                out.nodes.insert(coord, Some(node));
                None
            }
        };

        // +1 => right
        // -1 => left
        let chirality = if rand.bool() { 1 } else { -1 };
        'outer: for radius in 1..=radius {
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
                rand.shuffle(&mut ring);
                for coord in ring {
                    let on_spoke = (coord.x == 0 && coord.y.signum() == chirality)
                        || (coord.y == 0 && coord.z().signum() == chirality)
                        || (coord.z() == 0 && coord.x.signum() == chirality);

                    let prob = if on_spoke { 1.0 } else { prob };
                    if rand.f32() <= prob {
                        if let Some(node) = bank.pop() {
                            let neighbor_req = radius as f32 / ((radius - 1) as f32);
                            if let Some(failed_to_insert) =
                                try_insert(coord, node, rand.f32() <= neighbor_req)
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

        let coord_options = Coordinate::new(0, 0).range_iter(radius).collect_vec();
        while let Some(node) = bank.pop() {
            // TODO: Not sure how to do this nicely since `try_insert` eats `node`,
            // so I gave up and did it with a fold
            let result = (0..1000).try_fold(node, |node, _| {
                let rand_coord = coord_options[rand.usize(..coord_options.len())];
                try_insert(rand_coord, node, true)
            });
            if result.is_some() {
                // just give up and try again
                println!("giving up...");
                return Self::new_game(seed);
            }
        }
        println!("remaining (must be empty): {:?}", &bank);

        out
    }
}

impl SigmarNode {
    pub fn is_prime(&self) -> bool {
        matches!(
            self,
            SigmarNode::Water
                | SigmarNode::Earth
                | SigmarNode::Fire
                | SigmarNode::Air
        )
    }
    pub fn downgrade(&self) -> Option<SigmarNode> {
        match self {
            SigmarNode::Tin => Some(SigmarNode::Lead),
            SigmarNode::Iron => Some(SigmarNode::Tin),
            SigmarNode::Copper => Some(SigmarNode::Iron),
            SigmarNode::Silver => Some(SigmarNode::Copper),
            SigmarNode::Gold => Some(SigmarNode::Silver),
            _ => None,
        }
    }
    pub fn upgrade(&self) -> Option<SigmarNode> {
        match self {
            SigmarNode::Lead => Some(SigmarNode::Tin),
            SigmarNode::Tin => Some(SigmarNode::Iron),
            SigmarNode::Iron => Some(SigmarNode::Copper),
            SigmarNode::Copper => Some(SigmarNode::Silver),
            SigmarNode::Silver => Some(SigmarNode::Gold),
            _ => None,
        }
    }
    /// Does this cancel as a pair with other?
    fn cancels_with(&self, other: &SigmarNode) -> bool {
        match self {
            SigmarNode::Salt => other.is_prime(),
            SigmarNode::Quicksilver => other.upgrade().is_some(),
            SigmarNode::Vitae => matches!(other, SigmarNode::Mors),
            _ if self.is_prime() => self == other,
            _ => false,
        }
    }

    /// Return the standard game sans 1 Gold to go in the center.
    fn standard_game(aether: bool) -> Vec<SigmarNode> {
        let mut game = vec![];

        for element in &[
            SigmarNode::Water,
            SigmarNode::Earth,
            SigmarNode::Fire,
            SigmarNode::Air,
        ] {
            for _ in 0..8 {
                game.push(element.clone());
            }
        }
        for _ in 0..4 {
            game.push(SigmarNode::Salt);
        }
        if aether {
            for _ in 0..2 {
                game.push(SigmarNode::Aether);
            }
        }

        for node in &[SigmarNode::Lead, SigmarNode::Tin, SigmarNode::Iron, SigmarNode::Copper, SigmarNode::Silver] {
            game.push(node.to_owned());
            game.push(SigmarNode::Quicksilver);
        }

        game
    }
}
