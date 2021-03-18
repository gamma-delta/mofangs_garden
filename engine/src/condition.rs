use std::collections::HashMap;
use crate::data::{Identifier, DataGame};
use crate::evaluation::{Evaluation, EvalValue, eval_const};
use serde_json::{value::Value, map::Map};

pub type ConditionMap =
    HashMap<Identifier, fn(&ConditionParser, &Map<String, Value>) -> Result<Evaluation, String>>;

pub struct ConditionParser<'a> {
    game: &'a DataGame,
    functions: &'a ConditionMap,
}

impl ConditionParser<'_> {
    // Convenience method that parses a field of map.
    pub fn parse_key(&self, map: &Map<String, Value>, key: &str) -> Result<Evaluation, String> {
        let val = map
            .get(key)
            .ok_or_else(|| format!("Missing {} in Condition JSON object", key))?;
        self.parse(val)
    }
    // Convenience method that parses all elements of list.
    pub fn parse_list(&self, list: &Vec<Value>) -> Result<Vec<Evaluation>, String> {
        list.iter().try_fold(vec![], |mut exist, next| {
            self.parse(next).map(Box::new(move |v| {
                exist.push(v);
                exist
            }))
        })
    }
    pub fn parse(&self, value: &Value) -> Result<Evaluation, String> {
        match value {
            Value::Null => Ok(eval_const(EvalValue::Node(None))),
            Value::Bool(b) => Ok(eval_const(EvalValue::Bool(*b))),
            Value::Number(n) => Ok(eval_const(EvalValue::Number(n.to_owned()))),
            Value::String(s) => self.parse_string(s),
            Value::Array(vec) => self.parse_array(vec),
            Value::Object(map) => self.parse_object(map),
        }
    }
    fn parse_array(&self, array: &Vec<Value>) -> Result<Evaluation, String> {
        let mut eval_array = self.parse_list(array)?;
        Ok(Box::new(move |ctx| {
            eval_array
                .iter_mut()
                .try_fold(vec![], |mut arr, next| {
                    next(ctx).map(move |v| {
                        arr.push(v);
                        arr
                    })
                })
                .map(EvalValue::Array)
        }))
    }
    fn parse_string(&self, string: &str) -> Result<Evaluation, String> {
        if let Some(rest) = string.strip_prefix("@") {
            let id = rest.to_owned();
            Ok(Box::new(move |ctx| {
                ctx.scope
                    .get(&id)
                    .map(EvalValue::clone)
                    .ok_or_else(|| format!("Binding {} doesn't exist in the current context", id))
            }))
        } else {
            let identifier = Identifier::parse(&string, &self.game.id)?;
            if let Some(node) = self.game.nodes.get(&identifier) {
                Ok(eval_const(EvalValue::Node(Some(node.clone()))))
            } else {
                Err(format!("No such node as {:?} exists", identifier))
            }
        }
    }
    fn parse_object(&self, map: &Map<String, Value>) -> Result<Evaluation, String> {
        let type_val = map
            .get("type")
            .ok_or_else(|| String::from("Missing type in Condition JSON object"))?;
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
