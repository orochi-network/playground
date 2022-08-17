#![warn(unused)]
#![deny(
    trivial_casts,
    trivial_numeric_casts,
    variant_size_differences,
    stable_features,
    non_shorthand_field_patterns,
    renamed_and_removed_lints,
    private_in_public,
    unsafe_code
)]

use crate::dvm::{DVMContext, DVM};
use crate::opcode::{BinaryCode, Opcode};

use ark_groth16::{Groth16, Proof};
use ark_sponge::Absorb;
// For randomness (during paramgen and proof generation)
use ark_std::rand::Rng;

// For benchmarking
use std::{
    ops::AddAssign,
    time::{Duration, Instant},
};

// Bring in some tools for using pairing-friendly curves
// We're going to use the BLS12-377 pairing-friendly elliptic curve.
use ark_bls12_377::{Bls12_377, Fr};
use ark_bls12_381::Bls12_381;
use ark_ff::{BigInteger, Field, Fp256, One, PrimeField, Zero};
use ark_r1cs_std::{alloc::AllocVar, eq::EqGadget, fields::fp::FpVar};
use ark_std::test_rng;
use arkworks_r1cs_gadgets::poseidon::FieldHasherGadget;

// We'll use these interfaces to construct our circuit.
use ark_relations::{
    lc, ns,
    r1cs::{
        ConstraintSynthesizer, ConstraintSystemRef, LinearCombination, SynthesisError, Variable,
    },
};

/// This is our DVM circuit for proving state of DVM
struct DVMCircuit<F: PrimeField> {
    program: Vec<u8>,
    result: F,
}

/// Constructor for DVMCircuit
#[allow(dead_code)]
impl<F: PrimeField> DVMCircuit<F> {
    pub fn new(program: Vec<u8>, result: i32) -> Self {
        Self {
            program: program.clone(),
            result: F::from(result as u32),
        }
    }
}

impl<F: PrimeField> Clone for DVMCircuit<F> {
    fn clone(&self) -> Self {
        DVMCircuit {
            program: self.program.clone(),
            result: self.result.clone(),
        }
    }
}

/// Our DVM circuit implements this `Circuit` trait which
/// is used during paramgen and proving in order to
/// synthesize the constraint system.
impl<'a, F: PrimeField> ConstraintSynthesizer<F> for DVMCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let mut program_ptr = 0;
        let program = self.program;
        let mut stack = Vec::<i32>::new();

        while program_ptr < program.len() {
            let bin_code = BinaryCode::from(program[program_ptr]);
            match bin_code {
                BinaryCode::Push => {
                    program_ptr += 1;
                    let param = i32::from_be_bytes(
                        program.as_slice()[program_ptr..program_ptr + 4]
                            .try_into()
                            .unwrap(),
                    );
                    program_ptr += 4;
                    stack.push(param);
                }
                BinaryCode::Add => {
                    let b_val = stack.pop().unwrap() as u32;
                    let a_val = stack.pop().unwrap() as u32;
                    let a = FpVar::new_witness(cs.clone(), || Ok(F::from(a_val))).unwrap();
                    let b = FpVar::new_witness(cs.clone(), || Ok(F::from(b_val))).unwrap();
                    let c_val = a_val + b_val;
                    let c_t = a + b;
                    let c = FpVar::new_witness(cs.clone(), || Ok(F::from(c_val))).unwrap();
                    c.enforce_equal(&c_t)?;
                    stack.push(a_val as i32 + b_val as i32);
                    program_ptr += 1;
                }
                BinaryCode::Sub => {
                    let b_val = stack.pop().unwrap() as u32;
                    let a_val = stack.pop().unwrap() as u32;
                    let a = FpVar::new_witness(cs.clone(), || Ok(F::from(a_val))).unwrap();
                    let b = FpVar::new_witness(cs.clone(), || Ok(F::from(b_val))).unwrap();
                    if a_val > b_val {
                        let c_val = a_val - b_val;
                        let c_t = a - b;
                        let c = FpVar::new_witness(cs.clone(), || Ok(F::from(c_val))).unwrap();
                        c.enforce_equal(&c_t)?;
                    } else {
                        let c_val = b_val - a_val;
                        let c_t = b - a;
                        let c = FpVar::new_witness(cs.clone(), || Ok(F::from(c_val))).unwrap();
                        c.enforce_equal(&c_t)?;
                    }
                    stack.push(a_val as i32 - b_val as i32);
                    program_ptr += 1;
                }
                BinaryCode::Mul => {
                    let b_val = stack.pop().unwrap() as u32;
                    let a_val = stack.pop().unwrap() as u32;
                    let a = FpVar::new_witness(cs.clone(), || Ok(F::from(a_val))).unwrap();
                    let b = FpVar::new_witness(cs.clone(), || Ok(F::from(b_val))).unwrap();
                    let c_val = a_val * b_val;
                    let c_t = a * b;
                    let c = FpVar::new_witness(cs.clone(), || Ok(F::from(c_val))).unwrap();
                    c.enforce_equal(&c_t)?;
                    stack.push(a_val as i32 * b_val as i32);
                    program_ptr += 1;
                }
                BinaryCode::Div => {
                    let b_val = stack.pop().unwrap() as u32;
                    let a_val = stack.pop().unwrap() as u32;
                    let a = FpVar::new_witness(cs.clone(), || Ok(F::from(a_val))).unwrap();
                    let b = FpVar::new_witness(cs.clone(), || Ok(F::from(b_val))).unwrap();
                    let c_val = a_val / b_val;
                    let c = FpVar::new_witness(cs.clone(), || Ok(F::from(c_val))).unwrap();
                    let a_t = c * b;
                    a.enforce_equal(&a_t)?;
                    stack.push(a_val as i32 / b_val as i32);
                    program_ptr += 1;
                }
                BinaryCode::Pop => {
                    stack.pop();
                    program_ptr += 1;
                }
                BinaryCode::Swap => {
                    let a_val = stack.pop().unwrap() as u32;
                    let b_val = stack.pop().unwrap() as u32;
                    let a = FpVar::new_witness(cs.clone(), || Ok(F::from(a_val))).unwrap();
                    let b = FpVar::new_witness(cs.clone(), || Ok(F::from(b_val))).unwrap();
                    stack.push(a_val as i32);
                    stack.push(b_val as i32);
                    let n = stack.len();
                    let a_new_val = stack[n - 2] as u32;
                    let b_new_val = stack[n - 1] as u32;
                    let a_t = FpVar::new_witness(cs.clone(), || Ok(F::from(a_new_val))).unwrap();
                    let b_t = FpVar::new_witness(cs.clone(), || Ok(F::from(b_new_val))).unwrap();
                    b.enforce_equal(&b_t)?;
                    a.enforce_equal(&a_t)?;
                    program_ptr += 1;
                }
                BinaryCode::Ret => {
                    let result_val = stack.pop().unwrap() as u32;
                    let result_target =
                        FpVar::new_witness(cs.clone(), || Ok(F::from(result_val))).unwrap();
                    let result =
                        FpVar::new_witness(cs.clone(), || Ok(F::from(self.result))).unwrap();
                    result.enforce_equal(&result_target)?;
                    program_ptr += 1;
                }
                _ => {
                    program_ptr += 1;
                }
            };
        }
        Ok(())
    }
}

pub fn verify_dvm_circuit_groth16() {
    use ark_groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
    };

    let program = vec![
        0x05u8, 0x00, 0x00, 0x00, 0x56, 0x05, 0x00, 0x00, 0x00, 0x77, 0x01, 0x05, 0x00, 0x00, 0x00,
        0x22, 0x03, 0x05, 0x00, 0x00, 0x00, 0x02, 0x04, 0x05, 0x00, 0x00, 0xaf, 0xde, 0x08, 0x02,
        0x05, 0x00, 0x12, 0xae, 0x24, 0x05, 0x00, 0x11, 0x0e, 0x12, 0x01, 0x05, 0x00, 0x23, 0x45,
        0x23, 0x02, 0x07,
    ];

    // This may not be cryptographically safe, use
    // `OsRng` (for example) in production software.
    let rng = &mut test_rng();

    // Create parameters for our circuit
    let params = {
        let c = DVMCircuit::<Fr> {
            program,
            result: Fr::from(30483),
        };

        generate_random_parameters::<Bls12_377, _, _>(c, rng).unwrap()
    };

    // Prepare the verification key (for proof verification)
    let pvk = prepare_verifying_key(&params.vk);

    let program = vec![
        0x05u8, 0x00, 0x00, 0x00, 0x56, 0x05, 0x00, 0x00, 0x00, 0x77, 0x01, 0x05, 0x00, 0x00, 0x00,
        0x22, 0x03, 0x05, 0x00, 0x00, 0x00, 0x02, 0x04, 0x05, 0x00, 0x00, 0xaf, 0xde, 0x08, 0x02,
        0x05, 0x00, 0x12, 0xae, 0x24, 0x05, 0x00, 0x11, 0x0e, 0x12, 0x01, 0x05, 0x00, 0x23, 0x45,
        0x23, 0x02, 0x07,
    ];

    let c = DVMCircuit::new(program, 30483);

    // Create a groth16 proof with our parameters.
    let proof = create_random_proof(c, &params, rng).unwrap();
    println!("Proved DVM code with proof: {:?}", proof);
    assert!(verify_proof(&pvk, &proof, &[]).unwrap());
    println!("Verified proof!.");
}
