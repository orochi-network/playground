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

use crate::opcode::BinaryCode;

// Bring in some tools for using pairing-friendly curves
// We're going to use the BLS12-377 pairing-friendly elliptic curve.
use ark_bls12_377::{Bls12_377, Fr};
use ark_ff::{PrimeField, ToBytes};
use ark_r1cs_std::{alloc::AllocVar, eq::EqGadget, fields::fp::FpVar, prelude::FieldVar};
use ark_std::test_rng;

// We'll use these interfaces to construct our circuit.
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

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
            program,
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
                    let c_t = a.mul_by_inverse(&b).unwrap();
                    let c = FpVar::new_witness(cs.clone(), || Ok(F::from(c_val))).unwrap();
                    c.enforce_equal(&c_t)?;
                    stack.push(c_val as i32);
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
                    let result = FpVar::new_input(cs.clone(), || Ok(F::from(self.result))).unwrap();
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

fn to_prime_field_value<F: PrimeField>(v: i32) -> F {
    F::from(v as u32)
}

pub fn verify_dvm_circuit_groth16(result: i32) {
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
            program: program.clone(),
            result: to_prime_field_value(result),
        };

        generate_random_parameters::<Bls12_377, _, _>(c, rng).unwrap()
    };

    // Prepare the verification key (for proof verification)
    let pvk = prepare_verifying_key(&params.vk);

    let c = DVMCircuit::new(program.clone(), result);

    // Create a groth16 proof with our parameters.
    let proof = create_random_proof(c, &params, rng).unwrap();
    println!("Proved DVM code with proof: {:?}", proof);
    let result = to_prime_field_value(result);
    assert!(verify_proof(&pvk, &proof, &[result]).unwrap());
    println!("Verified proof!.");
}
