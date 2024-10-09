// Usage guide:
// Step 0: install circom & snarkjs
// Step 1: put file powersOfTau28_hez_final_21.ptau from snarkjs repo to test/circuits/circuit_artifacts
// Step 2: npm run smtbench smt_depth=<smt_depth> proof_system=<proof_system> circuit=<ciruit> (smt_depth = 1 ... 254, proof_system = groth16/plonk/fflonk, ciruit = smt-single-processor/smt-batch-processor)

import { assert as chaiAssert } from "chai";
import path from "path";
import { wasm as wasm_tester } from "circom_tester_fix";
import * as util from "util";
const exec = util.promisify(require("child_process").exec);
import { newMemEmptyTrie } from "circomlibjs";
import { DEFAULT_SMT_BATCH_SIZE } from "../src/constants/circuit";
import {
  applySMTDelete,
  applySMTInsert,
  applySMTUpdate,
  SMT_CIRCUIT,
  circuit_file,
  compiled_outputs_folder,
  compileZKey,
  curve,
  generateSnarkJSProof,
  hardware_file,
  printHardware,
  PROOF_SYSTEM,
  proofTester,
  ProofTesterLog,
  ptau_file,
  pushProofTesterReportToCSV,
  r1cs_file,
  TREE_DEPTH,
  vkey_file,
  zkey_file,
  SMTOperationResult,
  SMTBatchOperationWitnessData,
  SMTSingleOperationWitnessData,
} from "./smtbench-setup";

/////////////////////////////////////////////////////////////////////////////////////////////////////////
const PROOF_TESTER_BENCHMARKS: Array<ProofTesterLog> = [];

async function testSingleSMTInsert(tree, _key, _value, circuit) {
  const { witnessData, res } = await applySMTInsert(tree, _key, _value);

  if (generateSnarkJSProof) {
    PROOF_TESTER_BENCHMARKS.push(...(await proofTester(witnessData, "insert")));
  } else {
    const w = await circuit.calculateWitness(witnessData, true);
    await circuit.checkConstraints(w);
    await circuit.assertOut(w, { newRoot: tree.F.toObject(res.newRoot) });
  }
}

async function testSingleSMTDelete(tree, _key, circuit) {
  const { witnessData, res } = await applySMTDelete(tree, _key);

  if (generateSnarkJSProof) {
    PROOF_TESTER_BENCHMARKS.push(...(await proofTester(witnessData, "delete")));
  } else {
    const w = await circuit.calculateWitness(witnessData, true);
    await circuit.checkConstraints(w);
    await circuit.assertOut(w, { newRoot: tree.F.toObject(res.newRoot) });
  }
}

async function testSingleSMTUpdate(tree, _key, _newValue, circuit) {
  const { witnessData, res } = await applySMTUpdate(tree, _key, _newValue);

  if (generateSnarkJSProof) {
    PROOF_TESTER_BENCHMARKS.push(...(await proofTester(witnessData, "update")));
  } else {
    const w = await circuit.calculateWitness(witnessData, true);
    await circuit.checkConstraints(w);
    await circuit.assertOut(w, { newRoot: tree.F.toObject(res.newRoot) });
  }
}

describe("SMT Processor test", function () {
  let circuit;
  let tree;
  let Fr;

  this.timeout(1000000000);

  before(async () => {
    // generate circuit_file
    SMT_CIRCUIT.circuitFileGenerator({
      tree_depth: TREE_DEPTH,
      batch_size: DEFAULT_SMT_BATCH_SIZE,
      path: circuit_file,
    });
    console.log(
      `Compiling ${SMT_CIRCUIT.type} circuit file ${circuit_file}...`
    );

    const compilerOptions = {
      output: compiled_outputs_folder,
      verbose: true,
      inspect: true,
    };
    circuit = await wasm_tester(
      path.join(
        __dirname,
        "circuits",
        `${SMT_CIRCUIT.type}_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}.circom`
      ),
      compilerOptions
    );
    await circuit.loadSymbols();
    tree = await newMemEmptyTrie();
    Fr = tree.F;
    if (compileZKey) {
      // Only Groth16 requires Non-transparent trusted setup
      try {
        console.debug(`Generating ${PROOF_SYSTEM} zkey...`);
        let b = await exec(
          `snarkjs ${PROOF_SYSTEM} setup ${r1cs_file} ${ptau_file} ${zkey_file}`
        );
        // fflonk setup                  BETA version. Creates a FFLONK zkey from a circuit
        //  Usage:  snarkjs ffs [circuit.r1cs] [powersoftau.ptau] [circuit.zkey]
        if (compilerOptions.verbose) {
          console.log(b.stdout);
        }
        if (b.stderr) {
          console.error(b.stderr);
        }
        console.log(`${PROOF_SYSTEM} zkey generated.`);
        console.log(`Exporting ${PROOF_SYSTEM} verification key...`);
        b = await exec(
          `snarkjs zkey export verificationkey ${zkey_file} ${vkey_file}`
        );
        console.log(`${PROOF_SYSTEM} verification key exported.`);
      } catch (e) {
        chaiAssert(false, "circom compiler error \n" + e);
      }
    }
  });

  it("Print hardward information", async () => {
    await printHardware(hardware_file);
  });

  switch (SMT_CIRCUIT.type) {
    case "smt-single-processor":
      it("Quick test: Should verify SMT operation", async () => {
        await testSingleSMTInsert(tree, 111, 983, circuit);
        await testSingleSMTUpdate(tree, 111, 837, circuit);
        await testSingleSMTDelete(tree, 111, circuit);
      });
      break;

    case "smt-batch-processor":
      it("Quick test: Should verify SMT batch operation", async () => {
        const simulatedSMT = new Set();
        const SMTOperations: Array<{
          type: "insert" | "delete" | "update";
          key: number;
          value?: number;
        }> = [];

        for (let i = 0; i < DEFAULT_SMT_BATCH_SIZE; i++) {
          const key = Math.floor(Math.random() * 1000);
          const value = Math.floor(Math.random() * 1000);
          while (true) {
            let op = Math.floor(Math.random() * 3);
            if (op === 0 && !simulatedSMT.has(key)) {
              SMTOperations.push({ type: "insert", key, value });
              simulatedSMT.add(key);
              break;
            } else if (op === 1 && simulatedSMT.has(key)) {
              SMTOperations.push({ type: "delete", key });
              simulatedSMT.delete(key);
              break;
            } else if (op === 2 && simulatedSMT.has(key)) {
              SMTOperations.push({ type: "update", key, value });
              break;
            }
          }
        }

        console.log(SMTOperations);

        const smtSingleOperationWitnessDataList: Array<SMTSingleOperationWitnessData> =
          [];

        for (let i = 0; i < SMTOperations.length; i++) {
          console.log("okay ", SMTOperations[i]);
          let smtOpRes: SMTOperationResult;
          switch (SMTOperations[i].type) {
            case "insert":
              smtOpRes = await applySMTInsert(
                tree,
                SMTOperations[i].key,
                SMTOperations[i].value
              );
              break;
            case "delete":
              smtOpRes = await applySMTDelete(tree, SMTOperations[i].key);
              break;
            case "update":
              smtOpRes = await applySMTUpdate(
                tree,
                SMTOperations[i].key,
                SMTOperations[i].value
              );
              break;
            default:
              throw new Error("Invalid SMT operation type");
          }

          const { witnessData, res } = smtOpRes;
          smtSingleOperationWitnessDataList.push(witnessData);
        }

        const smtBatchOperationWitnessData: SMTBatchOperationWitnessData = {
          fnc: smtSingleOperationWitnessDataList.map((w) => w.fnc),
          oldRoot: smtSingleOperationWitnessDataList[0].oldRoot,
          siblings: smtSingleOperationWitnessDataList.map((w) => w.siblings),
          oldKey: smtSingleOperationWitnessDataList.map((w) => w.oldKey),
          oldValue: smtSingleOperationWitnessDataList.map((w) => w.oldValue),
          isOld0: smtSingleOperationWitnessDataList.map((w) => w.isOld0),
          newKey: smtSingleOperationWitnessDataList.map((w) => w.newKey),
          newValue: smtSingleOperationWitnessDataList.map((w) => w.newValue),
        };

        // TODO: Add the case for constraint-check only when generateSnarkJSProof is false
        if (generateSnarkJSProof) {
          PROOF_TESTER_BENCHMARKS.push(
            ...(await proofTester(smtBatchOperationWitnessData, "batch"))
          );
        } else {
          const w = await circuit.calculateWitness(
            smtBatchOperationWitnessData,
            true
          );
          await circuit.checkConstraints(w);
          await circuit.assertOut(w, { newRoot: tree.F.toObject(tree.root) });
        }
      });
      break;
  }

  // finally write results to benchmark file
  after(async () => {
    const headerDefinition = [
      { id: "task", title: "Task" },
      { id: "smtDepth", title: "SMT Depth" },
      { id: "curve", title: "Curve" },
      { id: "proofSystem", title: "Proof System" },
      { id: "type", title: "Type" },
      { id: "smtCircuit", title: "SMT Circuit" },
      { id: "duration", title: "Duration (ms)" },
    ];
    await pushProofTesterReportToCSV(PROOF_TESTER_BENCHMARKS, headerDefinition);
  });
});
