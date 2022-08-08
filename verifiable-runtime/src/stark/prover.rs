// Copyright (c) Facebook, Inc. and its affiliates. All rights reserved.
//
// This work is licensed under the terms of the MIT license.
// For a copy, see <https://opensource.org/licenses/MIT>.
use super::{
    rescue, BaseElement, DVMAir, FieldElement, ProofOptions, Prover, PublicInputs, Trace,
    TraceTable, CYCLE_LENGTH, NUM_HASH_ROUNDS,
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
        iterations: usize,
    ) -> TraceTable<BaseElement> {
        // allocate memory to hold the trace table
        let trace_length = iterations * CYCLE_LENGTH;
        let mut trace = TraceTable::new(4, trace_length);

        trace.fill(
            |state| {
                // initialize first state of the computation
                state[0] = seed[0];
                state[1] = seed[1];
                state[2] = track[0][0];
                state[3] = track[0][1];
            },
            |step, state| {
                // execute the transition function for all steps
                //
                // for the first 14 steps in every cycle, compute a single round of
                // Rescue hash; for the remaining 2 rounds, just carry over the values
                // in the first two registers to the next step
                if (step % CYCLE_LENGTH) < NUM_HASH_ROUNDS {
                    rescue::apply_round(state, step);
                } else {
                    state[2] = BaseElement::ZERO;
                    state[3] = BaseElement::ZERO;
                }
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
