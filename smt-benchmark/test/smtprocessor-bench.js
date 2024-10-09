// Usage guide:
// Step 0: install circom & snarkjs
// Step 1: put file powersOfTau28_hez_final_21.ptau from snarkjs repo to test/circuits/circuit_artifacts
// Step 2: npm run smtbench smt_depth=<smt_depth> proof_system=<proof_system> (smt_depth = 1 ... 254, proof_system = groth16/plonk/fflonk)

const chai = require("chai");
const path = require("path");
const wasm_tester = require("circom_tester_fix").wasm;
const util = require("util");
const exec = util.promisify(require("child_process").exec);
const newMemEmptyTrie = require("circomlibjs").newMemEmptyTrie;
const createCsvWriter = require("csv-writer").createObjectCsvWriter;
const fs = require("fs");
const chaiAssert = chai.assert;
const si = require("systeminformation");
const { z } = require("zod");
const argParser = require("../src/utils/argparser");
const {
  generateSMTProcessorCircuitFile,
} = require("../src/utils/smtprocessor-circuit-generator");

/////////////////////////////////////////////////////////////////////////////////////////////////////////
let TREE_DEPTH;
let PROOF_SYSTEM;

try {
  const argsSchema = z.object({
    smt_depth: z.string().transform((val) => {
      const parsed = parseInt(val, 10);
      if (isNaN(parsed) || parsed < 1 || parsed > 254) {
        throw new Error("smt_depth must be a number between 1 and 254");
      }
      return parsed;
    }),
    proof_system: z.enum(["groth16", "plonk", "fflonk"], {
      errorMap: (issue, ctx) => ({
        message: `Invalid proofSystem. Expected one of groth16, plonk, or fflonk.`,
      }),
    }),
  });
  const args = argParser.parseArgs();

  const parsedArgs = argsSchema.parse(args);
  const { smt_depth, proof_system } = parsedArgs;
  TREE_DEPTH = smt_depth;
  PROOF_SYSTEM = proof_system;
} catch (err) {
  console.log(err);
  console.log(
    "Usage: npm run smtbench smt_depth=<smt_depth> proof_system=<proof_system>"
  );
  console.log(`
Parameters:
  <smt_depth>    - SMT depth value (range: 1 ... 254)
  <proof_system>  - Choose the proof system (options: groth16, plonk, fflonk)
`);
  process.exit(1);
}

console.log(`SMT Depth: ${TREE_DEPTH}`);
console.log(`Proof System: ${PROOF_SYSTEM}`);

const curve = "bn128";
const PTAU_EXPONENT = 21; // Groth16: depth = 10 -> 2^13, depth = 254 -> 2^17
// const PROVER = `../../rapidsnark/package_macos_arm64/bin/prover`; // snarkjs ${PROOF_SYSTEM} prove
const PROVER = `snarkjs ${PROOF_SYSTEM} prove`;
const generateSnarkJSProof = true; // if false, only constraint check is executed
const compileZKey = false; // CHANGE THIS WHEN YOU WANT TO RECOMPILE THE ZKEY (DO THIS EACH TIME THE CIRCUIT CONTENT CHANGES)
/////////////////////////////////////////////////////////////////////////////////////////////////////////

// related paths

const circuit_folder = path.join(__dirname, "circuits");
const compiled_outputs_folder = path.join(circuit_folder, "compiled_outputs");
const circuit_artifacts_folder = path.join(circuit_folder, "circuit_artifacts");
const circuit_benchmark_folder = path.join(
  circuit_folder,
  "circuit_benchmarks/" + new Date().toISOString().split("T")[0]
);
// make sure these folders exist
const folders = [
  compiled_outputs_folder,
  circuit_artifacts_folder,
  circuit_benchmark_folder,
];
for (const folder of folders) {
  if (!fs.existsSync(folder)) {
    fs.mkdirSync(folder, { recursive: true });
  }
}

const circuit_file = `${circuit_folder}/smtprocessor_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}.circom`;
const r1cs_file = `${compiled_outputs_folder}/smtprocessor_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}.r1cs`;
const wasm_file = `${compiled_outputs_folder}/smtprocessor_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}_js/smtprocessor_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}.wasm`;
const ptau_file = `${circuit_artifacts_folder}/powersOfTau28_hez_final_${PTAU_EXPONENT}.ptau`;
const zkey_file = `${circuit_artifacts_folder}/smtprocessor_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}_circuit.zkey`;
const vkey_file = `${circuit_artifacts_folder}/smtprocessor_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}_circuit_verification_key.json`;
const input_file = `${circuit_artifacts_folder}/smtprocessor_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}_input.json`;
const wtns_file = `${circuit_artifacts_folder}/smtprocessor_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}.wtns`;
const public_input_file = `${circuit_artifacts_folder}/smtprocessor_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}_public.json`;
const proof_file = `${circuit_artifacts_folder}/smtprocessor_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}_proof.json`;
const benchmark_file = `${circuit_benchmark_folder}/smtprocessor_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}_benchmarks.csv`;
const hardware_file = `${circuit_benchmark_folder}/hardware_info.json`;

// generate circuit_file if it doesn't exist
if (!fs.existsSync(circuit_file)) {
  generateSMTProcessorCircuitFile(TREE_DEPTH, circuit_file);
}
/////////////////////////////////////////////////////////////////////////////////////////////////////////

// functions & tests

function print(circuit, w, s) {
  console.log(s + ": " + w[circuit.getSignalIdx(s)]);
}

function printKeySignal(circuit, w, key) {
  const signal = w[circuit.symbols[key].varIdx];
  console.log(`${key}: ${signal}`);
}

async function printHardware() {
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

const PROOF_TESTER_BENCHMARKS = [];

// log the time taken for proof generation and verification of a single task
async function proofTester(witnessData, task) {
  chaiAssert(true, task === "insert" || task === "delete" || task === "update");
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

  const proofGenerationLog = {
    task: task,
    smtDepth: TREE_DEPTH,
    curve: curve,
    proofSystem: PROOF_SYSTEM,
    type: "proof-generation",
    duration: t2 - t1, // ms
  };
  PROOF_TESTER_BENCHMARKS.push(proofGenerationLog);

  const proofVerificationLog = {
    task: task,
    smtDepth: TREE_DEPTH,
    curve: curve,
    proofSystem: PROOF_SYSTEM,
    type: "proof-verification",
    duration: t3 - t2, // ms
  };
  PROOF_TESTER_BENCHMARKS.push(proofVerificationLog);

  console.log(proofGenerationLog);
  console.log(proofVerificationLog);
}

async function testInsert(tree, _key, _value, circuit) {
  const key = tree.F.e(_key);
  const value = tree.F.e(_value);

  const res = await tree.insert(key, value);
  let siblings = res.siblings;
  for (let i = 0; i < siblings.length; i++)
    siblings[i] = tree.F.toObject(siblings[i]);
  while (siblings.length < TREE_DEPTH) siblings.push(0);

  const witnessData = {
    fnc: [1, 0],
    oldRoot: tree.F.toObject(res.oldRoot),
    siblings: siblings,
    oldKey: res.isOld0 ? 0 : tree.F.toObject(res.oldKey),
    oldValue: res.isOld0 ? 0 : tree.F.toObject(res.oldValue),
    isOld0: res.isOld0 ? 1 : 0,
    newKey: tree.F.toObject(key),
    newValue: tree.F.toObject(value),
  };

  if (generateSnarkJSProof) {
    await proofTester(witnessData, "insert");
  } else {
    const w = await circuit.calculateWitness(witnessData, true);
    await circuit.checkConstraints(w);
    await circuit.assertOut(w, { newRoot: tree.F.toObject(res.newRoot) });
  }
}

async function testDelete(tree, _key, circuit) {
  const key = tree.F.e(_key);
  const res = await tree.delete(key);
  let siblings = res.siblings;
  for (let i = 0; i < siblings.length; i++)
    siblings[i] = tree.F.toObject(siblings[i]);
  while (siblings.length < TREE_DEPTH) siblings.push(0);

  const witnessData = {
    fnc: [1, 1],
    oldRoot: tree.F.toObject(res.oldRoot),
    siblings: siblings,
    oldKey: res.isOld0 ? 0 : tree.F.toObject(res.oldKey),
    oldValue: res.isOld0 ? 0 : tree.F.toObject(res.oldValue),
    isOld0: res.isOld0 ? 1 : 0,
    newKey: tree.F.toObject(res.delKey),
    newValue: tree.F.toObject(res.delValue),
  };

  if (generateSnarkJSProof) {
    await proofTester(witnessData, "delete");
  } else {
    const w = await circuit.calculateWitness(witnessData, true);
    await circuit.checkConstraints(w);
    await circuit.assertOut(w, { newRoot: tree.F.toObject(res.newRoot) });
  }
}

async function testUpdate(tree, _key, _newValue, circuit) {
  const key = tree.F.e(_key);
  const newValue = tree.F.e(_newValue);
  const res = await tree.update(key, newValue);
  let siblings = res.siblings;
  for (let i = 0; i < siblings.length; i++)
    siblings[i] = tree.F.toObject(siblings[i]);
  while (siblings.length < TREE_DEPTH) siblings.push(0);

  const witnessData = {
    fnc: [0, 1],
    oldRoot: tree.F.toObject(res.oldRoot),
    siblings: siblings,
    oldKey: tree.F.toObject(res.oldKey),
    oldValue: tree.F.toObject(res.oldValue),
    isOld0: 0,
    newKey: tree.F.toObject(res.newKey),
    newValue: tree.F.toObject(res.newValue),
  };

  if (generateSnarkJSProof) {
    await proofTester(witnessData, "update");
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
    const compilerOptions = {
      output: compiled_outputs_folder,
      verbose: true,
      inspect: true,
    };
    circuit = await wasm_tester(
      path.join(
        __dirname,
        "circuits",
        `smtprocessor_${TREE_DEPTH}_${curve}_${PROOF_SYSTEM}.circom`
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
    await printHardware();
  });

  it("Quick test: Should verify SMT operation", async () => {
    await testInsert(tree, 111, 983, circuit);
    await testUpdate(tree, 111, 837, circuit);
    await testDelete(tree, 111, circuit);
  });

  // finally write results to benchmark file
  after(async () => {
    const csvWriter = createCsvWriter({
      path: benchmark_file,
      header: [
        { id: "task", title: "Task" },
        { id: "smtDepth", title: "SMT Depth" },
        { id: "curve", title: "Curve" },
        { id: "proofSystem", title: "Proof System" },
        { id: "type", title: "Type" },
        { id: "duration", title: "Duration (ms)" },
      ],
    });

    console.log(PROOF_TESTER_BENCHMARKS);

    await csvWriter
      .writeRecords(PROOF_TESTER_BENCHMARKS) // returns a promise
      .then(() => {
        console.log(`Done writing benchmarks to file ${benchmark_file}`);
      });
  });
});
