use itertools::Itertools;

/// We require `FancyWire` to know its own modulus.
pub trait KnowsModulus {
    fn modulus(&self) -> u16;
}

/// Collection of `FancyWire`, which could be used for Chinese Remainder Theorem or Mixed
/// Radix number representations.
pub struct FancyBundle<W: KnowsModulus>(Vec<W>);

impl <W: KnowsModulus> FancyBundle<W> {
    pub fn moduli(&self) -> Vec<u16> {
        self.0.iter().map(|w| w.modulus()).collect()
    }

    pub fn wires(&self) -> &[W] {
        &self.0
    }
}

/// `FancyBuilder` implements the basic fancy-garbling functions, either to create a
/// circuit or a streaming protocol.
pub trait FancyBuilder {
    /// The underlying datatype created by a `FancyBuilder`.
    type FancyWire: Clone + KnowsModulus;

    fn garbler_input(&mut self, q: u16) -> Self::FancyWire;
    fn evaluator_input(&mut self, q: u16) -> Self::FancyWire;
    fn constant(&mut self, x: u16, q: u16) -> Self::FancyWire;

    fn add(&mut self, x: &Self::FancyWire, y: &Self::FancyWire) -> Self::FancyWire;
    fn sub(&mut self, x: &Self::FancyWire, y: &Self::FancyWire) -> Self::FancyWire;
    fn mul(&mut self, x: &Self::FancyWire, y: &Self::FancyWire) -> Self::FancyWire;
    fn cmul(&mut self, x: &Self::FancyWire, c: u16) -> Self::FancyWire;
    fn proj(&mut self, x: &Self::FancyWire, q: u16, tt: Vec<u16>) -> Self::FancyWire;

    ////////////////////////////////////////////////////////////////////////////////
    // bonus functions built on top of basic fancy operations

    /// Create `n` garbler inputs with modulus `q`.
    fn garbler_inputs(&mut self, n: usize, q: u16) -> Vec<Self::FancyWire> {
        (0..n).map(|_| self.garbler_input(q)).collect()
    }

    /// Create `n` evaluator inputs with modulus `q`.
    fn evaluator_inputs(&mut self, n: usize, q: u16) -> Vec<Self::FancyWire> {
        (0..n).map(|_| self.evaluator_input(q)).collect()
    }

    /// Sum up a slice of `Self::FancyWire`.
    fn add_many(&mut self, args: &[Self::FancyWire]) -> Self::FancyWire {
        assert!(args.len() > 1);
        let mut z = args[0].clone();
        for x in args.iter().skip(1) {
            z = self.add(&z,&x);
        }
        z
    }

    /// Xor is just addition, with the requirement that `x` and `y` are mod 2.
    fn xor(&mut self, x: &Self::FancyWire, y: &Self::FancyWire) -> Self::FancyWire {
        assert!(x.modulus() == 2 && y.modulus() == 2);
        self.add(x,y)
    }

    /// Negate by xoring `x` with `1`.
    fn negate(&mut self, x: &Self::FancyWire) -> Self::FancyWire {
        assert_eq!(x.modulus(), 2);
        let one = self.constant(1,2);
        self.xor(x, &one)
    }

    /// And is just multiplication, with the requirement that `x` and `y` are mod 2.
    fn and(&mut self, x: &Self::FancyWire, y: &Self::FancyWire) -> Self::FancyWire {
        assert!(x.modulus() == 2 && y.modulus() == 2);
        self.mul(x,y)
    }

    /// Returns 1 if all `Self::FancyWire` equal 1.
    fn and_many(&mut self, args: &[Self::FancyWire]) -> Self::FancyWire {
        args.iter().skip(1).fold(args[0].clone(), |acc, x| self.and(&acc, x))
    }

    // TODO: with free negation, use demorgans and AND
    /// Returns 1 if any `Self::FancyWire` equals 1 in `args`.
    fn or_many(&mut self, args: &[Self::FancyWire]) -> Self::FancyWire {
        assert!(args.iter().all(|x| x.modulus() == 2));
        // convert all the wires to base b+1
        let b = args.len();
        let wires = args.iter().map(|x| {
            self.proj(x, b as u16 + 1, vec![0,1])
        }).collect_vec();

        // add them together
        let z = self.add_many(&wires);

        // decode the result in base 2
        let mut tab = vec![1;b+1];
        tab[0] = 0;
        self.proj(&z,2,tab)
    }

    /// Change the modulus of `x` to `to_modulus` using a projection gate.
    fn mod_change(&mut self, x: &Self::FancyWire, to_modulus: u16) -> Self::FancyWire {
        let from_modulus = x.modulus();
        if from_modulus == to_modulus {
            return x.clone();
        }
        let tab = (0..from_modulus).map(|x| x % to_modulus).collect();
        self.proj(x, to_modulus, tab)
    }

    /// Mixed radix addition of potentially many values.
    fn mixed_radix_addition(&mut self, xs: &[Vec<Self::FancyWire>]) -> Vec<Self::FancyWire> {
        let nargs = xs.len();
        let n = xs[0].len();
        assert!(xs.len() > 1 && xs.iter().all(|x| x.len() == n));

        let mut digit_carry = None;
        let mut carry_carry = None;
        let mut max_carry = 0;

        let mut res = Vec::with_capacity(n);

        for i in 0..n {
            // all the ith digits, in one vec
            let ds = xs.iter().map(|x| x[i].clone()).collect_vec();

            // compute the digit -- easy
            let digit_sum = self.add_many(&ds);
            let digit = digit_carry.map_or(digit_sum.clone(), |d| self.add(&digit_sum, &d));

            if i < n-1 {
                // compute the carries
                let q = xs[0][i].modulus();
                // max_carry currently contains the max carry from the previous iteration
                let max_val = nargs as u16 * (q-1) + max_carry;
                // now it is the max carry of this iteration
                max_carry = max_val / q;

                let modded_ds = ds.iter().map(|d| self.mod_change(d, max_val+1)).collect_vec();

                let carry_sum = self.add_many(&modded_ds);
                // add in the carry from the previous iteration
                let carry = carry_carry.map_or(carry_sum.clone(), |c| self.add(&carry_sum, &c));

                // carry now contains the carry information, we just have to project it to
                // the correct moduli for the next iteration
                let next_mod = xs[0][i+1].modulus();
                let tt = (0..=max_val).map(|i| (i / q) % next_mod).collect_vec();
                digit_carry = Some(self.proj(&carry, next_mod, tt));

                let next_max_val = nargs as u16 * (next_mod - 1) + max_carry;

                if i < n-2 {
                    if max_carry < next_mod {
                        carry_carry = Some(self.mod_change(digit_carry.as_ref().unwrap(), next_max_val + 1));
                    } else {
                        let tt = (0..=max_val).map(|i| i / q).collect_vec();
                        carry_carry = Some(self.proj(&carry, next_max_val + 1, tt));
                    }
                } else {
                    // next digit is MSB so we dont need carry_carry
                    carry_carry = None;
                }
            } else {
                digit_carry = None;
                carry_carry = None;
            }
            res.push(digit);
        }
        res
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Things dealing with bundles

    /// Crate an input bundle for the garbler using composite modulus `q`.
    fn garbler_input_bundle(&mut self, q: u128) -> FancyBundle<Self::FancyWire> {
        let ps = crate::util::factor(q);
        let ws = ps.into_iter().map(|p| self.garbler_input(p)).collect();
        FancyBundle(ws)
    }

    /// Crate an input bundle for the evaluator using composite modulus `q`.
    fn evaluator_input_bundle(&mut self, q: u128) -> FancyBundle<Self::FancyWire> {
        let ps = crate::util::factor(q);
        let ws = ps.into_iter().map(|p| self.evaluator_input(p)).collect();
        FancyBundle(ws)
    }

    /// Creates a bundle of constant wires for the CRT representation of `x` under
    /// composite modulus `q`.
    fn constant_bundle(&mut self, x: u128, q: u128) -> FancyBundle<Self::FancyWire> {
        let ps = crate::util::factor(q);
        let ws = ps.into_iter().map(|p| {
            let c = (x % p as u128) as u16;
            self.constant(c,p)
        }).collect();
        FancyBundle(ws)
    }

    /// Create `n` garbler input wires, under composite modulus `q`.
    fn garbler_input_bundles(&mut self, q: u128, n: usize) -> Vec<FancyBundle<Self::FancyWire>> {
        (0..n).map(|_| self.garbler_input_bundle(q)).collect()
    }

    /// Create `n` evaluator input wires, under composite modulus `q`.
    fn evaluator_input_bundles(&mut self, q: u128, n: usize) -> Vec<FancyBundle<Self::FancyWire>> {
        (0..n).map(|_| self.evaluator_input_bundle(q)).collect()
    }

    /// Add two wire bundles, residue by residue.
    fn add_bundles(&mut self, x: &FancyBundle<Self::FancyWire>, y: &FancyBundle<Self::FancyWire>)
        -> FancyBundle<Self::FancyWire> {
        assert_eq!(x.0.len(), y.0.len());
        let res = x.0.iter().zip(y.0.iter()).map(|(x,y)| self.add(x,y)).collect();
        FancyBundle(res)
    }

    /// Subtract two wire bundles, residue by residue.
    fn sub_bundles(&mut self, x: &FancyBundle<Self::FancyWire>, y: &FancyBundle<Self::FancyWire>)
        -> FancyBundle<Self::FancyWire> {
        assert_eq!(x.0.len(), y.0.len());
        let res = x.0.iter().zip(y.0.iter()).map(|(x,y)| self.sub(x,y)).collect();
        FancyBundle(res)
    }

    /// Multiplies each wire in `x` by the corresponding residue of `c`.
    fn cmul_bundle(&mut self, x: &FancyBundle<Self::FancyWire>, c: u128) -> FancyBundle<Self::FancyWire> {
        let primes = x.moduli();
        let cs = crate::util::crt(&primes, c);
        let ws = x.0.iter().zip(cs.into_iter()).map(|(x,c)| self.cmul(x,c)).collect();
        FancyBundle(ws)
    }
}


