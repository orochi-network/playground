# Halo2

## How to implement a circuit in Halo2

There are for fundamental steps that can be done to implement any Halo2 circuit as follows:

- Define a `Config` struct that contains a column used in a circuit.
- Define a `Chip` struct that constraints the circuit and provide assignment functions.
- Define a `MyCircuit` struct that  implements the `Circuit` trait.
- Test your circuit with specific input.

## Halo2 Example

In this, section, let us give a concrete example of a simple Halo2 circuit. We would like to prove that there exists values $u,v$
such that
$$y=u^3+u^2v+uv^2+v^3+1$$
for a public known $y$ without revealing $u$ and $v$.