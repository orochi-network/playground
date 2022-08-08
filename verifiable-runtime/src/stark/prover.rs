use winterfell::math::FieldElement;

use crate::helper::dvm_state_build;

// Copyright (c) Facebook, Inc. and its affiliates. All rights reserved.
//
// This work is licensed under the terms of the MIT license.
// For a copy, see <https://opensource.org/licenses/MIT>.
use super::{
    BaseElement, DVMAir, ProofOptions, Prover, PublicInputs, Trace, TraceTable, CYCLE_LENGTH,
};

// RESCUE PROVER
// ================================================================================================

pub struct DVMProver {
    options: ProofOptions,
}

impl DVMProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }

    pub fn build_trace(
        &self,
        seed: [BaseElement; 2],
        track: &Vec<[BaseElement; 2]>,
        trace_length: usize,
    ) -> TraceTable<BaseElement> {
        // allocate memory to hold the trace table
        let mut trace = TraceTable::new(2, trace_length);

        trace.fill(
            |state| {
                // initialize first state of the computation
                state[0] = seed[0];
                state[1] = seed[1];
            },
            |step, state| {
                // Repeating blake256 on current state of DVM
                (state[0], state[1]) =
                    dvm_state_build(&[state[0], state[1], track[step][0], track[step][1]]);
            },
        );

        trace
    }
}

impl Prover for DVMProver {
    type BaseField = BaseElement;
    type Air = DVMAir;
    type Trace = TraceTable<BaseElement>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
        let last_step = trace.length() - 1;
        PublicInputs {
            seed: [trace.get(0, 0), trace.get(1, 0)],
            result: [trace.get(0, last_step), trace.get(1, last_step)],
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
