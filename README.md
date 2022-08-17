# Playground

Orochi Network's playground and experiments

# Verifiable Runtime

Orochi Network Team introduced a PoC of **Verifiable Runtime**. The PoC implemented a _Dummy Virtual Machine (DVM)_, a minimal stack machine with tiny set of `OPCODE`. **DVM** can perform several deadly simple calculations and provide a _Zero Knowledge Proof (we used zk-SNARK in this PoC, [Groth16](https://github.com/arkworks-rs/groth16))_ for every executed `OPCODE` and state.

With this approach we can perform the computation off-chain and perform the verification on-chain. We hope with the result of this PoC we can implement verifiable runtime for EVM and WebAssembly.

## Testing

```text
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 2.60s
     Running `target/debug/verifiable-dvm`
PUSH    $0x000056
                                        [86]
PUSH    $0x000077
                                        [86, 119]
ADD     ($0x000056 + $0x000077)
                                        [205]
PUSH    $0x000022
                                        [205, 34]
MUL     ($0x0000cd * $0x000022)
                                        [6970]
PUSH    $0x000002
                                        [6970, 2]
DIV     ($0x001b3a / $0x000002)
                                        [3485]
PUSH    $0x00afde
                                        [3485, 45022]
SWAP    $0x00afde <-> 0x000d9d
                                        [45022, 3485]
SUB     ($0x00afde - $0x000d9d)
                                        [41537]
PUSH    $0x12ae24
                                        [41537, 1224228]
PUSH    $0x110e12
                                        [41537, 1224228, 1117714]
ADD     ($0x12ae24 + $0x110e12)
                                        [41537, 2341942]
PUSH    $0x234523
                                        [41537, 2341942, 2311459]
SUB     ($0x23bc36 - $0x234523)
                                        [41537, 30483]
RET     $0x007713
                                        [41537]
Result: 30483
Proved DVM code with proof: Proof { a: GroupAffine { x: Fp384(BigInteger384([4212296319677032387, 5132356508855749279, 14340390389823998666, 10946292700968097719, 4706432553098198891, 3194314605111472])), y: Fp384(BigInteger384([15088669080411454159, 5272495450509022489, 2206013327436588761, 15592264337391802676, 14427815099296082657, 44681044148203739])), infinity: false }, b: GroupAffine { x: QuadExtField { c0: Fp384(BigInteger384([16404280868235233322, 1797688941595961896, 4774517716222772765, 14388511095987761004, 5633750184855568696, 96146633473082540])), c1: Fp384(BigInteger384([4602687517106756605, 16336522289220270487, 18221766641829668647, 17679512565045385759, 5348254283381078905, 11411269001714325])) }, y: QuadExtField { c0: Fp384(BigInteger384([296408175288566808, 3362830945148156967, 17813962105572032906, 6625303835568658685, 13220854740379424403, 115754080442315537])), c1: Fp384(BigInteger384([2144260065565911853, 17489622930501175625, 17655734401953657012, 8014553315336947450, 10641145036799837896, 80836343615596385])) }, infinity: false }, c: GroupAffine { x: Fp384(BigInteger384([12304088042507582109, 4899207083484264484, 18025317530017838622, 2210759389553879120, 7236748955561380095, 41782477178787192])), y: Fp384(BigInteger384([13402592647338771041, 1565815337082493161, 2716459182320095829, 8227949373635235163, 2274507761441233128, 67937226989310537])), infinity: false } }
Verified proof!.
```

_build with ❤️_
