use halo2_proofs::circuit::Value;
use halo2_proofs::plonk::Selector;
use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Chip, Layouter, SimpleFloorPlanner},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Fixed, Instance},
    poly::Rotation,
};
use std::env;
use std::marker::PhantomData;

// We would like to prove that there exists (u,v) such that
// y=u^3+u^2*v+u*v^2+v^3+1
// where y is a known public value

// Step 1: Define the configuration table. A configuration table has
// 3 types of column: advice, instance, and fixed. Advices are
#[derive(Clone, Debug)]
struct ExampleConfig {
    // The advice column, containing the witness
    advice: [Column<Advice>; 3],
    // The instance column, containing the public values
    instance: Column<Instance>,
    // The fixed column, containing the fixed values, used for lookup
    fixed: Column<Fixed>,

    // The selectors. We need 3 selector for addition, multiplication
    // and addition with constant
    selector_add: Selector,
    selector_mul: Selector,
    selector_add_const: Selector,
}

// Step 2: Define a chip struct to constraint the circuit and provide
// assignment functions
struct ExampleChip<Field: FieldExt> {
    // the chip must contain the configuration table
    config: ExampleConfig,
    _marker: PhantomData<Field>,
}

// Implement the chip struct
// the chip struct must have two functions: config() and loaded()
// these functions are not that necessary in our example
impl<Field: FieldExt> Chip<Field> for ExampleChip<Field> {
    type Config = ExampleConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<Field: FieldExt> ExampleChip<Field> {
    // describe the arrangement of the circuit
    // normally we just need to define the gate in here
    fn configure(
        // meta is the constraint system struct used for creating gates
        // and enabling equalities over cells
        meta: &mut ConstraintSystem<Field>,
        advice: [Column<Advice>; 3],
        instance: Column<Instance>,
        fixed: Column<Fixed>,
    ) -> <Self as Chip<Field>>::Config {
        // specify columns used for proving copy constraints
        // enable_equality() allows the specified column to participate
        // in the permutation check
        meta.enable_equality(instance);
        // enable_constant() allows the fixed column to be used
        meta.enable_constant(fixed);
        for column in &advice {
            meta.enable_equality(*column);
        }

        // allocate columns with respect to selectors
        let selector_add = meta.selector();
        let selector_mul = meta.selector();
        let selector_add_const = meta.selector();

        // we start with 3 simple gates: the addition, multiplication and
        // addition with constant.  we can also define other custom gates
        // as well, but now we will just start with the basic

        // define addition gate
        // requires selector_add*(witness[0]+witness[1]-witness[2])=0
        meta.create_gate("add", |meta| {
            let selector_add = meta.query_selector(selector_add);
            // set lhs to be witness[0]
            let lhs = meta.query_advice(advice[0], Rotation::cur());
            // set rhs to be witness[1]
            let rhs = meta.query_advice(advice[1], Rotation::cur());
            // set out to be witness[2]
            let out = meta.query_advice(advice[2], Rotation::cur());
            vec![selector_add * (lhs + rhs - out)]
        });

        // define multiplication gate
        // requires selector_mul*(witness[0]*witness[1]-witness[2])=0
        meta.create_gate("mul", |meta| {
            let selector_mul = meta.query_selector(selector_mul);
            let lhs = meta.query_advice(advice[0], Rotation::cur());
            let rhs = meta.query_advice(advice[1], Rotation::cur());
            let out = meta.query_advice(advice[2], Rotation::cur());
            vec![selector_mul * (lhs * rhs - out)]
        });

        // define addition with constant gate
        // requires selector_add_const*(witness[0]+fixed-witness[2])=0
        meta.create_gate("add with constant", |meta| {
            let selector_add_const = meta.query_selector(selector_add_const);
            let lhs = meta.query_advice(advice[0], Rotation::cur());
            let fixed = meta.query_fixed(fixed, Rotation::cur());
            let out = meta.query_advice(advice[2], Rotation::cur());
            vec![selector_add_const * (lhs + fixed - out)]
        });

        ExampleConfig {
            advice,
            instance,
            fixed,
            selector_add,
            selector_mul,
            selector_add_const,
        }
    }
}

// 3. Define a circuit struct that implement the circuit trait
// the Circuit trait has THREE functions:
// without_witnesses(), configure() and synthesize()
// a) the function  without_witnesses() returns a copy of the circuit
// without the witness value
// b) the function configure() describes the gate arrangement and column
// arrangement
// c) the function synthesize() add the constraints between the instance,
// advice and fixed cells of the circuit
#[derive(Default)]
struct ExampleCircuit<Field: FieldExt> {
    u: Value<Field>,
    v: Value<Field>,
}

impl<Field: FieldExt> Circuit<Field> for ExampleCircuit<Field> {
    type Config = ExampleConfig;
    type FloorPlanner = SimpleFloorPlanner;

    // right now, we don't need this function in our example
    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Field>) -> Self::Config {
        let advice = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];
        let instance = meta.instance_column();
        let constant = meta.fixed_column();
        ExampleChip::configure(meta, advice, instance, constant)
    }

    // we shall define the constraints of our example here
    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Field>,
    ) -> Result<(), Error> {
        // y=u^3+u^2*v+u*v^2+v^3+1
        // meaning the constraints are:
        // y=t10+1
        // and t1= u^2, t2=t1*u, t3=v^2, t4=v*t3,  t5=u*v, t6=u*t5, t7=v*t5
        // t8=t2+t4, t9=t6+t7, t10=t8+t9

        // handling multiplication region
        // temp variables for multiplication constraints
        // t1= u^2, t2=t1*u, t3=v^2, t4=v*t3,  t5=u*v, t6=t5*u, t7=t5*v

        let t1 = self.u * self.u;
        let t2 = t1 * self.u;
        let t3 = self.v * self.v;
        let t4 = t3 * self.v;
        let t5 = self.u * self.v;
        let t6 = self.u * t5;
        let t7 = self.v * t5;

        // define the contraints of multiplication
        // we need t1=u*u, t2=u*v,
        let (
            (x_a1, x_b1, x_c1),
            (x_a2, x_b2, x_c2),
            (x_a3, x_b3, x_c3),
            (x_a4, x_b4, x_c4),
            (x_a5, x_b5, x_c5),
            (x_a6, x_b6, x_c6),
            (x_a7, x_b7, x_c7),
        ) = layouter.assign_region(
            || "multiplication region",
            |mut region| {
                // first row
                // require t1=u*u
                // the function enable() sets the selector of the 'offset'-th row to be 1
                // the selector selector_mul represents the condition selector_mul*(x_a1*x_b1-x_c1)=0
                // since we set the selector_mul selector to be 1, it means that the constraint
                // x_a1*x_b1-x_c1=0 is enabled
                config.selector_mul.enable(&mut region, 0)?;
                let x_a1 =
                // assign_advice() assigns the cell named x_a1 to be u
                // the parameters are: 'annotation', 'column', 'offset' and 'to'
                // from what I know, 'column' is the column to be assigned and
                // 'offset' denotes the position of column and
                // 'to' denotes the value that to be assigned.
                    region.assign_advice(|| "x_a1", config.advice[0].clone(), 0, || self.u)?;
                let x_b1 =
                    region.assign_advice(|| "x_b1", config.advice[1].clone(), 0, || self.u)?;
                let x_c1 = region.assign_advice(|| "x_c1", config.advice[2].clone(), 0, || t1)?;

                // second row
                // require t2=t1*u
                config.selector_mul.enable(&mut region, 1)?;
                let x_a2 = region.assign_advice(|| "x_a2", config.advice[0].clone(), 1, || t1)?;
                let x_b2 =
                    region.assign_advice(|| "x_b2", config.advice[1].clone(), 1, || self.u)?;
                let x_c2 = region.assign_advice(|| "x_c2", config.advice[2].clone(), 1, || t2)?;

                // third row
                // require t3=v*v
                config.selector_mul.enable(&mut region, 2)?;
                let x_a3 =
                    region.assign_advice(|| "x_a3", config.advice[0].clone(), 2, || self.v)?;
                let x_b3 =
                    region.assign_advice(|| "x_b3", config.advice[1].clone(), 2, || self.v)?;
                let x_c3 = region.assign_advice(|| "x_c3", config.advice[2].clone(), 2, || t3)?;

                // fourth row
                // require t4=t3*v
                config.selector_mul.enable(&mut region, 3)?;
                let x_a4 = region.assign_advice(|| "x_a4", config.advice[0].clone(), 3, || t3)?;
                let x_b4 =
                    region.assign_advice(|| "x_b4", config.advice[1].clone(), 3, || self.v)?;
                let x_c4 = region.assign_advice(|| "x_c4", config.advice[2].clone(), 3, || t4)?;

                // fifth row
                // require t5=u*v
                config.selector_mul.enable(&mut region, 4)?;
                let x_a5 =
                    region.assign_advice(|| "x_a5", config.advice[0].clone(), 4, || self.u)?;
                let x_b5 =
                    region.assign_advice(|| "x_b5", config.advice[1].clone(), 4, || self.v)?;
                let x_c5 = region.assign_advice(|| "x_c5", config.advice[2].clone(), 4, || t5)?;

                // sixth row
                // require t6=t5*u
                config.selector_mul.enable(&mut region, 5)?;
                let x_a6 = region.assign_advice(|| "x_a6", config.advice[0].clone(), 5, || t5)?;
                let x_b6 =
                    region.assign_advice(|| "x_b6", config.advice[1].clone(), 5, || self.u)?;
                let x_c6 = region.assign_advice(|| "x_c6", config.advice[2].clone(), 5, || t6)?;

                // seventh row
                // require t7=t5*v
                config.selector_mul.enable(&mut region, 5)?;
                let x_a7 = region.assign_advice(|| "x_a7", config.advice[0].clone(), 6, || t5)?;
                let x_b7 =
                    region.assign_advice(|| "x_b7", config.advice[1].clone(), 6, || self.v)?;
                let x_c7 = region.assign_advice(|| "x_c7", config.advice[2].clone(), 6, || t7)?;

                Ok((
                    (x_a1.cell(), x_b1.cell(), x_c1.cell()),
                    (x_a2.cell(), x_b2.cell(), x_c2.cell()),
                    (x_a3.cell(), x_b3.cell(), x_c3.cell()),
                    (x_a4.cell(), x_b4.cell(), x_c4.cell()),
                    (x_a5.cell(), x_b5.cell(), x_c5.cell()),
                    (x_a6.cell(), x_b6.cell(), x_c6.cell()),
                    (x_a7.cell(), x_b7.cell(), x_c7.cell()),
                ))
            },
        )?;

        // temp variables for addition constraints
        // t8=t2+t4, t9=t6+t7, t10=t8+t9, t11=t10+1

        let t8 = t2 + t4;
        let t9 = t6 + t7;
        let t10 = t8 + t9;
        let t11 = t10 + Value::known(Field::from(1));

        let ((x_a8, x_b8, x_c8), (x_a9, x_b9, x_c9), (x_a10, x_b10, x_c10), (x_a11, x_c11)) =
            layouter.assign_region(
                || "addition region",
                |mut region| {
                    // first row
                    // require t8=t2+t4
                    // now we turn on the addition selector to handle the addition region
                    config.selector_add.enable(&mut region, 0)?;
                    let x_a8 =
                        region.assign_advice(|| "x_a8", config.advice[0].clone(), 0, || t2)?;
                    let x_b8 =
                        region.assign_advice(|| "x_b8", config.advice[1].clone(), 0, || t4)?;
                    let x_c8 =
                        region.assign_advice(|| "x_c8", config.advice[2].clone(), 0, || t8)?;

                    // second row
                    // require t9=t6+t7
                    config.selector_add.enable(&mut region, 1)?;
                    let x_a9 =
                        region.assign_advice(|| "x_a9", config.advice[0].clone(), 1, || t6)?;
                    let x_b9 =
                        region.assign_advice(|| "x_b9", config.advice[1].clone(), 1, || t7)?;
                    let x_c9 =
                        region.assign_advice(|| "x_c9", config.advice[2].clone(), 1, || t9)?;

                    // third row
                    // require t10=t8+t9
                    config.selector_add.enable(&mut region, 2)?;
                    let x_a10 =
                        region.assign_advice(|| "x_a10", config.advice[0].clone(), 2, || t8)?;
                    let x_b10 =
                        region.assign_advice(|| "x_b10", config.advice[1].clone(), 2, || t9)?;
                    let x_c10 =
                        region.assign_advice(|| "x_c10", config.advice[2].clone(), 2, || t10)?;

                    // third row
                    // require t11=t10+1
                    config.selector_add_const.enable(&mut region, 3)?;
                    let x_a11 =
                        region.assign_advice(|| "x_a11", config.advice[0].clone(), 3, || t10)?;

                    // assign the fixed value to be 1
                    region.assign_fixed(
                        || "constant 1",
                        config.fixed.clone(),
                        3,
                        || Value::known(Field::from(1)),
                    )?;

                    let x_c11 =
                        region.assign_advice(|| "x_c11", config.advice[2].clone(), 3, || t11)?;
                    Ok((
                        (x_a8.cell(), x_b8.cell(), x_c8.cell()),
                        (x_a9.cell(), x_b9.cell(), x_c9.cell()),
                        (x_a10.cell(), x_b10.cell(), x_c10.cell()),
                        (x_a11.cell(), x_c11.cell()),
                    ))
                },
            )?;

        // note that t11 is the instance result, so we constraint it to
        // be equal to the instance
        layouter.constrain_instance(x_c11, config.instance, 0)?;

        // finally, we enforce the copy constraints between the cells
        // there are actually a lot of copy contraints here
        layouter.assign_region(
            || "equality",
            |mut region| {
                region.constrain_equal(x_a1, x_b1)?; // namely, x_a1 = x_b1

                region.constrain_equal(x_c1, x_a2)?; // namely, x_c1 = x_a2

                region.constrain_equal(x_a1, x_b2)?; // namely, x_a1 = x_b2

                region.constrain_equal(x_a3, x_b3)?; // namely, x_a3 = x_b3

                region.constrain_equal(x_c3, x_a4)?; // namely, x_c3 = x_a4

                region.constrain_equal(x_b3, x_b4)?; // namely, x_b3 = x_b4

                region.constrain_equal(x_a1, x_a5)?; // namely, x_a1 = x_a5

                region.constrain_equal(x_b3, x_b5)?; // namely, x_b3 = x_b5

                region.constrain_equal(x_c5, x_a6)?; // namely, x_c5 = x_a6

                region.constrain_equal(x_a1, x_b6)?; // namely, x_a1 = x_b6

                region.constrain_equal(x_c5, x_a7)?; // namely, x_c5 = x_a7

                region.constrain_equal(x_b3, x_b7)?; // namely, x_b3 = x_b7

                region.constrain_equal(x_c2, x_a8)?; // namely, x_c2 = x_a8

                region.constrain_equal(x_c4, x_b8)?; // namely, x_c4 = x_b8

                region.constrain_equal(x_c6, x_a9)?; // namely, x_c6 = x_a9

                region.constrain_equal(x_c7, x_b9)?; // namely, x_c6 = x_b9

                region.constrain_equal(x_c8, x_a10)?; // namely, x_c8 = x_a10

                region.constrain_equal(x_c9, x_b10)?; // namely, x_c9 = x_b10

                region.constrain_equal(x_c10, x_a11)?; // namely, x_c10 = x_a11

                Ok(())
            },
        )?;
        Ok(())
    }
}

// 4.Test our circuit
fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::halo2curves::bn256::Fr as Fp;

    // create witness
    let u = Fp::from(12);
    let v = Fp::from(9);

    // create instance
    let res = u * u * u + v * v * v + u * u * v + u * v * v + Fp::from(1);

    // instantiate the circuit
    let circuit = ExampleCircuit {
        u: Value::known(u),
        v: Value::known(v),
    };

    // the number of rows cannot exceed 2^k
    let k = 5;
    // prove and verify
    let prover = MockProver::run(k, &circuit, vec![vec![res]]).unwrap();
    assert_eq!(prover.verify(), Ok(()));
}
