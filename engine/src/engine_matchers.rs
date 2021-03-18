use crate::change::{ChangeParser, PredicateMap};
use crate::data::{Identifier, DataNode};
use crate::evaluation::{EvalValue, Predicate};
use serde_json::{map::Map, value::Value};

fn engine_mapping_key(evaluator: &ChangeParser, map: &Map<String, Value>) -> Result<Predicate<DataNode>, String> {
    let mapping = map["tag"].as_str().ok_or_else(|| "Tag key needs to be a string!".to_string())?;
    // TODO: Borrow rather than clone this
    let set = evaluator.game.tags.get(mapping).ok_or_else(|| format!("Tag {} doesn't exist!", mapping))?.clone();
    let bind = map.get("bind").map(|k| k.as_str().ok_or_else(|| format!("Binding {} must be a string!", k))).transpose()?.map(str::to_string);
    Ok(Box::new(move |n, ctx| {
        let has = set.contains(n);
        if has {
            if let Some(binding) = &bind {
                ctx.scope.add(binding.to_string(), EvalValue::Node(Some(n.clone())));
            }
        }
        Ok(has)
    }))
}

fn engine_tag(evaluator: &ChangeParser, map: &Map<String, Value>) -> Result<Predicate<DataNode>, String> {
    let mapping = map["mapping"].as_str().ok_or_else(|| "Mapping key needs to be a string!".to_string())?;
    // TODO: Borrow rather than clone this
    let tmap = evaluator.game.mappings.get(mapping).ok_or_else(|| format!("Mapping {} doesn't exist!", mapping))?.clone();
    let keybind = map.get("bind-key").map(|k| k.as_str().ok_or_else(|| format!("Binding {} must be a string!", k))).transpose()?.map(str::to_string);
    let valbind = map.get("bind-val").map(|k| k.as_str().ok_or_else(|| format!("Binding {} must be a string!", k))).transpose()?.map(str::to_string);
    Ok(Box::new(move |n, ctx| {
        Ok(match tmap.get(n) {
            Some(v) => {
                if let Some(key) = &keybind {
                    ctx.scope.add(key.to_string(), EvalValue::Node(Some(n.clone())));
                }
                if let Some(val) = &valbind {
                    ctx.scope.add(val.to_string(), EvalValue::Node(Some(v.clone())));
                }
                true
            },
            None => false,
        })
    }))
}

pub fn register(map: &mut PredicateMap) {
    let mut register_one = |name, fun| map.insert(Identifier::parse(name, "engine").unwrap(), fun);
    register_one("mapping-key", engine_mapping_key);
    register_one("tag", engine_tag);
}
