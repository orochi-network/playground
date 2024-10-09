# Circomlib SMT benchmark

## Description

- This repository contains source code to benchmark performance of proof of operation on Sparse Merkle Tree of [Iden3's Circom implementation](https://github.com/iden3/circomlib/blob/master/circuits/smt/smtprocessor.circom)

## Organisation

This respository contains 5 folders:
- `test/smtprocessor-bench.js`: benchmark runner, please read the guide in the source code in case you want to run it
- `test/benchmark.bash`: sample benchmark instance
- `test/circuits`: contains generated Circom source code of SMT processor components
- `test/circuits/circuit_artifacts`: contains inputs for proof generation & proof verification
- `test/circuits/compiled_ouputs`: contains outputs of circuit compilation process
- `test/circuits/circuit_benchmarks/yyyy-mm-dd`: contains detailed reports of benchmarks on date `yyyy-mm-dd`