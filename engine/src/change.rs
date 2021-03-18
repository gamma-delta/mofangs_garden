use std::collections::HashMap;
use crate::data::{Identifier, DataNode, DataGame};
use crate::evaluation::{Predicate, Evaluation, EvalValue, EvalContext, eval_const};
use crate::condition::ConditionParser;
use serde_json::{value::Value, map::Map};

pub struct ChangeTree {
    checkers: Vec<Checker>,
}

enum Matcher {
    Simple(Vec<DataNode>),
    Complex(Vec<Predicate<DataNode>>)
}

impl Matcher {
    fn test(&self, against: &[DataNode]) -> Vec<usize> {
        match self {
            // TODO implement matcher logic
            Matcher::Simple(nodes) => vec![],
            Matcher::Complex(nodes) => vec![],
        }
    }
}

enum NextChecker {
    Checker(Box<Checker>),
    Resolution(Vec<Evaluation>),
}

pub struct Checker {
    predicate: Matcher,
    guard: Option<Evaluation>,
    child: NextChecker,
}

pub type MatcherMap =
    HashMap<Identifier, fn(&ChangeParser, &Map<String, Value>) -> Result<Predicate<DataNode>, String>>;

pub struct ChangeParser<'a> {
    game: &'a DataGame,
    cond_parser: &'a ConditionParser<'a>,
    functions: &'a MatcherMap,
}

impl ChangeParser<'_> {
    pub fn parse(&self, value: &Value) -> Result<Checker, String> {
        let map = value.as_object().ok_or_else(|| format!("All mofang changes must be objects!"))?;
        let inputs = map["input"].as_array().ok_or_else(|| format!("Input to a mofang change must be an array!"))?;
        Ok(Checker {
            predicate: self.parse_inputs(inputs)?,
            guard: if matches!(map["guard"], Value::Null) {
                None
            } else {
                Some(self.cond_parser.parse(&map["guard"])?)
            },
            child: self.parse_next(map)?,
        })
    }
    fn parse_next(&self, map: &Map<String, Value>) -> Result<NextChecker, String> {
        let result = &map["result"];
        if !matches!(result, Value::Null) {
            let outputs = result.as_array().ok_or_else(|| format!("Outputs to a mofang change must be an array!"))?;
            return outputs.into_iter().map(|x| self.cond_parser.parse(x)).collect::<Result<Vec<_>, _>>().map(NextChecker::Resolution);
        }
        let next = &map["next"];
        if !matches!(next, Value::Null) {
            self.parse(&next).map(Box::new).map(NextChecker::Checker)
        } else {
            Err(String::from("Mofang change must have a result or a next!"))
        }
    }
    fn parse_inputs(&self, inputs: &Vec<Value>) -> Result<Matcher, String> {
        let mut data_nodes = vec![];
        'simple: loop {
            for val in inputs {
                if let Some(node) = self.parse_literal(val)? {
                    data_nodes.push(node);
                } else {
                    break 'simple;
                }
            }
            data_nodes.sort();
            return Ok(Matcher::Simple(data_nodes))
        }
        let mut predicates = data_nodes.into_iter().map(ChangeParser::to_predicate).collect::<Vec<_>>();
        for val in inputs {
            if let Some(node) = self.parse_literal(val)? {
                predicates.push(ChangeParser::to_predicate(node));
            } else {
                predicates.push(self.parse_predicate(val)?);
            }
        }
        Ok(Matcher::Complex(predicates))
    }
    fn to_predicate(value: DataNode) -> Predicate<DataNode> {
        Box::new(move |val, ctx| Ok(*val == value))
    }
    fn parse_literal(&self, value: &Value) -> Result<Option<DataNode>, String> {
        if let Some(val) = value.as_str() {
            let id = Identifier::parse(val, &self.game.id)?;
            let node = self.game.nodes.get(&id).map(DataNode::clone);
            if node.is_none() {
                Err(format!("Unknown node type {:?}", id))
            } else {
                Ok(node)
            }
        } else {
            Ok(None)
        }
    }
    fn parse_predicate(&self, value: &Value) -> Result<Predicate<DataNode>, String> {
        Err(String::from("nimpl"))
    }
}
