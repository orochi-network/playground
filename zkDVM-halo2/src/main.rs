use strum::IntoEnumIterator;
use zkDVM_halo2::{test::{test_executing_gcd::TestExecutingGreatestCommonDivisor, program_execution_handler::ProgramExecutionHandler}, dummy_virtual_machine::{
    opcode::Opcode, numeric_encoding::NumericEncoding
}};

fn main() {
    TestExecutingGreatestCommonDivisor::execute(100);

    let x = vec!['1', '2', '3'];
    let y = vec!['a', 'b', 'c'];
    let product: Vec<String> = x
        .iter()
        .map(|&item_x| y
            .iter()
            .map(move |&item_y| [item_x, item_y]
                .iter()
                .collect()
                )
            )
        .flatten()
        .collect();
    
    println!("{:?}", product);

    let res: Vec<(u32, u32)> = (0..10).map(|index| {
        Opcode::iter().map(move |opcode| (index as u32, opcode.to_u32()))
    }).flatten().collect();
    println!("{:?}", res);
}
