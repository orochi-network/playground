// Copyright (c) Facebook, Inc. and its affiliates. All rights reserved.
//
// This work is licensed under the terms of the MIT license.
// For a copy, see <https://opensource.org/licenses/MIT>.

use std::string::String;

use winterfell::{
    math::{fields::f128::BaseElement, FieldElement},
    FieldExtension, HashFunction, ProofOptions, Prover, StarkProof, Trace, TraceTable,
    VerifierError,
};

#[allow(clippy::module_inception)]
pub(crate) mod rescue;

mod air;
use air::{DVMAir, PublicInputs};

mod prover;
use prover::DVMProver;

use crate::DIGEST_SIZE;

// CONSTANTS
// ================================================================================================

const CYCLE_LENGTH: usize = 16;
const NUM_HASH_ROUNDS: usize = 14;
const TRACE_WIDTH: usize = 4;

// EXAMPLE OPTIONS
// ================================================================================================

pub fn new_proof_option() -> ProofOptions {
    ProofOptions::new(
        42,
        4,
        16,
        HashFunction::Blake3_256,
        FieldExtension::None,
        8,
        128,
    )
}

pub fn prove_dvm(seed: [BaseElement; DIGEST_SIZE], track: Vec<[BaseElement; DIGEST_SIZE]>) {
    let prover = DVMProver::new(new_proof_option());
    prover.build_trace(seed, &track, track.len());
}
