use std::collections::HashMap;
use crate::data::{Identifier, DataNode, DataGame};
use crate::nodes::PartialResult;
use crate::evaluation::{Predicate, Evaluation, EvalValue, EvalContext, eval_const};
use crate::condition::ConditionParser;
use serde_json::{value::Value, map::Map};
use itertools::Itertools;

pub struct ChangeTree {
    checkers: Vec<Checker>,
}

impl ChangeTree {
    pub fn test(&mut self, ctx: &mut EvalContext, against: &[DataNode]) -> Result<PartialResult<Vec<Option<DataNode>>>, String> {
        // TODO: Iter through checkers, return the first one that gives us Succ
        // If none give us Succ, return whether at least one gave us Cont
        Err(String::from("not impled"))
    }
}

enum Matcher {
    Simple(Vec<DataNode>),
    Complex(Vec<Predicate<DataNode>>)
}

impl Matcher {
    fn len(&self) -> usize {
        match self {
            Matcher::Simple(nodes) => nodes.len(),
            Matcher::Complex(nodes) => nodes.len(),
        }
    }
    fn test(&mut self, ctx: &mut EvalContext, against: &HashMap<usize, &DataNode>) -> Result<PartialResult<Vec<usize>>, String> {
        match self {
            Matcher::Simple(nodes) => {
                let against = against.iter().sorted_unstable_by_key(|(_, v)| *v).collect::<Vec<_>>();
                let found = if nodes.len() > against.len() {
                    None
                } else {
                    nodes.iter().try_fold((vec![], against.iter()), |(mut vec, mut iter), node| {
                        iter.find(|(_, &n)| n == node).map(|(&i, _)| {
                            vec.push(i);
                            (vec, iter)
                        })
                    })
                };
                Ok(match found {
                    Some((v, _)) => PartialResult::Success(v),
                    None => {
                        if nodes.len() >= against.len() && against.iter().try_fold(nodes.iter(), |mut iter, (_, &node)| iter.find(|&n| n == node).map(|_| iter)).is_some() {
                            PartialResult::Continue
                        } else {
                            PartialResult::Failure
                        }
                    }
                })
            }
            // TODO: This is annoyingly slow, optimize it dammit
            Matcher::Complex(ref mut nodes) => {
                let mut stack = vec![];
                let test_res = nodes.len() <= against.len() && Self::test_rec_matcher(ctx, nodes, against, &mut stack)?;
                Ok(if test_res {
                    PartialResult::Success(stack)
                } else {
                    if nodes.len() >= against.len() && Self::test_rec_node(ctx, nodes, against.iter().collect::<Vec<_>>().as_slice(), &mut stack)? {
                        PartialResult::Continue
                    } else {
                        PartialResult::Failure
                    }
                })
            }
        }
    }
    fn test_rec_matcher(ctx: &mut EvalContext, matchers: &mut [Predicate<DataNode>], against: &HashMap<usize, &DataNode>, stack: &mut Vec<usize>) -> Result<bool, String> {
        match matchers {
            [] => Ok(true),
            [head, tail @ ..] => {
                for (&i, n) in against.iter() {
                    if stack.iter().find(|&&v| v == i).is_none() && head(n, ctx)? {
                        stack.push(i);
                        let result = Self::test_rec_matcher(ctx, tail, against, stack)?;
                        if result {
                            return Ok(result);
                        }
                        stack.pop();
                    }
                }
                Ok(false)
            }
        }
    }
    fn test_rec_node(ctx: &mut EvalContext, matchers: &mut [Predicate<DataNode>], against: &[(&usize, &&DataNode)], stack: &mut Vec<usize>) -> Result<bool, String> {
        match against {
            [] => Ok(true),
            [(_, node), tail @ ..] => {
                for i in 0..matchers.len() {
                    if stack.iter().find(|&&v| v == i).is_none() && matchers[i](node, ctx)? {
                        stack.push(i);
                        let result = Self::test_rec_node(ctx, matchers, tail, stack)?;
                        if result {
                            return Ok(result);
                        }
                        stack.pop();
                    }
                }
                Ok(false)
            }
        }
    }
}

pub struct Checker {
    predicate: Matcher,
    guard: Option<Evaluation>,
    result: Vec<Evaluation>,
    child: Option<Box<Checker>>,
}

impl Checker {
    pub fn test(&mut self, ctx: &mut EvalContext, against: &[DataNode]) -> Result<PartialResult<Vec<Option<DataNode>>>, String> {
        let mut map: HashMap<usize, &DataNode> = against.iter().enumerate().collect();
        ctx.scope.push();
        let result = self.test_rec(ctx, &mut map)?;
        ctx.scope.pop();
        Ok(result.map(|map| map.into_iter().sorted_unstable_by_key(|t| t.0).map(|(_, v)| v).collect::<Vec<_>>()))
    }
    pub fn test_rec(&mut self, ctx: &mut EvalContext, against: &mut HashMap<usize, &DataNode>) -> Result<PartialResult<HashMap<usize, Option<DataNode>>>, String> {
        let result = match self.predicate.test(ctx, against)? {
            PartialResult::Success(vals) if matches!(self.guard.as_mut().map(|g| g(ctx)).transpose()?.map(EvalValue::as_bool).transpose()?, Some(true)) => {
                for pos in &vals {
                    against.remove(&pos);
                }
                let mut map = match self.child {
                    Some(ref mut child) => {
                        ctx.scope.push();
                        let child_res = child.test_rec(ctx, against)?;
                        ctx.scope.pop();
                        if let PartialResult::Success(map) = child_res {
                            map
                        } else {
                            return Ok(child_res);
                        }
                    },
                    None => HashMap::new(),
                };
                for pos in vals {
                    map.insert(pos, self.result[pos](ctx)?.as_node()?);
                }
                PartialResult::Success(map)
            },
            PartialResult::Continue => PartialResult::Continue,
            _ => PartialResult::Failure,
        };
        Ok(result)
    }
}

pub type PredicateMap =
    HashMap<Identifier, fn(&ChangeParser, &Map<String, Value>) -> Result<Predicate<DataNode>, String>>;

pub struct ChangeParser<'a> {
    pub game: &'a DataGame,
    pub cond_parser: &'a ConditionParser<'a>,
    functions: &'a PredicateMap,
}

impl ChangeParser<'_> {
    pub fn parse(&self, value: &Value) -> Result<Checker, String> {
        let map = value.as_object().ok_or_else(|| format!("All mofang changes must be objects!"))?;
        let inputs = map["input"].as_array().ok_or_else(|| format!("Input to a mofang change must be an array!"))?;
        let predicate = self.parse_inputs(inputs)?;
        Ok(Checker {
            result: self.parse_result(map, predicate.len())?,
            predicate: predicate,
            guard: if matches!(map["guard"], Value::Null) {
                None
            } else {
                Some(self.cond_parser.parse(&map["guard"])?)
            },
            child: self.parse_next(map)?.map(Box::new),
        })
    }
    fn parse_result(&self, map: &Map<String, Value>, expect_len: usize) -> Result<Vec<Evaluation>, String> {
        let result = map.get("result").ok_or_else(|| format!("Every match needs a result!"))?;
        if matches!(result, Value::Null) {
            Ok((0..expect_len).map(|_| eval_const(EvalValue::Node(None))).collect::<Vec<_>>())
        } else {
            let outputs = result.as_array().ok_or_else(|| format!("Outputs to a mofang change must be an array!"))?;
            if outputs.len() == expect_len {
                outputs.into_iter().map(|x| self.cond_parser.parse(x)).collect::<Result<Vec<_>, _>>()
            } else {
                Err(format!("Expected {} outputs for {} inputs (got {})!", expect_len, expect_len, outputs.len()))
            }
        }
    }
    fn parse_next(&self, map: &Map<String, Value>) -> Result<Option<Checker>, String> {
        let next = &map["next"];
        if matches!(next, Value::Null) {
            Ok(None)
        } else {
            self.parse(&next).map(Option::Some)
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
        if predicates.len() > 3 {
            // TODO: Remove this when we have a chance to optimize harder
            eprintln!("Complex matcher has length >3 (got {}). Performance may suffer!", predicates.len());
        }
        Ok(Matcher::Complex(predicates))
    }
    fn to_predicate(value: DataNode) -> Predicate<DataNode> {
        Box::new(move |val, _| Ok(*val == value))
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
        let map = value.as_object().ok_or_else(|| String::from("Mapping input must be map or string!"))?;
        let type_val = map
            .get("type")
            .ok_or_else(|| String::from("Missing type in Mapping JSON object"))?;
        let type_str = type_val
            .as_str()
            .ok_or_else(|| format!("Type of JSON object {} isn't a string", type_val))?;
        let type_id = Identifier::parse(type_str, &self.game.id)?;
        let generator = self
            .functions
            .get(&type_id)
            .ok_or_else(|| format!("No such function with identifier {:?}!", type_id))?;
        generator(self, map)
    }
}
