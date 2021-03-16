use crate::Board;

pub trait Node: Sized {
    /// How many contiguous open neighbors does this node need to be free?
    fn freeness_req(&self) -> usize;

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
