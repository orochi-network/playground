use halo2_proofs::circuit::Value;
use halo2_proofs::plonk::Selector;
use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter, SimpleFloorPlanner},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Fixed, Instance},
    poly::Rotation,
};
use std::env;
use std::marker::PhantomData;

// We would like to prove that there exists (u,v) such that
// y=u^3+u^2*v+u*v^2+v^3+1
// where y is a known public value

// Step 1: Define the configuration table
#[derive(Clone, Debug)]
struct MyConfig {
    // The advice column, containing the witness
    advice: [Column<Advice>; 3],
    // The instance column, containing the public values
    instance: Column<Instance>,
    // The fixed column, containing the fixed values, used for lookup
    constant: Column<Fixed>,

    // The selectors
    s_add: Selector,
    s_mul: Selector,
    s_add_c: Selector,
    s_mul_c: Selector,
}

// Step 2: Define a chip struct to constraint the circuit and provide
// assignment functions
struct FChip<Field: FieldExt> {
    // the chip must contain the configuration table
    config: MyConfig,
    _marker: PhantomData<Field>,
}

// Implement the chip struct
// the chip struct must have two functions: config() and loaded()
// these functions are not that necessary
impl<Field: FieldExt> Chip<Field> for FChip<Field> {
    type Config = MyConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<Field: FieldExt> FChip<Field> {
    // describe the arrangement of the circuit
    // normally we just need to define the gate in here
    fn configure(
        meta: &mut ConstraintSystem<Field>,
        advice: [Column<Advice>; 3],
        instance: Column<Instance>,
        constant: Column<Fixed>,
    ) -> <Self as Chip<Field>>::Config {
        // specify columns used for proving copy constraints
        // enable_equality() allows the specified column to participate
        // in the permutation check
        meta.enable_equality(instance);
        // enable_constant() allows the fixed column to be used
        meta.enable_constant(constant);
        for column in &advice {
            meta.enable_equality(*column);
        }

        // extract columns with respect to selectors
        let s_add = meta.selector();
        let s_mul = meta.selector();
        let s_add_c = meta.selector();
        let s_mul_c = meta.selector();

        // define addition gate
        // requires s_add*(witness[0]+witness[1]-witness[2])=0
        meta.create_gate("add", |meta| {
            let s_add = meta.query_selector(s_add);
            let lhs = meta.query_advice(advice[0], Rotation::cur());
            let rhs = meta.query_advice(advice[1], Rotation::cur());
            let out = meta.query_advice(advice[2], Rotation::cur());
            vec![s_add * (lhs + rhs - out)]
        });

        // define multiplication gate
        // requires s_mul*(witness[0]*witness[1]-witness[2])=0
        meta.create_gate("mul", |meta| {
            let s_mul = meta.query_selector(s_mul);
            let lhs = meta.query_advice(advice[0], Rotation::cur());
            let rhs = meta.query_advice(advice[1], Rotation::cur());
            let out = meta.query_advice(advice[2], Rotation::cur());
            vec![s_mul * (lhs * rhs - out)]
        });

        // define addition with constant gate
        // requires s_add_c*(witness[0]+fixed-witness[2])=0
        meta.create_gate("add with constant", |meta| {
            let s_add_c = meta.query_selector(s_add_c);
            let lhs = meta.query_advice(advice[0], Rotation::cur());
            let fixed = meta.query_fixed(constant, Rotation::cur());
            let out = meta.query_advice(advice[2], Rotation::cur());
            vec![s_add_c * (lhs + fixed - out)]
        });

        // define multiplication with constant gate
        // requires s_mul_c*(witness[0]*fixed-witness[2])=0
        meta.create_gate("mul with constant", |meta| {
            let s_mul_c = meta.query_selector(s_mul_c);
            let lhs = meta.query_advice(advice[0], Rotation::cur());
            let fixed = meta.query_fixed(constant, Rotation::cur());
            let out = meta.query_advice(advice[2], Rotation::cur());
            vec![s_mul_c * (lhs * fixed - out)]
        });

        MyConfig {
            advice,
            instance,
            constant,
            s_add,
            s_mul,
            s_add_c,
            s_mul_c,
        }
    }
}

// 3. Define a circuit struct that implement the circuit trait
// the Circuit trait has THREE functions:
// without_witnesses(), configure() and synthesize()
// a) the function  without_witnesses() returns a copy of the circuit
// without the witness value
// b) the function configure() describes the gate arrangement and column
// arrangement
// c) the function synthesize() synthesizes the circuit
#[derive(Default)]
struct MyCircuit<Field: FieldExt> {
    u: Value<Field>,
    v: Value<Field>,
}

fn main() {}
