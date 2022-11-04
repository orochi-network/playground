pub enum AccessType {
    Garbage = 0x00, // only 1 cell of memory
    Stack = 0x01, // indicating the stack
    Memory = 0x02, // indicating the memory
}