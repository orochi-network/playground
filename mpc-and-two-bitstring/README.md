# MPC for Agent-to-Agent Calendar Matching (using `mpz`)

## Overview

This project demonstrates a **secure multi-party computation (MPC)** protocol between two agents — Alice and Bob — who want to schedule a meeting without revealing their full availability to each other.

Each party privately encodes their weekly calendar as a **bitstring**:
- `1` indicates a **FREE** timeslot
- `0` indicates a **BUSY** timeslot

Using MPC, both agents jointly compute the **bitwise AND** of their calendars. The result reveals only the common free slots, ensuring **privacy** of individual schedules.

### Time Slot Configuration

- **Working hours/day**: 8 hours  
- **Days/week**: 7  
- **Time slot granularity**: 30 minutes  

**Total slots:**  
`8 hours * 2 slots/hour * 7 days = 112 timeslots`  
→ Each calendar is a 112-bit bitstring.

### Example

```

Alice:   01010101 01010101 ...
Bob:     00000101 11110101 ...
Result:  00000101 01010101 ...

````

Only the overlapping `1` bits represent mutual availability.

## Project Structure

```
mpc-and-two-bitstring/
├── common/     # Shared utilities for TCP communication between Alice and Bob
├── mpc/        # Core MPC logic and binaries for Alice and Bob
├── Cargo.toml  # Rust project configuration
```

## Running the Demo

Open **two terminal windows** — one for each agent.

### Terminal 1 (Alice)

```bash
cd mpc
cargo run --bin alice
````

### Terminal 2 (Bob)

```bash
cd mpc
cargo run --bin bob
```

The program will establish a TCP connection and compute the secure intersection of their calendars.

### Result
```plaintext
match_time_slot: [2b, 40, 14, 2, 28, 0, 92, 86, a9, 35, 14, 0, 1, 83]
```
This is the hexa format of result.

## Explain code:

- Open TCP connection 

    ```Rust    
        // Open a connection.
        let tcp = tcp_connect(Role::Alice, DEFAULT_LOCAL).await?;
        let mut context = Context::new_single_threaded(tcp);
    ```

- Instantiate a vm for garbled circuits
    ```Rust
        // Instantiate a vm for garbled circuits.
        let mut garble_vm = setup_garbler().await?;
    ```

- Define Input Type

    ```Rust
        // One day 8 hours = 16 * 0.5 hours
        // One week = 7 days
        // Each bit represents 0.5 hours of availability ==> 1 week = 7 days * 16 * 0.5 hours
        // 1 week = 7 * 16 = 112 bits = 14 * U8
        let alice_calendar: Array<U8, 14> = garble_vm.alloc()?; //Alice's calendar
        let bob_calendar: Array<U8, 14> = garble_vm.alloc()?;   //Bob's calendar
    ```

- Define Input Visibility
    
    ```Rust
        // Define input visibility.
        garble_vm.mark_private(alice_calendar)?;
        garble_vm.mark_blind(bob_calendar)?;
    ```

    for Alice her calendar is private and Bob's calendar is blind, and vice versa.

- Define Circuit:

    ```Rust
        let and_two_bitstring_circuit = and_two_bitstring_circuit(112);
    ```

- Define output of circuit (bitwise AND of 2 bitstring):

    ```Rust
        let output: Array<U8, 14> = garble_vm
        .call(Call::builder(and_two_bitstring_circuit.into()).arg(alice_calendar).arg(bob_calendar).build()?)?;
    ```

- Assign input value:

    ```Rust
        // Assign the Alice's Calendar.
        garble_vm.assign(
            alice_calendar,
            [
                0x2b_u8, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf
            ],
        )?;
    ```

    This is 14xU8 represent for 112 bit.

- Run MPC:

    ```Rust
        // Commit the values
        garble_vm.commit(alice_calendar)?;
        garble_vm.commit(bob_calendar)?;

        // Execute the circuit.
        garble_vm.execute_all(&mut context).await?;
    ```

- Print Output (bitwise AND of 2 bitstring):

    ```Rust
        let match_time_slot = output.try_recv()?.unwrap();
        println!("final output: {:x?}", match_time_slot);
    ```