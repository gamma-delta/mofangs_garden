use std::collections::HashMap;
use crate::data::*;
use serde_json::Number;
use hex2d::Coordinate;
use std::{borrow::Borrow, hash::Hash};

pub struct Scope<K, V> {
    // Implementation note:
    // Since HashMaps are presumably expensive to init,
    // we count empty scopes pushed above a given scope
    // until we actually attempt to add something.
    bindings: Vec<(HashMap<K, V>, usize)>,
}

impl<K, V> Scope<K, V>
where
    K: Eq + Hash,
{
    pub fn new(base: HashMap<K, V>) -> Self {
        Scope {
            bindings: vec![(base, 0)],
        }
    }
    pub fn get<Q: ?Sized + Hash + Eq>(&self, id: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        self.bindings
            .iter()
            .rev()
            .map(|scope| scope.0.get(id))
            .find(Option::is_some)
            .flatten()
    }
    pub fn push(&mut self) -> &mut Self {
        self.bindings.last_mut().unwrap().1 += 1;
        self
    }
    pub fn add(&mut self, key: K, val: V) -> &mut Self {
        let (map, fake_count) = self.bindings.last_mut().unwrap();
        if *fake_count == 0 {
            map.insert(key, val);
        } else {
            *fake_count -= 1;
            let mut map = HashMap::new();
            map.insert(key, val);
            self.bindings.push((map, 0));
        }
        self
    }
    pub fn pop(&mut self) -> &mut Self {
        let (_, fake_count) = self.bindings.last_mut().unwrap();
        if *fake_count > 0 {
            *fake_count -= 1;
        } else if self.bindings.len() > 1 {
            self.bindings.pop();
        }
        self
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalValue {
    Bool(bool),
    Number(Number),
    Node(Option<DataNode>),
    Array(Vec<EvalValue>),
}

impl EvalValue {
    pub fn as_bool(self) -> Result<bool, String> {
        match self {
            EvalValue::Bool(b) => Ok(b),
            _ => Err(format!("Expected boolean, got {:?}", self)),
        }
    }
    pub fn as_int(self) -> Result<i64, String> {
        match self {
            EvalValue::Number(n) => n
                .as_i64()
                .ok_or_else(|| format!("Expected integer, got {}", n)),
            _ => Err(format!("Expected number, got {:?}", self)),
        }
    }
    pub fn as_node(self) -> Result<Option<DataNode>, String> {
        match self {
            EvalValue::Node(n) => Ok(n),
            _ => Err(format!("Expected node, got {:?}", self)),
        }
    }
}


pub struct EvalContext<'a> {
    pub pos: Coordinate,
    pub game: &'a DataGame,
    pub scope: Scope<String, EvalValue>,
}

pub type Evaluation = Box<dyn FnMut(&mut EvalContext) -> Result<EvalValue, String>>;
pub type Predicate<T> = Box<dyn FnMut(&T, &mut EvalContext) -> Result<bool, String>>;

pub fn eval_const(val: EvalValue) -> Evaluation {
    Box::new(move |_| Ok(val.clone()))
}
