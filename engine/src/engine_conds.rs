use crate::condition::{ConditionParser, ConditionMap};
use crate::data::Identifier;
use crate::evaluation::{EvalValue, Evaluation};
use serde_json::{map::Map, value::Value};

fn engine_if(evaluator: &ConditionParser, map: &Map<String, Value>) -> Result<Evaluation, String> {
    let mut cond = evaluator.parse_key(&map, "cond")?;
    let mut then = evaluator.parse_key(&map, "then")?;
    let mut els = evaluator.parse_key(&map, "else")?;
    Ok(Box::new(move |ctx| {
        let cond = cond(ctx)?;
        if cond.as_bool()? {
            then(ctx)
        } else {
            els(ctx)
        }
    }))
}

fn engine_neighbors(
    evaluator: &ConditionParser,
    map: &Map<String, Value>,
) -> Result<Evaluation, String> {
    let mut value = evaluator.parse_key(&map, "value")?;
    Ok(Box::new(move |ctx| {
        let value = value(ctx)?;
        let target = value.as_int()?;
        let count = match ctx
            .pos
            .neighbors()
            .iter()
            .position(|&coord| ctx.game.get_node(coord).is_some())
        {
            Some(pos) => {
                // At least one neighbor exists, iter around it
                ctx.pos
                    .neighbors()
                    .iter()
                    .cycle()
                    .skip(pos + 1)
                    .take(6)
                    .fold((0, 0), |(maxrun, run), &neighbor| {
                        if ctx.game.get_node(neighbor).is_some() {
                            (maxrun.max(run), 0)
                        } else {
                            (maxrun, run + 1)
                        }
                    })
                    .0
            }
            // Everything is empty and fine
            None => 6,
        };
        Ok(EvalValue::Bool(count >= target))
    }))
}

// TODO: This is messy as hell
/*
fn engine_fold<B, O>(evaluator: &ConditionParser, map: &Map<String, Value>, mut start: B, fold: fn(B, EvalValue) -> Result<Result<B, O>, String>, finish: fn(Result<B, O>) -> EvalValue) -> Result<Evaluation, String> where B: Clone {
    let values = map.get("values").ok_or_else(|| format!("Missing values array"))?;
    let array = values.as_array().ok_or_else(|| format!("values must be an array"))?;
    let conds = evaluator.parse_list(array)?;
    Ok(Box::new(move |ctx| {
        let start = start.clone();
        for mut cond in conds {
            let start = fold(start, cond(ctx)?)?;
            if start.is_err() {
                return Ok(finish(start))
            }
        }
        Ok(finish(Ok(start)))
    }))
}
*/
fn get_values(evaluator: &ConditionParser, map: &Map<String, Value>) -> Result<Vec<Evaluation>, String> {
    let values = map.get("values").ok_or_else(|| format!("engine:and needs a values array"))?;
    let array = values.as_array().ok_or_else(|| format!("values must be an array"))?;
    evaluator.parse_list(array)
}
fn engine_and(evaluator: &ConditionParser, map: &Map<String, Value>) -> Result<Evaluation, String> {
    let mut conds = get_values(evaluator, map)?;
    Ok(Box::new(move |ctx| {
        for cond in &mut conds {
            if !cond(ctx)?.as_bool()? {
                return Ok(EvalValue::Bool(false))
            }
        }
        Ok(EvalValue::Bool(true))
    }))
}
fn engine_or(evaluator: &ConditionParser, map: &Map<String, Value>) -> Result<Evaluation, String> {
    let mut conds = get_values(evaluator, map)?;
    Ok(Box::new(move |ctx| {
        for cond in &mut conds {
            if cond(ctx)?.as_bool()? {
                return Ok(EvalValue::Bool(true))
            }
        }
        Ok(EvalValue::Bool(false))
    }))
}
fn engine_equals(evaluator: &ConditionParser, map: &Map<String, Value>) -> Result<Evaluation, String> {
    let mut conds = get_values(evaluator, map)?;
    Ok(Box::new(move |ctx| {
        match conds.as_mut_slice() {
            [] => 
                Ok(EvalValue::Bool(true)),
            [ref mut head, ref mut tail @ ..] => {
                let mut head = head(ctx);
                for next in tail {
                    let next = next(ctx);
                    if head != next {
                        return Ok(EvalValue::Bool(false))
                    }
                    head = next
                }
                Ok(EvalValue::Bool(true))
            },
        }
    }))
}

pub fn register(map: &mut ConditionMap) {
    let mut register_one = |name, fun| map.insert(Identifier::parse(name, "engine").unwrap(), fun);
    register_one("if", engine_if);
    register_one("and", engine_and);
    register_one("or", engine_or);
    register_one("equals", engine_equals);
    register_one("contiguous-neighbors", engine_neighbors);
}
