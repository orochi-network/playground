# Playground

Orochi Network's playground and experiment

# Verifiable Runtime

Orochi Network Team introduced a PoC of **Verifiable Runtime**. The PoC implemented a _Dummy Virtual Machine (DVM)_, a minimal stack machine with tiny set of `OPCODE`. **DVM** can perform several deadly simple calculations and provide a verifiable _Zero Knowledge Proof (we use zk-STARK in this PoC)_ for every executed `OPCODE`.

With this approach we can perform the computation off-chain and performing the verification on-chain. We hope with the result of this PoC we can implement verifiable runtime fo EVM and WebAssembly.

_build with ❤️_
