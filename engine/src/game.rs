use crate::data::*;
use crate::evaluation::*;
use crate::condition::ConditionParser;
use crate::{engine_conds, engine_matchers};
use serde_json::{value::Value, map::Map};
use std::collections::HashMap;

pub struct GameParser {
    conditions: ConditionMap,
    matchers: MatcherMap,
}
impl GameParser {
    pub fn new() -> Self {
        let mut me = GameParser {
            conditions: HashMap::new(),
            matchers: HashMap::new(),
        }:
        engine_conds::register(conditions);
        engine_matchers::register(matches);
        me
    }
    // TODO: add stubs for registering more things
    pub fn parse(&self, value: Value) -> Result<DataGame, String> {
        let mut BaseGame = DataGame {
            id: "",
            nodes: HashMap::new(),
            tags: HashMap::new(),
            mappings: HashMap::new(),
            board: HashMap::new(),
            changes: ChangeTree::new(),
            select: eval_const(EvalValue::(0))
        }
        nodes
    }
}
