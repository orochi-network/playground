use super::read_write_access::ReadWriteAccess;

// this struct aims to store the access at time_tag from stack[location]
// access can either be read or write with value
#[derive(Debug)]
pub struct StackAccess {
    location: usize,
    time_tag: u32, 
    access_operation: ReadWriteAccess,
    value: u32,
}

impl StackAccess {
    pub fn new(location: usize, time_tag: u32, access_operation: ReadWriteAccess, value: u32) -> Self {
        Self {
            location: location,
            time_tag: time_tag,
            access_operation: access_operation,
            value: value,
        }
    }

    pub fn get_location(&self) -> usize {
        self.location
    }

    pub fn get_time_tag(&self) -> u32 {
        self.time_tag
    }

    pub fn get_access_operation(&self) -> ReadWriteAccess {
        self.access_operation
    }

    pub fn get_value(&self) -> u32 {
        self.value
    }
}