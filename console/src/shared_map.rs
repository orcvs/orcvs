use lang::{Atoms, Interpreter, Portal};
use std::sync::Arc;
use std::sync::RwLock;

// https://draft.ryhl.io/blog/shared-mutable-state/
#[derive(Clone, Debug)]
pub struct SharedMap {
    inner: Arc<RwLock<ExpressionMap>>,
}

#[derive(Clone, Debug)]
pub struct ExpressionMap {
    data: Vec<Option<Atoms>>,
}

impl SharedMap {
    pub fn new(capacity: usize) -> Self {
        let data = vec![None; capacity];
        let map = ExpressionMap { data };

        Self {
            inner: Arc::new(RwLock::new(map)),
        }
    }

    pub fn get(&self, idx: usize) -> Option<Atoms> {
        let lock = self.inner.read().unwrap();
        lock.data[idx].clone()
    }

    pub fn insert(&self, idx: usize, a: Atoms) {
        let mut lock = self.inner.write().unwrap();
        lock.data[idx] = Some(a)
    }

    pub fn remove(&self, idx: usize) {
        let mut lock = self.inner.write().unwrap();
        lock.data[idx] = None;
    }

    pub fn execute(&self) -> Vec<Result<Portal, lang::Error>> {
        let lock = self.inner.write().unwrap();
        lock.data
            .iter()
            .filter(|o| o.is_some())
            .map(|o| match o {
                Some(atoms) => Interpreter::execute(atoms),
                None => Ok(Portal::default()),
            })
            .collect()
    }
}
