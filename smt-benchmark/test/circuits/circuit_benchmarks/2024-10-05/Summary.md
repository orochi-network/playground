### Benchmarks on Iden3's SMT implementation in Circom and executed in SnarkJS

| Proof System | SMT Depth | Number of Constraints | Proof Generation Time (ms) | Proof Verification Time (ms)  | Curve |
|--------------|-----------|-----------------------|----------------------------|-------------------------------|-------|
|    groth16   |    10     |          6705         |          595.895638        |           238.5096            | bn128 |
|     plonk    |    10     |         59649         |        23155.911704        |           239.083488          | bn128 |
|    fflonk    |    10     |         59649         |        32036.210133        |           254.236632          | bn128 |
|    groth16   |   254     |        128301         |         4370.937712        |           229.242474          | bn128 |
|     plonk    |   254     |        1313624        |       814942.789944        |           335.704792          | bn128 |
|    fflonk    |   254     |        1313624        |            N/A             |               N/A             | bn128 |

### Some observations:
- `groth16` still outperforms `plonk` & `fflonk` (100x faster) in proof generation process
- For proof verfication, `plonk` & `fflonk` do not show superiority over `groth16`
- The difference might come from various aspects such as hardware, delay, implementation, proof system's charisteristics, ... Overall, the non-transparency of `groth16` still shows superior performance compared to the other two transparent proof systems

### Some potential TODOs:
- Testing with other curves (e.g. bls12-381, ...)
- Testing with other prover (snarkjs wasm version, rapidsnark, gnark, ...)
- Expand to recent DSLs & proof systems (Nova, Halo2, ...)
- Testing with other OS & hardwares