// Usage guide:
// Step 0: install circom & snarkjs
// Step 1: put file powersOfTau28_hez_final_21.ptau from snarkjs repo to test/circuits/circuit_artifacts
// Step 2: npm run smtbench smt_depth=<smt_depth> proof_system=<proof_system> circuit=<ciruit> (smt_depth = 1 ... 254, proof_system = groth16/plonk/fflonk, ciruit = smt-single-processor/smt-batch-processor)

import path from "path";
import * as util from "util";
const exec = util.promisify(require("child_process").exec);
import { createObjectCsvWriter as createCsvWriter } from "csv-writer";
import * as fs from "fs";
import si from "systeminformation";
import { z } from "zod";
import * as argParser from "../src/utils/argparser";
import {
  SMTBatchProcessorGenerator,
  SMTProcessorGenerator,
} from "../src/utils/smtprocessor-circuit-generator";
import {
  CircuitFileGenerator,
  CircuitTypeEnum,
  CircuitTypeEnumValidator,
} from "../src/types/circuit-type";
import {
  ProofSystemTypeEnum,
  ProofSystemTypeEnumValidator,
} from "../src/types/proof-system-type";
import { SMTTaskTypeEnum } from "src/types/smt-task-type";
import { ObjectStringifierHeader } from "csv-writer/src/lib/record";

/////////////////////////////////////////////////////////////////////////////////////////////////////////
export let TREE_DEPTH: number;
export let PROOF_SYSTEM: ProofSystemTypeEnum;
export let SMT_CIRCUIT: CircuitFileGenerator;

try {
  const argsSchema = z.object({
    smt_depth: z.string().transform((val) => {
      const parsed = parseInt(val, 10);
      if (isNaN(parsed) || parsed < 1 || parsed > 254) {
        throw new Error("smt_depth must be a number between 1 and 254");
      }
      return parsed;
    }),
    proof_system: ProofSystemTypeEnumValidator,
    circuit: CircuitTypeEnumValidator,
  });
  const args = argParser.parseArgs();

  const parsedArgs = argsSchema.parse(args);
  const { smt_depth, proof_system, circuit } = parsedArgs;
  TREE_DEPTH = smt_depth;
  PROOF_SYSTEM = proof_system;

  switch (circuit) {
    case "smt-single-processor":
      SMT_CIRCUIT = SMTProcessorGenerator;
      break;
    case "smt-batch-processor":
      SMT_CIRCUIT = SMTBatchProcessorGenerator;
      break;
    default:
      throw new Error(
        `Invalid circuit type. Must be "smt-single-processor" or "smt-batch-processor"`
      );
  }
} catch (err) {
  console.log(err);
  console.log(
    "Usage: npm run smtbench smt_depth=<smt_depth> proof_system=<proof_system>"
  );
  console.log(`
Parameters:
  <smt_depth>    - SMT depth value (range: 1 ... 254)
  <proof_system>  - Choose the proof system (options: groth16, plonk, fflonk)
  <circuit>       - Choose the circuit type (options: smt-single-processor, smt-batch-processor)
`);
  process.exit(1);
}

console.log(`SMT Depth: ${TREE_DEPTH}`);
console.log(`Proof System: ${PROOF_SYSTEM}`);
console.log(`Circuit: ${SMT_CIRCUIT.type}`);

export const curve = "bn128";
export const PTAU_EXPONENT = 21; // Groth16: depth = 10 -> 2^13, depth = 254 -> 2^17
export const PROVER = `../../rapidsnark/package_macos_arm64/bin/prover`; // snarkjs ${PROOF_SYSTEM} prove
// export const PROVER = `snarkjs ${PROOF_SYSTEM} prove`;
export const generateSnarkJSProof = true; // if false, only constraint check is executed
export const compileZKey = true; // CHANGE THIS WHEN YOU WANT TO RECOMPILE THE ZKEY (DO THIS EACH TIME THE SMT_CIRCUIT CONTENT CHANGES)
/////////////////////////////////////////////////////////////////////////////////////////////////////////

// related paths

export const circuit_folder = path.join(__dirname, "circuits");
export const compiled_outputs_folder = path.join(
  circuit_folder,
  "compiled_outputs"
);
export const circuit_artifacts_folder = path.join(
  circuit_folder,
  "circuit_artifacts"
);
export const circuit_benchmark_folder = path.join(
  circuit_folder,
  "circuit_benchmarks/" + new Date().toISOString().split("T")[0]
);
// make sure these folders exist
export const folders = [
  compiled_outputs_folder,
  circuit_artifacts_folder,
  circuit_benchmark_folder,
];
for (const folder of folders) {
  if (!fs.existsSync(folder)) {
    fs.mkdirSync(folder, { recursive: true });
  }
}

export const circuit_file = `${circuit_folder}/${SMT_CIRCUIT.type}_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}.circom`;
export const r1cs_file = `${compiled_outputs_folder}/${SMT_CIRCUIT.type}_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}.r1cs`;
export const wasm_file = `${compiled_outputs_folder}/${SMT_CIRCUIT.type}_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}_js/${SMT_CIRCUIT.type}_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}.wasm`;
export const ptau_file = `${circuit_artifacts_folder}/powersOfTau28_hez_final_${PTAU_EXPONENT}.ptau`;
export const zkey_file = `${circuit_artifacts_folder}/${SMT_CIRCUIT.type}_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}_circuit.zkey`;
export const vkey_file = `${circuit_artifacts_folder}/${SMT_CIRCUIT.type}_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}_circuit_verification_key.json`;
export const input_file = `${circuit_artifacts_folder}/${SMT_CIRCUIT.type}_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}_input.json`;
export const wtns_file = `${circuit_artifacts_folder}/${SMT_CIRCUIT.type}_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}.wtns`;
export const public_input_file = `${circuit_artifacts_folder}/${SMT_CIRCUIT.type}_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}_public.json`;
export const proof_file = `${circuit_artifacts_folder}/${SMT_CIRCUIT.type}_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}_proof.json`;
export const benchmark_file = `${circuit_benchmark_folder}/${SMT_CIRCUIT.type}_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}_benchmarks.csv`;
export const hardware_file = `${circuit_benchmark_folder}/hardware_info.json`;

/////////////////////////////////////////////////////////////////////////////////////////////////////////

// functions & tests

function print(circuit, w, s) {
  console.log(s + ": " + w[circuit.getSignalIdx(s)]);
}

function printKeySignal(circuit, w, key) {
  const signal = w[circuit.symbols[key].varIdx];
  console.log(`${key}: ${signal}`);
}

export async function printHardware(hardware_file: string) {
  try {
    const data = await si.cpu();
    console.log("CPU Information:");
    console.log("- manufacturer: " + data.manufacturer);
    console.log("- brand: " + data.brand);
    console.log("- speed: " + data.speed);
    console.log("- cores: " + data.cores);
    console.log("- physical cores: " + data.physicalCores);
    console.log("...");

    // print to file
    fs.writeFileSync(hardware_file, JSON.stringify(data, null, 2));
  } catch (e) {
    console.log(e);
  }
}

export interface ProofTesterLog {
  task: SMTTaskTypeEnum;
  noOperations: number; // for single operation, this should be 1
  smtDepth: string;
  curve: "bn128";
  proofSystem: ProofSystemTypeEnum;
  type: "proof-generation" | "proof-verification";
  smtCircuit: CircuitTypeEnum;
  duration: number;
}

export interface SMTSingleOperationWitnessData {
  fnc: number[];
  oldRoot: number;
  siblings: number[];
  oldKey: number;
  oldValue: number;
  isOld0: number;
  newKey: number;
  newValue: number;
}

export interface SMTBatchOperationWitnessData {
  fnc: number[][]; // Each operation gets its own function signal
  oldRoot: number; // The root for the batch operation
  siblings: number[][]; // Each operation has its own siblings array
  oldKey: number[]; // Each operation gets its own key
  oldValue: number[]; // Each operation gets its own value
  isOld0: number[]; // Each operation has its own flag
  newKey: number[]; // Each operation gets its own new key
  newValue: number[]; // Each operation gets its own new value
}

// log the time taken for proof generation and verification of a single task
export async function proofTester(
  witnessData: SMTSingleOperationWitnessData | SMTBatchOperationWitnessData,
  task: SMTTaskTypeEnum
): Promise<Array<ProofTesterLog>> {
  const numberOfSMTOperations = Array.isArray(witnessData.newKey)
    ? witnessData.newKey.length
    : 1;
  // write witness data to input_file
  fs.writeFileSync(input_file, JSON.stringify(witnessData));

  // push to witness file
  await exec(`snarkjs wtns calculate ${wasm_file} ${input_file} ${wtns_file}`);

  // generating proof
  const t1 = performance.now();

  await exec(
    `${PROVER} ${zkey_file} ${wtns_file} ${proof_file} ${public_input_file}`
  );
  // ./package/bin/prover <circuit.zkey> <witness.wtns> <proof.json> <public.json></public.json>
  const t2 = performance.now();
  // verify proof
  const b = await exec(
    `snarkjs ${PROOF_SYSTEM} verify ${vkey_file} ${public_input_file} ${proof_file}`
  );
  const t3 = performance.now();

  const generatedProofTesterLogs: Array<ProofTesterLog> = [
    {
      task: task,
      noOperations: numberOfSMTOperations,
      smtDepth: TREE_DEPTH.toString(),
      curve: curve,
      proofSystem: PROOF_SYSTEM,
      type: "proof-generation",
      smtCircuit: SMT_CIRCUIT.type,
      duration: t2 - t1, // ms
    },
    {
      task: task,
      noOperations: numberOfSMTOperations,
      smtDepth: TREE_DEPTH.toString(),
      curve: curve,
      proofSystem: PROOF_SYSTEM,
      type: "proof-verification",
      smtCircuit: SMT_CIRCUIT.type,
      duration: t3 - t2, // ms
    },
  ];

  console.log(generatedProofTesterLogs);
  return generatedProofTesterLogs;
}

export interface SMTOperationResult {
  witnessData: SMTSingleOperationWitnessData;
  res: any;
}

export async function applySMTInsert(
  tree,
  _key,
  _value
): Promise<SMTOperationResult> {
  const key = tree.F.e(_key);
  const value = tree.F.e(_value);

  const res = await tree.insert(key, value);
  let siblings = res.siblings;
  for (let i = 0; i < siblings.length; i++)
    siblings[i] = tree.F.toObject(siblings[i]);
  while (siblings.length < TREE_DEPTH) siblings.push(0);

  const witnessData: SMTSingleOperationWitnessData = {
    fnc: [1, 0],
    oldRoot: tree.F.toObject(res.oldRoot),
    siblings: siblings,
    oldKey: res.isOld0 ? 0 : tree.F.toObject(res.oldKey),
    oldValue: res.isOld0 ? 0 : tree.F.toObject(res.oldValue),
    isOld0: res.isOld0 ? 1 : 0,
    newKey: tree.F.toObject(key),
    newValue: tree.F.toObject(value),
  };
  return {
    witnessData: witnessData,
    res: res,
  };
}

// NOTE: this can works because the information about the value is already stored in the SMT db storage
export async function applySMTDelete(tree, _key): Promise<SMTOperationResult> {
  const key = tree.F.e(_key);
  const res = await tree.delete(key);
  let siblings = res.siblings;
  for (let i = 0; i < siblings.length; i++)
    siblings[i] = tree.F.toObject(siblings[i]);
  while (siblings.length < TREE_DEPTH) siblings.push(0);

  const witnessData: SMTSingleOperationWitnessData = {
    fnc: [1, 1],
    oldRoot: tree.F.toObject(res.oldRoot),
    siblings: siblings,
    oldKey: res.isOld0 ? 0 : tree.F.toObject(res.oldKey),
    oldValue: res.isOld0 ? 0 : tree.F.toObject(res.oldValue),
    isOld0: res.isOld0 ? 1 : 0,
    newKey: tree.F.toObject(res.delKey),
    newValue: tree.F.toObject(res.delValue),
  };

  return {
    witnessData: witnessData,
    res: res,
  };
}

export async function applySMTUpdate(
  tree,
  _key,
  _newValue
): Promise<SMTOperationResult> {
  const key = tree.F.e(_key);
  const newValue = tree.F.e(_newValue);
  const res = await tree.update(key, newValue);
  let siblings = res.siblings;
  for (let i = 0; i < siblings.length; i++)
    siblings[i] = tree.F.toObject(siblings[i]);
  while (siblings.length < TREE_DEPTH) siblings.push(0);

  const witnessData: SMTSingleOperationWitnessData = {
    fnc: [0, 1],
    oldRoot: tree.F.toObject(res.oldRoot),
    siblings: siblings,
    oldKey: tree.F.toObject(res.oldKey),
    oldValue: tree.F.toObject(res.oldValue),
    isOld0: 0,
    newKey: tree.F.toObject(res.newKey),
    newValue: tree.F.toObject(res.newValue),
  };
  return {
    witnessData: witnessData,
    res: res,
  };
}

export const pushProofTesterReportToCSV = async (
  PROOF_TESTER_BENCHMARKS: Array<any>,
  csvHeaderDefinition: ObjectStringifierHeader
) => {
  const csvWriter = createCsvWriter({
    path: benchmark_file,
    header: csvHeaderDefinition,
  });

  console.log(PROOF_TESTER_BENCHMARKS);

  await csvWriter
    .writeRecords(PROOF_TESTER_BENCHMARKS) // returns a promise
    .then(() => {
      console.log(`Done writing benchmarks to file ${benchmark_file}`);
    });
};
