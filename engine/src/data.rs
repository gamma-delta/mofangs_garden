use hex2d::Coordinate;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier {
    id: String,
}
impl Identifier {
    pub fn parse(id: &str, game: &str) -> Result<Identifier, String> {
        // TODO use a regex on this probably
        match id.split(":").collect::<Vec<_>>().as_slice() {
            [id] => Ok(Identifier {
                id: format!("{}:{}", game, id),
            }),
            [_, _] => Ok(Identifier {
                id: String::from(id),
            }),
            _ => Err(format!("Bad identifier {}!", id)),
        }
    }
}

// TODO: Fill in these stubs

pub struct DataGame {
    pub id: String,
    pub nodes: HashMap<Identifier, DataNode>,
    pub tags: HashMap<String, HashSet<DataNode>>,
    pub mappings: HashMap<String, HashMap<DataNode, DataNode>>,
    board: HashMap<Coordinate, Option<DataNode>>,
}

impl DataGame {
    pub fn get_node(&self, coord: Coordinate) -> Option<&DataNode> {
        self.board.get(&coord).map(Option::as_ref).flatten()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DataNode {
    name: Identifier,
}
