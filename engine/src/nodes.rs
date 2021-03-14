use enum_map::Enum;
use itertools::Itertools;

/// One of the marbles on the game board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Enum)]
pub enum Node {
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

    Change,

    Qi,
}

impl Node {
    pub fn is_elemental(self) -> bool {
        matches!(
            self,
            Node::Wood | Node::Fire | Node::Earth | Node::Metal | Node::Water
        )
    }

    /// What's the number of contiguous open nodes required to be selectable?
    pub fn freeness_req(self) -> usize {
        match self {
            Node::Qi => 6,
            _ => 3,
        }
    }

    /// Does this cancel as a pair with other?
    fn cancels_with(self, other: Node) -> bool {
        // Human cancels *all* elementals
        if self == Node::Human && other.is_elemental()
            || other == Node::Human && self.is_elemental() {
            return true;
        }
        // One-to-one cancellations
        let expect = match self {
            Node::Fire =>  Some(Node::Metal),
            Node::Metal => Some(Node::Earth),
            Node::Earth => Some(Node::Water),
            Node::Water => Some(Node::Wood),
            Node::Wood => Some(Node::Fire),

            Node::Heavenly => Some(Node::Yang),
            Node::Earthly => Some(Node::Yin),
            Node::Yang => Some(Node::Heavenly),
            Node::Yin => Some(Node::Earthly),

            Node::Qi => Some(Node::Qi),
            Node::Change => Some(Node::Change),
            _ => None
        };
        expect.filter(|o| *o == other).is_some()
    }

    /// What does Change turn this into, if any?
    pub fn change(self) -> Option<Node> {
        match self {
            Node::Wood => Some(Node::Fire),
            Node::Fire => Some(Node::Earth),
            Node::Earth => Some(Node::Metal),
            Node::Metal => Some(Node::Water),
            Node::Water => Some(Node::Wood),

            Node::Heavenly => Some(Node::Earthly),
            Node::Earthly => Some(Node::Human),
            Node::Human => Some(Node::Heavenly),

            _ => None,
        }
    }

    /// Given a list of Nodes, see whether this pattern could exist and what the outcome is.
    ///
    /// - Return `Err` if this is impossible.
    /// - Return Ok(None) if this is possible but not a complete pattern.
    /// - Return Ok(Some) with each node mapped to its result if it is complete.
    pub fn select(nodes: &[Node]) -> Result<Option<Vec<Option<Node>>>, ()> {
        if nodes.is_empty() {
            // HOW
            unreachable!("You can't select 0 nodes!")
        } else if nodes.len() == 1 {
            return Ok(None);
        } else if nodes.len() == 2 && nodes[0].cancels_with(nodes[1]) {
            return Ok(Some(vec![None, None]));
        } else {
            // Sort this top-to-bottom
            let (original_idxes, sorted): (Vec<_>, Vec<_>) = nodes
                .iter()
                .cloned()
                .enumerate()
                .sorted_unstable_by_key(|x| x.1)
                .unzip();
            let unsort =
                |v: Vec<Option<Node>>| original_idxes.iter().map(|&idx| v[idx]).collect_vec();

            if sorted == [Node::Yin, Node::Yang] {
                // Yin and Yang become change
                return Ok(Some(unsort(vec![Some(Node::Change), Some(Node::Change)])));
            } else if sorted == [Node::Heavenly, Node::Earthly]
                || sorted == [Node::Heavenly, Node::Human]
                || sorted == [Node::Earthly, Node::Human]
            {
                // 2 of the cycle can be selected but don't do anything
                return Ok(None);
            } else if sorted == [Node::Heavenly, Node::Earthly, Node::Human] {
                // all 3 of the cycle cancel
                return Ok(Some(vec![None, None, None]));
            } else if sorted.len() == 2 && sorted[0].is_elemental() && sorted[1] == Node::Qi {
                // Qi matches with elements and turns the other into qi
                return Ok(Some(unsort(vec![Some(Node::Qi), None])));
            } else if sorted.len() == 2 && sorted[1] == Node::Change {
                // Change combines with lots of things
                if let Some(res) = sorted[0].change() {
                    return Ok(Some(unsort(vec![Some(res), None])));
                }
            }
        }

        Err(())
    }
}
