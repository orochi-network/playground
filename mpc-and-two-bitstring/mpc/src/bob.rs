use common::{tcp_connect, Role, DEFAULT_LOCAL};
use mpc::{setup_evaluator, and_two_bitstring_circuit};
use mpz_common::Context;
use mpz_memory_core::{binary::U8, Array, MemoryExt, ViewExt};
use mpz_vm_core::{Call, CallableExt, Execute};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open a connection.
    let tcp = tcp_connect(Role::Bob, DEFAULT_LOCAL).await?;
    let mut context = Context::new_single_threaded(tcp);

    // Instantiate a vm for garbled circuits.
    let mut evaluator = setup_evaluator().await?;

    // Define input types.

    // One day 8 hours = 16 * 0.5 hours
    // One week = 7 days
    // Each bit represents 0.5 hours of availability ==> 1 week = 7 days * 16 * 0.5 hours
    // 1 week = 7 * 16 = 112 bits = 14 * U8
    let alice_calendar: Array<U8, 14> = evaluator.alloc()?; //Alice's calendar
    let bob_calendar: Array<U8, 14> = evaluator.alloc()?;   //Bob's calendar

    // Define input visibility.
    evaluator.mark_blind(alice_calendar)?;
    evaluator.mark_private(bob_calendar)?;

    let and_two_bitstring_circuit = and_two_bitstring_circuit(112);
    let output: Array<U8, 14> = evaluator
        .call(Call::builder(and_two_bitstring_circuit.into()).arg(alice_calendar).arg(bob_calendar).build()?)?;

    let mut output = evaluator.decode(output)?;

    // Assign the Bob's Calendar.
    evaluator.assign(
        bob_calendar,
        [
            0x6b_u8, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93,
        ],
    )?;

    // Commit the values
    evaluator.commit(alice_calendar)?;
    evaluator.commit(bob_calendar)?;

    // Execute the circuit.
    evaluator.execute_all(&mut context).await?;

    let match_time_slot = output.try_recv()?.unwrap();
    println!("match_time_slot: {:x?}", match_time_slot);

    Ok(())
}
