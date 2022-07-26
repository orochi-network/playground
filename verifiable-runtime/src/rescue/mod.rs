// Copyright (c) Facebook, Inc. and its affiliates. All rights reserved.
//
// This work is licensed under the terms of the MIT license.
// For a copy, see <https://opensource.org/licenses/MIT>.
use log::debug;
use std::string::String;
use std::time::Instant;
use winterfell::{
    math::{fields::f128::BaseElement, log2, FieldElement},
    FieldExtension, HashFunction, ProofOptions, Prover, StarkProof, Trace, TraceTable,
    VerifierError,
};

#[allow(clippy::module_inception)]
pub(crate) mod rescue;

mod air;
use air::{PublicInputs, RescueAir};

mod prover;
use prover::RescueProver;

// CONSTANTS
// ================================================================================================

const CYCLE_LENGTH: usize = 16;
const NUM_HASH_ROUNDS: usize = 14;
const TRACE_WIDTH: usize = 4;

pub trait Example {
    fn prove(&self) -> StarkProof;
    fn verify(&self, proof: StarkProof) -> Result<(), VerifierError>;
    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError>;
}

// EXAMPLE OPTIONS
// ================================================================================================

pub struct DVMProofOptions {
    hash_fn: String,
    num_queries: Option<usize>,
    blowup_factor: Option<usize>,
    grinding_factor: u32,
    field_extension: u32,
    folding_factor: usize,
}

impl DVMProofOptions {
    pub fn new() -> Self {
        DVMProofOptions {
            hash_fn: String::from("blake3_192"),
            num_queries: Option::from(42 as usize),
            blowup_factor: Option::from(4 as usize),
            grinding_factor: 16,
            field_extension: 1,
            folding_factor: 8,
        }
    }

    pub fn get_proof_option(&self, q: usize, b: usize) -> ProofOptions {
        let num_queries = self.num_queries.unwrap_or(q);
        let blowup_factor = self.blowup_factor.unwrap_or(b);
        let field_extension = match self.field_extension {
            1 => FieldExtension::None,
            2 => FieldExtension::Quadratic,
            3 => FieldExtension::Cubic,
            val => panic!("'{}' is not a valid field extension option", val),
        };
        let hash_fn = match self.hash_fn.as_str() {
            "blake3_192" => HashFunction::Blake3_192,
            "blake3_256" => HashFunction::Blake3_256,
            "sha3_256" => HashFunction::Sha3_256,
            val => panic!("'{}' is not a valid hash function option", val),
        };

        ProofOptions::new(
            num_queries,
            blowup_factor,
            self.grinding_factor,
            hash_fn,
            field_extension,
            self.folding_factor,
            256,
        )
    }
}

// RESCUE HASH CHAIN EXAMPLE
// ================================================================================================

pub fn get_example(chain_length: usize) -> Box<dyn Example> {
    Box::new(RescueExample::new(
        chain_length,
        DVMProofOptions::new().get_proof_option(42, 4),
    ))
}

pub struct RescueExample {
    options: ProofOptions,
    chain_length: usize,
    seed: [BaseElement; 2],
    result: [BaseElement; 2],
}

impl RescueExample {
    pub fn new(chain_length: usize, options: ProofOptions) -> RescueExample {
        assert!(
            chain_length.is_power_of_two(),
            "chain length must a power of 2"
        );
        let seed = [BaseElement::from(42u8), BaseElement::from(43u8)];

        // compute the sequence of hashes using external implementation of Rescue hash
        let now = Instant::now();
        let result = compute_hash_chain(seed, chain_length);
        debug!(
            "Computed a chain of {} Rescue hashes in {} ms",
            chain_length,
            now.elapsed().as_millis(),
        );

        RescueExample {
            options,
            chain_length,
            seed,
            result,
        }
    }
}

// EXAMPLE IMPLEMENTATION
// ================================================================================================

impl Example for RescueExample {
    fn prove(&self) -> StarkProof {
        // generate the execution trace
        debug!(
            "Generating proof for computing a chain of {} Rescue hashes\n\
            ---------------------",
            self.chain_length
        );

        // create a prover
        let prover = RescueProver::new(self.options.clone());

        // generate the execution trace
        let now = Instant::now();
        let trace = prover.build_trace(self.seed, self.chain_length);
        let trace_length = trace.length();
        debug!(
            "Generated execution trace of {} registers and 2^{} steps in {} ms",
            trace.width(),
            log2(trace_length),
            now.elapsed().as_millis()
        );

        // generate the proof
        prover.prove(trace).unwrap()
    }

    fn verify(&self, proof: StarkProof) -> Result<(), VerifierError> {
        let pub_inputs = PublicInputs {
            seed: self.seed,
            result: self.result,
        };
        winterfell::verify::<RescueAir>(proof, pub_inputs)
    }

    fn verify_with_wrong_inputs(&self, proof: StarkProof) -> Result<(), VerifierError> {
        let pub_inputs = PublicInputs {
            seed: self.seed,
            result: [self.result[0], self.result[1] + BaseElement::ONE],
        };
        winterfell::verify::<RescueAir>(proof, pub_inputs)
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn compute_hash_chain(seed: [BaseElement; 2], length: usize) -> [BaseElement; 2] {
    let mut values = seed;
    let mut result = [BaseElement::ZERO; 2];
    for _ in 0..length {
        rescue::hash(values, &mut result);
        values.copy_from_slice(&result);
    }
    result
}
