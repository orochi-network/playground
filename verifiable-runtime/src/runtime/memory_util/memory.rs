use std::ops::Index;

pub struct Memory {
    memory: Vec<u32>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            memory: Vec::<u32>::new(),
        }
    }
}

impl Index<usize> for Memory {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.memory[index]
    }
}