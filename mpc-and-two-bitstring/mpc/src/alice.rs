use common::{tcp_connect, Role, DEFAULT_LOCAL};
use mpc::{setup_garbler, and_two_bitstring_circuit};
use mpz_common::Context;
use mpz_memory_core::{binary::U8, Array, MemoryExt, ViewExt};
use mpz_vm_core::{Call, CallableExt, Execute};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open a connection.
    let tcp = tcp_connect(Role::Alice, DEFAULT_LOCAL).await?;
    let mut context = Context::new_single_threaded(tcp);

    // Instantiate a vm for garbled circuits.
    let mut garble_vm = setup_garbler().await?;

    // One day 8 hours = 16 * 0.5 hours
    // One week = 7 days
    // Each bit represents 0.5 hours of availability ==> 1 week = 7 days * 16 * 0.5 hours
    // 1 week = 7 * 16 = 112 bits = 14 * U8
    let alice_calendar: Array<U8, 14> = garble_vm.alloc()?; //Alice's calendar
    let bob_calendar: Array<U8, 14> = garble_vm.alloc()?;   //Bob's calendar

    // Define input visibility.
    garble_vm.mark_private(alice_calendar)?;
    garble_vm.mark_blind(bob_calendar)?;

    let and_two_bitstring_circuit = and_two_bitstring_circuit(112);

    let output: Array<U8, 14> = garble_vm
        .call(Call::builder(and_two_bitstring_circuit.into()).arg(alice_calendar).arg(bob_calendar).build()?)?;

    let mut output = garble_vm.decode(output)?;

    // Assign the Alice's Calendar.
    garble_vm.assign(
        alice_calendar,
        [
            0x2b_u8, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf
        ],
    )?;

    // Commit the values
    garble_vm.commit(alice_calendar)?;
    garble_vm.commit(bob_calendar)?;

    // Execute the circuit.
    garble_vm.execute_all(&mut context).await?;

    let match_time_slot = output.try_recv()?.unwrap();
    println!("match_time_slot: {:x?}", match_time_slot);

    Ok(())
}
