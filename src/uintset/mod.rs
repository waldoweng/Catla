use std::collections::hash_map;

pub struct UIntSet {
    m: hash_map::HashMap<i32, bool>,
}

impl UIntSet {
    pub fn new() -> Self {
        Self {
            m: hash_map::HashMap::new(),
        }
    }

    pub fn has(&self, k: i32) -> bool {
        match self.m.get(&k) {
            Some(_) => return true,
            None => return false,
        }
    }
    pub fn add(&mut self, k: i32) -> Result<(), ()> {
        match self.m.insert(k, true) {
            Some(_) => return Ok(()),
            None => return Ok(()),
        }
    }
}
