use enum_map::Enum;
use hex2d::{Angle, Coordinate, Direction, Spin};
use itertools::Itertools;
use mofang_engine::{all_unique, Board, Node, PartialResult};

/// One of the marbles on the game board.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Enum)]
pub enum MofangNode {
    Wood,
    Fire,
    Earth,
    Metal,
    Water,

    Heavenly,
    Earthly,
    Human,

    Yin,
    Yang,

    Creation,
    Destruction,

    Qi,
}

impl Node for MofangNode {
    fn name() -> &'static str {
        "mofang"
    }
    fn texture_name(&self) -> &'static str {
        match self {
            MofangNode::Wood => "wood",
            MofangNode::Fire => "fire",
            MofangNode::Earth => "earth",
            MofangNode::Metal => "metal",
            MofangNode::Water => "water",
            MofangNode::Heavenly => "heavenly",
            MofangNode::Earthly => "earthly",
            MofangNode::Human => "human",
            MofangNode::Yin => "yin",
            MofangNode::Yang => "yang",
            MofangNode::Creation => "creation",
            MofangNode::Destruction => "destruction",
            MofangNode::Qi => "qi",
        }
    }
    fn can_select(&self, board: &Board<MofangNode>, coord: &Coordinate, selected: &[Coordinate]) -> bool {
        let freeness_req = match selected {
            // Human magic
            [human_coord]
                if matches!(board.get_node(*human_coord), Some(MofangNode::Human))
                    && self.is_elemental() =>
            {
                2
            }
            _ => self.freeness_req(),
        };
        board.max_open_neighbors(coord) >= freeness_req
    }

    /// Given a list of Nodes, see whether this pattern could exist
    /// and, if so, what to replace each Node with.
    fn select(nodes: &[&MofangNode]) -> PartialResult<Vec<Option<MofangNode>>> {
        match nodes.len() {
            0 => unreachable!("You can't select 0 nodes!"),
            1 => PartialResult::Continue,
            2 if nodes[0].cancels_with(&nodes[1]) => PartialResult::Success(vec![None, None]),
            _ => {
                // TODO: I am bad at rust, not sure how sorted_unstable_by_key works
                let (original_idxes, sorted): (Vec<_>, Vec<_>) = nodes
                    .iter()
                    .cloned()
                    .enumerate()
                    .sorted_unstable_by_key(|(_i, n)| *n)
                    .unzip();
                let unsort = |v| {
                    original_idxes
                        .into_iter()
                        .zip(v)
                        .sorted_unstable_by_key(|val| val.0)
                        .map(|(_, val)| val)
                        .collect_vec()
                };

                match sorted.as_slice() {
                    // Destruction requires some Special Handling
                    [rest @ .., MofangNode::Destruction]
                        if rest.iter().all(|&n| n.is_elemental()) && all_unique(rest.iter()) =>
                    {
                        if rest.len() == 5 {
                            // we caught them all
                            PartialResult::Success(vec![None; 6])
                        } else {
                            // we're still working on it
                            PartialResult::Continue
                        }
                    }
                    // Yin and Yang become change
                    [MofangNode::Yin, MofangNode::Yang] => PartialResult::Success(vec![
                        Some(MofangNode::Creation),
                        Some(MofangNode::Creation),
                    ]),

                    // 2 of the cycle can be selected but don't do anything
                    [MofangNode::Heavenly, MofangNode::Earthly]
                    | [MofangNode::Heavenly, MofangNode::Human]
                    | [MofangNode::Earthly, MofangNode::Human] => PartialResult::Continue,

                    [MofangNode::Heavenly, MofangNode::Earthly, MofangNode::Human] => {
                        PartialResult::Success(vec![None, None, None])
                    }

                    // Qi matches with elements
                    /*
                    [element, MofangNode::Qi] if element.is_elemental() => {
                        PartialResult::Success(unsort(vec![Some(MofangNode::Qi), None]))
                    }
                    */

                    /*
                    [MofangNode::Heavenly, MofangNode::Yang] => {
                        PartialResult::Success(unsort(vec![Some(MofangNode::Yang), None]))
                    }
                    [MofangNode::Earthly, MofangNode::Yin] => {
                        PartialResult::Success(unsort(vec![Some(MofangNode::Yin), None]))
                    }
                    */
                    // Human ingenuity can attract any element
                    [element, MofangNode::Human] if element.is_elemental() => {
                        PartialResult::Success(unsort(vec![None, Some((*element).clone())]))
                    }

                    [changeable, MofangNode::Creation] if changeable.can_change() =>
                    // Change nodes
                    {
                        PartialResult::Success(unsort(vec![changeable.change(), None]))
                    }

                    _ => PartialResult::Failure,
                }
            }
        }
    }

    fn new_game(seed: u64) -> Board<MofangNode> {
        let radius = 11;
        let rand = fastrand::Rng::new();

        let mut bank = Self::standard_game();
        rand.shuffle(&mut bank);

        let mut out = Board::new(11);
        out.nodes
            .insert(Coordinate::new(0, 0), Some(MofangNode::Destruction));

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
                    || (matches!(node, MofangNode::Qi)
                        && Direction::all().iter().filter(|&&dir| {
                            matches!(out.get_node(coord + dir), Some(MofangNode::Qi))
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
        let chirality = if rand.bool() { 1 } else { -1 };
        // make the non-spokes surrounding the center always be qi
        for idx in 0..3 {
            let pos = Coordinate::new(0, -chirality).rotate_around_zero(Angle::from_int(idx * 2));
            try_insert(pos, MofangNode::Qi, false);
        }

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

                    if on_spoke && radius == 0 {
                        // Just place Qi
                        try_insert(coord, MofangNode::Qi, false);
                    }
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

impl MofangNode {
    /// What's the number of contiguous open nodes required to be selectable?
    fn freeness_req(&self) -> usize {
        match self {
            MofangNode::Qi => 5,
            _ => 3,
        }
    }

    pub fn is_elemental(&self) -> bool {
        matches!(
            self,
            MofangNode::Wood
                | MofangNode::Fire
                | MofangNode::Earth
                | MofangNode::Metal
                | MofangNode::Water
        )
    }
    /// Does this cancel as a pair with other?
    fn cancels_with(&self, other: &MofangNode) -> bool {
        // One-to-one cancellations
        let expect = match self {
            MofangNode::Fire => Some(MofangNode::Metal),
            MofangNode::Metal => Some(MofangNode::Wood),
            MofangNode::Wood => Some(MofangNode::Earth),
            MofangNode::Earth => Some(MofangNode::Water),
            MofangNode::Water => Some(MofangNode::Fire),

            MofangNode::Heavenly => Some(MofangNode::Yang),
            MofangNode::Yang => Some(MofangNode::Heavenly),
            MofangNode::Earthly => Some(MofangNode::Yin),
            MofangNode::Yin => Some(MofangNode::Earthly),

            MofangNode::Qi => Some(MofangNode::Qi),
            MofangNode::Creation => Some(MofangNode::Creation),
            _ => None,
        };
        expect.filter(|o| *o == *other).is_some()
    }

    /// What does Change turn this into, if any?
    fn change(&self) -> Option<MofangNode> {
        match self {
            MofangNode::Wood => Some(MofangNode::Fire),
            MofangNode::Fire => Some(MofangNode::Earth),
            MofangNode::Earth => Some(MofangNode::Metal),
            MofangNode::Metal => Some(MofangNode::Water),
            MofangNode::Water => Some(MofangNode::Wood),

            MofangNode::Heavenly => Some(MofangNode::Earthly),
            MofangNode::Earthly => Some(MofangNode::Human),
            MofangNode::Human => Some(MofangNode::Heavenly),

            _ => None,
        }
    }

    /// Can a Change node change this?
    fn can_change(&self) -> bool {
        self.change().is_some()
    }

    /// Return the standard game sans 1 Destruction to go in the center
    /// and 3 qi to surround it.
    fn standard_game() -> Vec<MofangNode> {
        let mut game = vec![];

        for element in &[
            MofangNode::Wood,
            MofangNode::Fire,
            MofangNode::Earth,
            MofangNode::Metal,
            MofangNode::Water,
        ] {
            for _ in 0..7 {
                game.push(element.clone());
            }
        }

        for node in &[MofangNode::Heavenly, MofangNode::Earthly, MofangNode::Human] {
            for _ in 0..4 {
                game.push(node.clone());
            }
        }

        game.push(MofangNode::Yin);
        game.push(MofangNode::Yang);

        for _ in 0..3 {
            game.push(MofangNode::Qi);
        }

        game
    }
}
