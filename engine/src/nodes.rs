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
        } else if nodes.len() == 2 && nodes[0] == Node::Wood && nodes[1] == Node::Earth
            || nodes[0] == Node::Fire && nodes[1] == Node::Metal
            || nodes[0] == Node::Earth && nodes[1] == Node::Water
            || nodes[0] == Node::Metal && nodes[1] == Node::Wood
            || nodes[0] == Node::Water && nodes[1] == Node::Fire
            || nodes == [Node::Qi, Node::Qi]
        {
            // clippy wants me to combine these...
            // anyways, destroying elements or qi+qi cancen
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
            } else if sorted == [Node::Change, Node::Change] {
                // 2 Changes cancel
                return Ok(Some(unsort(vec![None, None])));
            } else if sorted == [Node::Heavenly, Node::Earthly]
                || sorted == [Node::Heavenly, Node::Human]
                || sorted == [Node::Earthly, Node::Human]
            {
                // 2 of the cycle can be selected but don't do anything
                return Ok(None);
            } else if sorted == [Node::Heavenly, Node::Earthly, Node::Human] {
                // all 3 of the cycle cancel
                return Ok(Some(unsort(vec![None, None, None])));
            } else if sorted == [Node::Heavenly, Node::Yang]
                || sorted == [Node::Earthly, Node::Yin]
                || sorted.len() == 2 && sorted[0].is_elemental() && sorted[1] == Node::Human
            {
                // Match cycles with yin, yang, and elements
                return Ok(Some(unsort(vec![None, None])));
            } else if sorted.len() == 2 && sorted[0].is_elemental() && sorted[1] == Node::Qi {
                // Qi matches with elements and turns the other into qi
                return Ok(Some(unsort(vec![Some(Node::Qi), None])));
            } else if sorted.len() == 2 && sorted[1] == Node::Change {
                // Change combines with lots of things
                let operand = sorted[0];
                if operand == Node::Change {
                    return Ok(Some(unsort(vec![None, None])));
                } else {
                    let res = match operand {
                        Node::Wood => Some(Node::Fire),
                        Node::Fire => Some(Node::Earth),
                        Node::Earth => Some(Node::Metal),
                        Node::Metal => Some(Node::Water),
                        Node::Water => Some(Node::Wood),

                        Node::Heavenly => Some(Node::Earthly),
                        Node::Earthly => Some(Node::Human),
                        Node::Human => Some(Node::Heavenly),

                        _ => None,
                    };
                    if let Some(res) = res {
                        return Ok(Some(unsort(vec![Some(res), None])));
                    }
                }
            }
        }

        Err(())
    }
}
