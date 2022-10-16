use zkDVM_halo2::opcode_definition::Opcode;
use zkDVM_halo2::opcode_definition::NumericEncoding;

fn main() {
    let s = Opcode::JMPI;
    println!("{}", s.to_u32());
    let d = Opcode::from_u32(12);
    println!("{}", d.to_u32());
}
