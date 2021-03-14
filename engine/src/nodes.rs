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

/// Represents a success, failure, or needs-more-info.
pub enum PartialResult<T> {
    Success(T),
    Continue,
    Failure,
}

impl<T> PartialResult<T> {
    /// Is this a success or a needs-more-info?
    pub fn is_valid(&self) -> bool {
        !matches!(self, PartialResult::Failure)
    }
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
        // One-to-one cancellations
        let expect = match self {
            Node::Fire => Some(Node::Metal),
            Node::Metal => Some(Node::Wood),
            Node::Wood => Some(Node::Earth),
            Node::Earth => Some(Node::Water),
            Node::Water => Some(Node::Fire),

            Node::Qi => Some(Node::Qi),
            Node::Change => Some(Node::Change),
            _ => None,
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

    /// Given a list of Nodes, see whether this pattern could exist
    /// and, if so, what to replace each Node with.
    pub fn select(nodes: &[Node]) -> PartialResult<Vec<Option<Node>>> {
        match nodes.len() {
            0 => unreachable!("You can't select 0 nodes!"),
            1 => PartialResult::Continue,
            2 if nodes[0].cancels_with(nodes[1]) =>
                PartialResult::Success(vec![None, None]),
            _ => {
                let (original_idxes, sorted): (Vec<_>, Vec<_>) = nodes
                    .iter()
                    .cloned()
                    .enumerate()
                    .sorted_unstable_by_key(|x| x.1)
                    .unzip();
                let unsort =
                    |v: Vec<Option<Node>>| original_idxes.iter().map(|&idx| v[idx]).collect_vec();

                match sorted.as_slice() {
                    // Yin and Yang become change
                    [Node::Yin, Node::Yang] => PartialResult::Success(vec![Some(Node::Change), Some(Node::Change)]),

                    // 2 of the cycle can be selected but don't do anything
                    [Node::Heavenly, Node::Earthly] | [Node::Heavenly, Node::Human] | [Node::Earthly, Node::Human] =>
                        PartialResult::Continue,

                    [Node::Heavenly, Node::Earthly, Node::Human] =>
                        PartialResult::Success(vec![None, None, None]),

                    // Qi matches with elements and turns the other into qi
                    [element, Node::Qi] if element.is_elemental() =>
                        PartialResult::Success(unsort(vec![Some(Node::Qi), None])),

                    // Heavenly things attract Yang, Earthly things attract Yin
                    [Node::Heavenly, Node::Yang] =>
                        PartialResult::Success(unsort(vec![Some(Node::Yang), None])),
                    [Node::Earthly, Node::Yin] =>
                        PartialResult::Success(unsort(vec![Some(Node::Yin), None])),

                    // Human ingenuity can attract any element
                    [element, Node::Human] if element.is_elemental() =>
                        PartialResult::Success(unsort(vec![None, Some(*element)])),

                    [changeable, Node::Change] if changeable.can_change() =>
                        // Change nodes
                        PartialResult::Success(unsort(vec![changeable.change(), None])),
                    _ => PartialResult::Failure,
                }
            }
        }
    }
}
