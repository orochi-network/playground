# Playground

Orochi Network's playground and experiments

# Verifiable Runtime

Orochi Network Team introduced a PoC of **Verifiable Runtime**. The PoC implemented a _Dummy Virtual Machine (DVM)_, a minimal stack machine with tiny set of `OPCODE`. **DVM** can perform several deadly simple calculations and provide a verifiable _Zero Knowledge Proof (we use zk-STARK in this PoC)_ for every executed `OPCODE`.

With this approach we can perform the computation off-chain and perform the verification on-chain. We hope with the result of this PoC we can implement verifiable runtime for EVM and WebAssembly.

## Testing

```text
$ cargo run
   Compiling verifiable-runtime v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 0.38s
     Running `target/debug/verifiable-runtime`
PUSH    $0x000056
                                        [86]
PUSH    $0x000077
                                        [86, 119]
ADD     ($0x000077 + $0x000056)
                                        [205]
PUSH    $0x000022
                                        [205, 34]
MUL     ($0x000022 * $0x0000cd)
                                        [6970]
PUSH    $0x000002
                                        [6970, 2]
SWAP    $0x000002 <-> 0x001b3a
                                        [2, 6970]
DIV     ($0x001b3a / $0x000002)
                                        [3485]
PUSH    $0x00afde
                                        [3485, 45022]
SUB     ($0x00afde - $0x000d9d)
                                        [41537]
RET     $0x00a241
                                        []
Result: 41537
```

_build with ❤️_
