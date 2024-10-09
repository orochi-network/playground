import * as fs from "fs";
import { CircuitFileGenerator } from "src/types/circuit-type";

const SMTProcessorGenerator: CircuitFileGenerator = {
  type: "smt-single-processor",
  circuitFileGenerator: ({ tree_depth, path }) => {
    if (!tree_depth) {
      throw new Error("tree_depth is required for SMTProcessor");
    }
    if (!path) {
      throw new Error("path is required for SMTProcessor");
    }
    const content = `pragma circom 2.0.0;
    include "../../node_modules/circomlib/circuits/smt/smtprocessor.circom";
    component main = SMTProcessor(${tree_depth});
    `;
    fs.writeFileSync(path, content);
  },
};

const SMTBatchProcessorGenerator: CircuitFileGenerator = {
  type: "smt-batch-processor",
  circuitFileGenerator: ({ tree_depth, batch_size, path }) => {
    if (!tree_depth) {
      throw new Error("tree_depth is required for SMTBatchProcessor");
    }
    if (!batch_size) {
      throw new Error("batch_size is required for SMTBatchProcessor");
    }
    if (!path) {
      throw new Error("path is required for SMTBatchProcessor");
    }
    const content = `pragma circom 2.0.0;
    include "../../src/circuits/smtbatchprocessor.circom";
    component main = SMTBatchProcessor(${tree_depth}, ${batch_size});
    `;
    fs.writeFileSync(path, content);
  },
};

export { SMTProcessorGenerator, SMTBatchProcessorGenerator };
