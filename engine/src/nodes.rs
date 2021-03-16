use crate::{Board, Coordinate};

pub trait Node: Sized {
    /// What game are these nodes for?
    fn name() -> &'static str;
    /// What texture does this node have?
    fn texture_name(&self) -> &'static str;
    /// Can the node at this position be selected?
    fn can_select(&self, board: &Board<Self>, coordinate: &Coordinate, selected: &[Coordinate]) -> bool;

    /// Given a list of Nodes, see whether this pattern could exist
    /// and, if so, what to replace each Node with.
    fn select(nodes: &[&Self]) -> PartialResult<Vec<Option<Self>>>;

    /// Create a new game with the given seed.
    fn new_game(seed: u64) -> Board<Self>;

    /// Create a new random game.
    fn new_game_random() -> Board<Self> {
        Node::new_game(fastrand::u64(..))
    }
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
