use ff::{BatchInvert, Field};
use halo2_proofs::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{floor_planner::V1, Layouter, Value, Region},
    dev::{metadata, FailureLocation, MockProver, VerifyFailure},
    halo2curves::pasta::EqAffine,
    plonk::*,
    poly::{
        commitment::ParamsProver,
        ipa::{
            commitment::{IPACommitmentScheme, ParamsIPA},
            multiopen::{ProverIPA, VerifierIPA},
            strategy::AccumulatorStrategy,
        },
        Rotation, VerificationStrategy,
    },
    transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
    },
};
use halo2curves::pasta::Fp;
use rand_core::{OsRng, RngCore};
use std::{iter, cmp::Ordering};

// sample a random table represented by a vector of vector. 
// The vector has HEIGHT rows where each row is a vector
// Each row has WIDTH elements
fn rand_2d_vector<Field: FieldExt, Rng: RngCore> (
    rng: &mut Rng,
    width: usize,
    height: usize,
) -> Vec<Vec<Field>> {
    let res = vec![(); height].iter().map(
        |_| vec![(); width].iter().map(
            |_| Field::random(&mut *rng)
        ).collect()
    ).collect();
    res
}

// Create a table of height rows
// Each row is sampled from the rows of lookup_table_vec
fn sample_random_values_from_lookup_table<Field: FieldExt, Rng: RngCore> (
    lookup_table_vec: &Vec<Vec<Field>>,
    rng: &mut Rng,
    width: usize,
    height: usize,
) -> Vec<Vec<Field>> {
    for row in lookup_table_vec.iter() {
        assert_eq!(row.len(), width);
    }

    let mut res = vec![vec![Field::zero(); width]; height];
    for i in 0..height {
        let rand_row_index = (rng.next_u32() as usize) % lookup_table_vec.len();
        res[i] = lookup_table_vec[rand_row_index].clone();
    }

    res
}

// receive a table of 
fn pad_array<Field: FieldExt> (
    original: &Vec<Vec<Field>>,
    width: usize,
    new_height: usize,
) -> Vec<Vec<Field>> {
    assert!(new_height >= original.len(), "NEW_HEIGHT ({}) must be at least CURRENT_HEIGHT ({})", new_height, original.len());
    for row in original.iter() {
        assert_eq!(row.len(), width);
    }
    let mut res: Vec<Vec<Field>> = vec![vec![Field::zero(); width]; new_height]; 
    for i in 0..original.len() {
        res[i] = original[i].clone();
    }

    for i in original.len()..new_height {
        res[i] = res[0].clone();
    }

    res
}

#[derive(Clone)]
struct MyConfig<const WIDTH: usize> {
    q_lookup: Selector,
    q_first_lookup: Selector,
    q_last_lookup: Selector,
    q_first_row_equal: Selector,
    q_other_row_equal: Selector,
    lookup_table: [Column<Advice>; WIDTH],
    value_table: [Column<Advice>; WIDTH],
    arranged_lookup_table: [Column<Advice>; WIDTH],
    arranged_value_table: [Column<Advice>; WIDTH],
    theta: Challenge,
    beta: Challenge,
    gamma: Challenge,
    z_lookup: Column<Advice>,
}

impl<const Width: usize> MyConfig<Width> {
    fn configure<Field: FieldExt>(meta: &mut ConstraintSystem<Field>) -> Self {
        let [q_lookup, q_first_lookup, q_last_lookup, q_first_row_equal, q_other_row_equal] = [(); 5].map(|_| meta.selector());
        let lookup_table = [(); Width].map(|_| meta.advice_column_in(FirstPhase));
        let value_table = [(); Width].map(|_| meta.advice_column_in(FirstPhase));
        let arranged_lookup_table = [(); Width].map(|_| meta.advice_column_in(FirstPhase));
        let arranged_value_table = [(); Width].map(|_| meta.advice_column_in(FirstPhase));
        let [theta, beta, gamma] = [(); 3].map(|_| meta.challenge_usable_after(FirstPhase));
        let z_lookup = meta.advice_column_in(SecondPhase);

        meta.create_gate("z should start with 1", |meta| {
            let q_first_lookup = meta.query_selector(q_first_lookup);
            let z_lookup = meta.query_advice(z_lookup, Rotation::cur());
            let one = Expression::Constant(Field::one());
            vec![q_first_lookup * (one - z_lookup)]
        });

        meta.create_gate("z should end with 1", |meta| {
            let q_last_lookup = meta.query_selector(q_last_lookup);
            let z_lookup = meta.query_advice(z_lookup, Rotation::cur());
            let one = Expression::Constant(Field::one());

            vec![q_last_lookup * (one - z_lookup)]
        });

        meta.create_gate("z should have valid transition", |meta| {
            let q_lookup = meta.query_selector(q_lookup);
            let lookup_table = lookup_table.map(|advice| meta.query_advice(advice, Rotation::cur()));
            let value_table = value_table.map(|advice| meta.query_advice(advice, Rotation::cur()));
            let arranged_lookup_table = arranged_lookup_table.map(|advice| meta.query_advice(advice, Rotation::cur()));
            let arranged_value_table = arranged_value_table.map(|advice| meta.query_advice(advice, Rotation::cur()));
            let [theta, beta, gamma] = [theta, beta, gamma].map(|challenge| meta.query_challenge(challenge));
            let [z, z_w] = [Rotation::cur(), Rotation::next()].map(|rotation| meta.query_advice(z_lookup, rotation));

            let lookup_merged = lookup_table.iter().cloned().reduce(|acc, v| acc * theta.clone() + v).unwrap();
            let value_merged = value_table.iter().cloned().reduce(|acc, v| acc * theta.clone() + v).unwrap();
            let arranged_lookup_merged = arranged_lookup_table.iter().cloned().reduce(|acc, v| acc * theta.clone() + v).unwrap();
            let arranged_value_merged = arranged_value_table.iter().cloned().reduce(|acc, v| acc * theta.clone() + v).unwrap();
            vec![q_lookup * (z_w * (arranged_lookup_merged + gamma.clone()) * (arranged_value_merged + beta.clone()) - z * (lookup_merged + gamma) * (value_merged + beta))]
        });

        meta.create_gate("first rows of arranged_lookup_table and arranged_value_table are equal", |meta| {
            let q_first_row_equal = meta.query_selector(q_first_row_equal);
            let arranged_lookup_table = arranged_lookup_table.map(|advice| meta.query_advice(advice, Rotation::cur()));
            let arranged_value_table = arranged_value_table.map(|advice| meta.query_advice(advice, Rotation::cur()));
            let theta = meta.query_challenge(theta);

            let arranged_lookup_merged = arranged_lookup_table.iter().cloned().reduce(|acc, v| acc * theta.clone() + v).unwrap();
            let arranged_value_merged = arranged_value_table.iter().cloned().reduce(|acc, v| acc * theta.clone() + v).unwrap();
            vec![q_first_row_equal * (arranged_lookup_merged - arranged_value_merged)]
        });

        meta.create_gate("each of other rows of arranged_value_table is equal to previous row or equal to the same one in arranged_lookup_table", |meta| {
            let q_other_row_equal = meta.query_selector(q_other_row_equal);
            let arranged_lookup_table = arranged_lookup_table.map(|advice| meta.query_advice(advice, Rotation::cur()));
            let [arranged_value_table, prev_arranged_value_table] = [Rotation::cur(), Rotation::prev()].map(
                |rotation| arranged_value_table.map(|advice| meta.query_advice(advice, rotation))
            );
            let theta = meta.query_challenge(theta);

            let arranged_lookup_merged = arranged_lookup_table.iter().cloned().reduce(|acc, v| acc * theta.clone() + v).unwrap();
            let arranged_value_merged = arranged_value_table.iter().cloned().reduce(|acc, v| acc * theta.clone() + v).unwrap();
            let prev_arranged_value_merged = prev_arranged_value_table.iter().cloned().reduce(|acc, v| acc * theta.clone() + v).unwrap();
            vec![q_other_row_equal * (arranged_lookup_merged - arranged_value_merged.clone()) * (arranged_value_merged - prev_arranged_value_merged)]
        });

        Self {
            q_lookup: q_lookup,
            q_first_lookup: q_first_lookup,
            q_last_lookup: q_last_lookup,
            q_first_row_equal: q_first_row_equal,
            q_other_row_equal: q_other_row_equal,
            lookup_table: lookup_table,
            value_table: value_table,
            arranged_lookup_table: arranged_lookup_table,
            arranged_value_table: arranged_value_table,
            theta: theta,
            beta: beta,
            gamma: gamma,
            z_lookup: z_lookup,
        }
    }
}

#[derive(Default, Clone)]
struct MyCircuit<Field: FieldExt, const WIDTH: usize, const HEIGHT: usize> {
    lookup_table: Value<[[Field; HEIGHT]; WIDTH]>,
    value_table: Value<[[Field; HEIGHT]; WIDTH]>,
    arranged_lookup_table: Value<[[Field; HEIGHT]; WIDTH]>,
    arranged_value_table: Value<[[Field; HEIGHT]; WIDTH]>,
}

fn print_vec_table<Field: FieldExt>(table: &Vec<Vec<Field>>) {
    for row in table {
        for element in row {
            print!("{} ", element.get_lower_128());
        }
        println!();
    }
}

fn print_array_table<Field: FieldExt, const WIDTH: usize, const HEIGHT: usize>(table: &Value<[[Field; HEIGHT]; WIDTH]>) {
    table.map(|table_value| {
        for i in 0..HEIGHT {
            for j in 0..WIDTH {
                print!("{} ", table_value[j][i].get_lower_32());
            }
            println!();
        }
    });
}


impl<Field: FieldExt, const WIDTH: usize, const HEIGHT: usize> MyCircuit<Field, WIDTH, HEIGHT> {
    fn rand<Rng: RngCore>(
        rng: &mut Rng,
        lookup_table_height: usize,
        value_table_height: usize,
    ) -> Self {
        assert!(lookup_table_height <= HEIGHT, "lookup_table_height must be at most HEIGHT");
        assert!(value_table_height <= HEIGHT, "value_table_height must be at most HEIGHT");
        
        // generate random lookup and value tables where rows of value table are sampled from rows of lookup table
        let original_lookup_table = rand_2d_vector::<Field, Rng>(rng, WIDTH, lookup_table_height);
        let original_value_table = sample_random_values_from_lookup_table::<Field, Rng>(&original_lookup_table, rng, WIDTH, value_table_height);

        let lookup_table = pad_array::<Field>(&original_lookup_table, WIDTH, HEIGHT);
        let value_table = pad_array::<Field>(&original_value_table, WIDTH, HEIGHT);

        // defining the comparison function
        let cmp_rows = |first: &Vec<Field>, second: &Vec<Field>| -> Ordering {
            assert_eq!(first.len(), WIDTH);
            assert_eq!(second.len(), WIDTH);
            let mut res = Ordering::Equal;
            for i in 0..first.len() {
                let ordering = first[i].get_lower_128().cmp(&second[i].get_lower_128()); 
                if ordering.ne(&Ordering::Equal) {
                    res = ordering;
                    break;
                }
            }

            res
        };

        // sort the tables
        let [mut sorted_lookup_table, mut sorted_value_table] = [lookup_table.clone(), value_table.clone()];
        sorted_lookup_table.sort_by(cmp_rows);
        sorted_value_table.sort_by(cmp_rows);

        // print_vec_table::<Field>(&sorted_lookup_table);
        // println!("----------");
        // print_vec_table::<Field>(&sorted_value_table);

        // check validity of 2 sorted tables and re-arrange sorted_lookup_table
        // I use 2-pointer trick here
        let mut arranged_lookup_table = vec![vec![Field::zero(); WIDTH]; HEIGHT];
        let mut selected_rows = [false; HEIGHT];
        let mut filled_rows = [false; HEIGHT];

        let mut checking_index = 0;
        let mut is_row_changed = true;
        
        for (index, value_row) in sorted_value_table.iter().enumerate() {
            while cmp_rows(&sorted_lookup_table[checking_index], value_row) == Ordering::Less {
                checking_index += 1;
                is_row_changed = true;
            }
            // println!("index: {}, checking index: {}", index, checking_index);
            // assert_eq!(cmp_rows(&sorted_value_table[checking_index], value_row), Ordering::Equal);
            if is_row_changed {
                selected_rows[checking_index] = true;
                filled_rows[index] = true;
                arranged_lookup_table[index] = value_row.clone();
                is_row_changed = false;
            }
        }

        checking_index = 0;
        for index in 0..sorted_value_table.len() {
            if !filled_rows[index] {
                while selected_rows[checking_index] {
                    checking_index += 1;
                }
                arranged_lookup_table[index] = sorted_lookup_table[checking_index].clone();
                checking_index += 1;
            }
        }

        // now arrange the result
        let transform_to_array = |table: &Vec<Vec<Field>>| -> [[Field; HEIGHT]; WIDTH] {
            let mut res = [[Field::zero(); HEIGHT]; WIDTH];
            for i in 0..HEIGHT {
                for j in 0..WIDTH {
                    res[j][i] = table[i][j];
                }
            }
            res
        };

        Self {
            lookup_table: Value::known(transform_to_array(&lookup_table)),
            value_table: Value::known(transform_to_array(&value_table)),
            arranged_lookup_table: Value::known(transform_to_array(&arranged_lookup_table)),
            arranged_value_table: Value::known(transform_to_array(&sorted_value_table)),
        }
    }
}

impl<Field: FieldExt, const WIDTH: usize, const HEIGHT: usize> Circuit<Field> for MyCircuit<Field, WIDTH, HEIGHT> {
    type Config = MyConfig<WIDTH>;

    type FloorPlanner = V1;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Field>) -> Self::Config {
        MyConfig::configure(meta)
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<Field>) -> Result<(), Error> {
        let theta = layouter.get_challenge(config.theta);
        let beta = layouter.get_challenge(config.beta);
        let gamma = layouter.get_challenge(config.gamma);
        layouter.assign_region(
            || "lookup value_table in lookup_table", 
            |mut region| {
                config.q_first_lookup.enable(&mut region, 0)?;
                config.q_last_lookup.enable(&mut region, HEIGHT)?;
                for offset in 0..HEIGHT {
                    config.q_lookup.enable(&mut region, offset)?;
                }
                config.q_first_row_equal.enable(&mut region, 0)?;
                for offset in 1..HEIGHT {
                    config.q_other_row_equal.enable(&mut region, offset)?;
                }

                // First phase
                for (idx, (&column, values)) in config
                    .lookup_table
                    .iter()
                    .zip(self.lookup_table.transpose_array().iter())
                    .enumerate()
                {
                    for (offset, &value) in values.transpose_array().iter().enumerate() {
                        region.assign_advice(
                            || format!("lookup_table[{}][{}]", idx, offset),
                            column,
                            offset,
                            || value,
                        )?;
                    }    
                }

                for (idx, (&column, values)) in config
                    .value_table
                    .iter()
                    .zip(self.value_table.transpose_array().iter())
                    .enumerate()
                {
                    for (offset, &value) in values.transpose_array().iter().enumerate() {
                        region.assign_advice(
                            || format!("value_table[{}][{}]", idx, offset),
                            column,
                            offset,
                            || value,
                        )?;
                    }
                }

                for (idx, (&column, values)) in config
                    .arranged_lookup_table
                    .iter()
                    .zip(self.arranged_lookup_table.transpose_array().iter())
                    .enumerate()
                {
                    for (offset, &value) in values.transpose_array().iter().enumerate() {
                        region.assign_advice(
                            || format!("arranged_lookup_table[{}][{}]", idx, offset),
                            column,
                            offset,
                            || value,
                        )?;
                    }    
                }

                for (idx, (&column, values)) in config
                    .arranged_value_table
                    .iter()
                    .zip(self.arranged_value_table.transpose_array().iter())
                    .enumerate()
                {
                    for (offset, &value) in values.transpose_array().iter().enumerate() {
                        region.assign_advice(
                            || format!("arranged_value_table[{}][{}]", idx, offset),
                            column,
                            offset,
                            || value,
                        )?;
                    }
                }

                // Second Phase

                // compute z
                let mut z = self.lookup_table.zip(self.value_table).zip(self.arranged_lookup_table).zip(self.arranged_value_table).zip(theta).zip(beta).zip(gamma).map(
                    |((((((lookup_table, value_table), arranged_lookup_table), arranged_value_table), theta), beta), gamma)| {
                        let mut product_vec = vec![Field::zero(); HEIGHT];
                        for (index, product) in product_vec.iter_mut().enumerate() {
                            let (mut compressed_t, mut compressed_v) = (Field::zero(), Field::zero());
                            for j in 0..WIDTH {
                                compressed_t *= theta;
                                compressed_t += arranged_lookup_table[j][index];
                                compressed_v *= theta;
                                compressed_v += arranged_value_table[j][index];
                            }
                            *product = (compressed_t + gamma) * (compressed_v + beta);
                        }
                        product_vec.iter_mut().batch_invert();
                        for (index, product) in product_vec.iter_mut().enumerate() {
                            let (mut compressed_t, mut compressed_v) = (Field::zero(), Field::zero());
                            for j in 0..WIDTH {
                                compressed_t *= theta;
                                compressed_t += lookup_table[j][index];
                                compressed_v *= theta;
                                compressed_v += value_table[j][index];
                            }
                            *product *= (compressed_t + gamma) * (compressed_v + beta);
                        }
                        let z = iter::once(Field::one())
                            .chain(product_vec)
                            .scan(Field::one(), |state, cur| {
                                *state *= &cur;
                                Some(*state)
                            })
                            .collect::<Vec<_>>();
                        
                        z
                    }
                );
                for (offset, value) in z.transpose_vec(HEIGHT + 1).into_iter().enumerate() {
                    region.assign_advice(
                        || format!("z[{}]", offset),
                        config.z_lookup,
                        offset,
                        || value,
                    )?;
                }
                Ok(())
            }
        )
    }
}

fn test_mock_prover<Field: FieldExt, const WIDTH: usize, const HEIGHT: usize>(
    k: u32,
    circuit: MyCircuit<Field, WIDTH, HEIGHT>,
    expected: Result<(), Vec<(metadata::Constraint, FailureLocation)>>,
) {
    let prover = MockProver::run::<_>(k, &circuit, vec![]).unwrap();
    match (prover.verify(), expected) {
        (Ok(_), Ok(_)) => {}
        (Err(err), Err(expected)) => {
            assert_eq!(
                err.into_iter()
                    .map(|failure| match failure {
                        VerifyFailure::ConstraintNotSatisfied {
                            constraint,
                            location,
                            ..
                        } => (constraint, location),
                        _ => panic!("MockProver::verify has result unmatching expected"),
                    })
                    .collect::<Vec<_>>(),
                expected
            )
        }
        (_, _) => panic!("MockProver::verify has result unmatching expected"),
    };
}

fn test_prover<C: CurveAffine, const WIDTH: usize, const HEIGHT: usize>(
    k: u32,
    circuit: MyCircuit<C::Scalar, WIDTH, HEIGHT>,
    expected: bool,
) {
    let params = ParamsIPA::<C>::new(k);
    let vk = keygen_vk(&params, &circuit).unwrap();
    let pk = keygen_pk(&params, vk, &circuit).unwrap();

    let proof = {
        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);

        create_proof::<IPACommitmentScheme<C>, ProverIPA<C>, _, _, _, _>(
            &params,
            &pk,
            &[circuit],
            &[&[]],
            OsRng,
            &mut transcript,
        )
        .expect("proof generation should not fail");

        transcript.finalize()
    };

    let accepted = {
        let strategy = AccumulatorStrategy::new(&params);
        let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);

        verify_proof::<IPACommitmentScheme<C>, VerifierIPA<C>, _, _, _>(
            &params,
            pk.get_vk(),
            strategy,
            &[&[]],
            &mut transcript,
        )
        .map(|strategy| strategy.finalize())
        .unwrap_or_default()
    };

    assert_eq!(accepted, expected);
}

fn main() {
    const WIDTH: usize = 4;
    const HEIGHT: usize = 5;
    const K: u32 = 8;

    let circuit = &MyCircuit::<Fp, WIDTH, HEIGHT>::rand(&mut OsRng, 3, 3);

    println!("Lookup table: ");
    print_array_table::<Fp, WIDTH, HEIGHT>(&circuit.lookup_table);
    println!("Arranged lookup table: ");
    print_array_table::<Fp, WIDTH, HEIGHT>(&circuit.arranged_lookup_table);
    println!("Arranged value table: ");
    print_array_table::<Fp, WIDTH, HEIGHT>(&circuit.arranged_value_table);
    println!("Value table: ");
    print_array_table::<Fp, WIDTH, HEIGHT>(&circuit.value_table);

    {
        test_mock_prover(K, circuit.clone(), Ok(()));
        test_prover::<EqAffine, WIDTH, HEIGHT>(K, circuit.clone(), true);
    }

    // #[cfg(not(feature = "sanity-checks"))]
    // {
    //     use std::ops::IndexMut;

    //     let mut circuit = circuit.clone();
    //     circuit.shuffled = circuit.shuffled.map(|mut shuffled| {
    //         shuffled.index_mut(0).swap(0, 1);
    //         shuffled
    //     });

    //     test_mock_prover(
    //         K,
    //         circuit.clone(),
    //         Err(vec![(
    //             ((1, "z should end with 1").into(), 0, "").into(),
    //             FailureLocation::InRegion {
    //                 region: (0, "Shuffle original into shuffled").into(),
    //                 offset: 32,
    //             },
    //         )]),
    //     );
    //     test_prover::<EqAffine, WIDTH, HEIGHT>(K, circuit, false);
    // }
}