use crate::error::FancyError;
use crate::fancy::bundle::{Bundle, BundleGadgets};
use crate::fancy::{Fancy, HasModulus};
use crate::util;
use itertools::Itertools;
use std::ops::Deref;

/// Bundle which is explicitly binary representation.
#[derive(Clone)]
pub struct BinaryBundle<W: Clone + HasModulus>(Bundle<W>);

impl<W: Clone + HasModulus> BinaryBundle<W> {
    /// Create a new binary bundle from a vector of wires.
    pub fn new(ws: Vec<W>) -> BinaryBundle<W> {
        BinaryBundle(Bundle::new(ws))
    }

<<<<<<< HEAD
    /// Mark a regular bundle as Binary.
    pub fn from_bundle(b: Bundle<W>) -> BinaryBundle<W> {
        BinaryBundle(b)
    }

    /// Extract the underlying bundle from this binary bundle.
    pub fn extract(self) -> Bundle<W> {
        self.0
||||||| merged common ancestors
    /// Unwrap the underlying bundle from this binary bundle.
    pub fn unwrap<'a>(&'a self) -> &'a Bundle<W> {
        &self.0
=======
    /// Mark a regular bundle as Binary.
    pub fn from_bundle(b: Bundle<W>) -> BinaryBundle<W> {
        BinaryBundle(b)
    }

    /// Unwrap the underlying bundle from this binary bundle.
    pub fn borrow<'a>(&'a self) -> &'a Bundle<W> {
        &self.0
>>>>>>> 8b3bce563e51c2202836be81d3874b034f172324
    }

    /// Extract the underlying bundle from this binary bundle.
    pub fn extract(self) -> Bundle<W> {
        self.0
    }
}

impl<W: Clone + HasModulus> Deref for BinaryBundle<W> {
    type Target = Bundle<W>;

    fn deref(&self) -> &Bundle<W> {
        &self.0
    }
}

impl<W: Clone + HasModulus> From<Bundle<W>> for BinaryBundle<W> {
    fn from(b: Bundle<W>) -> BinaryBundle<W> {
        debug_assert!(b.moduli().iter().all(|&p| p == 2));
        BinaryBundle(b)
    }
}

impl<F: Fancy> BinaryGadgets for F {}

/// Extension trait for `Fancy` providing gadgets that operate over bundles of mod2 wires.
pub trait BinaryGadgets: Fancy + BundleGadgets {
    /// Create an input bundle for the garbler using `nbits` base 2 inputs and optional input `x`.
    fn bin_garbler_input_bundle(
        &mut self,
        nbits: usize,
        opt_x: Option<u128>,
    ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
        self.garbler_input_bundle(&vec![2; nbits], opt_x.map(|x| util::u128_to_bits(x, nbits)))
            .map(BinaryBundle)
    }

    /// Create an input bundle for the evaluator using n base 2 inputs.
    fn bin_evaluator_input_bundle(
        &mut self,
        n: usize,
    ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
        self.evaluator_input_bundle(&vec![2; n]).map(BinaryBundle)
    }

    /// Create `n` garbler input bundles, each with `nbits` bits and optional
    /// inputs `xs`.
    fn bin_garbler_input_bundles(
        &mut self,
        nbits: usize,
        n: usize,
        opt_xs: Option<Vec<u128>>,
    ) -> Result<Vec<BinaryBundle<Self::Item>>, Self::Error> {
        if let Some(xs) = opt_xs {
            if xs.len() != n {
                return Err(Self::Error::from(FancyError::InvalidArgNum {
                    got: xs.len(),
                    needed: n,
                }));
            }
            xs.into_iter()
                .map(|x| self.bin_garbler_input_bundle(nbits, Some(x)))
                .collect()
        } else {
            (0..n)
                .map(|_| self.bin_garbler_input_bundle(nbits, None))
                .collect()
        }
    }

    /// Create `n` evaluator input bundles, each with `nbits` bits.
    fn bin_evaluator_input_bundles(
        &mut self,
        nbits: usize,
        n: usize,
    ) -> Result<Vec<BinaryBundle<Self::Item>>, Self::Error> {
        (0..n)
            .map(|_| self.bin_evaluator_input_bundle(nbits))
            .collect()
    }

    /// Create a constant bundle using base 2 inputs.
    fn bin_constant_bundle(
        &mut self,
        val: u128,
        nbits: usize,
    ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
        self.constant_bundle(&util::u128_to_bits(val, nbits), &vec![2; nbits])
            .map(BinaryBundle)
    }

    /// Xor the bits of two bundles together pairwise.
    fn bin_xor(
        &mut self,
        x: &BinaryBundle<Self::Item>,
        y: &BinaryBundle<Self::Item>,
    ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
<<<<<<< HEAD
        self.add_bundles(&x, &y).map(BinaryBundle)
||||||| merged common ancestors
        self.add_bundles(x.unwrap(), y.unwrap()).map(BinaryBundle)
=======
        self.add_bundles(x.borrow(), y.borrow()).map(BinaryBundle)
>>>>>>> 8b3bce563e51c2202836be81d3874b034f172324
    }

    /// And the bits of two bundles together pairwise.
    fn bin_and(
        &mut self,
        x: &BinaryBundle<Self::Item>,
        y: &BinaryBundle<Self::Item>,
    ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
<<<<<<< HEAD
        self.mul_bundles(&x, &y).map(BinaryBundle)
||||||| merged common ancestors
        self.mul_bundles(x.unwrap(), y.unwrap()).map(BinaryBundle)
=======
        self.mul_bundles(x.borrow(), y.borrow()).map(BinaryBundle)
>>>>>>> 8b3bce563e51c2202836be81d3874b034f172324
    }

    /// Binary addition. Returns the result and the carry.
    fn bin_addition(
        &mut self,
        xs: &BinaryBundle<Self::Item>,
        ys: &BinaryBundle<Self::Item>,
    ) -> Result<(BinaryBundle<Self::Item>, Self::Item), Self::Error> {
        if xs.moduli() != ys.moduli() {
            return Err(Self::Error::from(FancyError::UnequalModuli));
        }
        let xwires = xs.wires();
        let ywires = ys.wires();
        let (mut z, mut c) = self.adder(&xwires[0], &ywires[0], None)?;
        let mut bs = vec![z];
        for i in 1..xwires.len() {
            let res = self.adder(&xwires[i], &ywires[i], Some(&c))?;
            z = res.0;
            c = res.1;
            bs.push(z);
        }
        Ok((BinaryBundle::new(bs), c))
    }

    /// Binary addition. Avoids creating extra gates for the final carry.
    fn bin_addition_no_carry(
        &mut self,
        xs: &BinaryBundle<Self::Item>,
        ys: &BinaryBundle<Self::Item>,
    ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
        if xs.moduli() != ys.moduli() {
            return Err(Self::Error::from(FancyError::UnequalModuli));
        }
        let xwires = xs.wires();
        let ywires = ys.wires();
        let (mut z, mut c) = self.adder(&xwires[0], &ywires[0], None)?;
        let mut bs = vec![z];
        for i in 1..xwires.len() - 1 {
            let res = self.adder(&xwires[i], &ywires[i], Some(&c))?;
            z = res.0;
            c = res.1;
            bs.push(z);
        }
        z = self.add_many(&[
            xwires.last().unwrap().clone(),
            ywires.last().unwrap().clone(),
            c,
        ])?;
        bs.push(z);
        Ok(BinaryBundle::new(bs))
    }

    /// Binary multiplication.
    ///
    /// Returns the lower-order half of the output bits, ie a number with the same number
    /// of bits as the inputs.
    fn bin_multiplication_lower_half(
        &mut self,
        xs: &BinaryBundle<Self::Item>,
        ys: &BinaryBundle<Self::Item>,
    ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
        if xs.moduli() != ys.moduli() {
            return Err(Self::Error::from(FancyError::UnequalModuli));
        }

        let xwires = xs.wires();
        let ywires = ys.wires();

        let mut sum = xwires
            .iter()
            .map(|x| self.and(x, &ywires[0]))
            .collect::<Result<Vec<Self::Item>, Self::Error>>()
            .map(BinaryBundle::new)?;

        for i in 1..xwires.len() {
            let mul = xwires
                .iter()
                .map(|x| self.and(x, &ywires[i]))
                .collect::<Result<Vec<Self::Item>, Self::Error>>()
                .map(BinaryBundle::new)?;
            let shifted = self.shift(&mul, i).map(BinaryBundle)?;
            sum = self.bin_addition_no_carry(&sum, &shifted)?;
        }

        Ok(sum)
    }

    /// Compute the twos complement of the input bundle (which must be base 2).
    fn bin_twos_complement(
        &mut self,
        xs: &BinaryBundle<Self::Item>,
    ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
        let not_xs = xs
            .wires()
            .iter()
            .map(|x| self.negate(x))
            .collect::<Result<Vec<Self::Item>, Self::Error>>()
            .map(BinaryBundle::new)?;
        let one = self.bin_constant_bundle(1, xs.size())?;
        self.bin_addition_no_carry(&not_xs, &one)
    }

    /// Subtract two binary bundles. Returns the result and whether it underflowed.
    ///
    /// Due to the way that `twos_complement(0) = 0`, underflow indicates `y != 0 && x >= y`.
    fn bin_subtraction(
        &mut self,
        xs: &BinaryBundle<Self::Item>,
        ys: &BinaryBundle<Self::Item>,
    ) -> Result<(BinaryBundle<Self::Item>, Self::Item), Self::Error> {
        let neg_ys = self.bin_twos_complement(&ys)?;
        self.bin_addition(&xs, &neg_ys)
    }

    /// If `x=0` return `c1` as a bundle of constant bits, else return `c2`.
    fn bin_multiplex_constant_bits(
        &mut self,
        x: &Self::Item,
        c1: u128,
        c2: u128,
        nbits: usize,
    ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
        let c1_bs = util::u128_to_bits(c1, nbits)
            .into_iter()
            .map(|x: u16| x > 0)
            .collect_vec();
        let c2_bs = util::u128_to_bits(c2, nbits)
            .into_iter()
            .map(|x: u16| x > 0)
            .collect_vec();
        c1_bs
            .into_iter()
            .zip(c2_bs.into_iter())
            .map(|(b1, b2)| self.mux_constant_bits(x, b1, b2))
            .collect::<Result<Vec<Self::Item>, Self::Error>>()
            .map(BinaryBundle::new)
    }

    /// Write the constant in binary and that gives you the shift amounts, Eg.. 7x is 4x+2x+x.
    fn bin_cmul(
        &mut self,
        x: &BinaryBundle<Self::Item>,
        c: u128,
        nbits: usize,
    ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
        let zero = self.bin_constant_bundle(0, nbits)?;
        util::u128_to_bits(c, nbits)
            .into_iter()
            .enumerate()
            .filter_map(|(i, b)| if b > 0 { Some(i) } else { None })
            .fold(Ok(zero), |z, shift_amt| {
                let s = self.shift(x, shift_amt).map(BinaryBundle)?;
                self.bin_addition_no_carry(&(z?), &s)
            })
    }

    /// Compute the absolute value of a binary bundle.
    fn bin_abs(
        &mut self,
        x: &BinaryBundle<Self::Item>,
    ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
        let sign = x.wires().last().unwrap();
        let negated = self.bin_twos_complement(x)?;
        self.multiplex(&sign, x, &negated).map(BinaryBundle)
    }

    /// Returns 1 if `x < y`.
    fn bin_lt(
        &mut self,
        x: &BinaryBundle<Self::Item>,
        y: &BinaryBundle<Self::Item>,
    ) -> Result<Self::Item, Self::Error> {
        // underflow indicates y != 0 && x >= y
        // requiring special care to remove the y != 0, which is what follows.
        let (_, lhs) = self.bin_subtraction(x, y)?;

        // Now we build a clause equal to (y == 0 || x >= y), which we can OR with
        // lhs to remove the y==0 aspect.
        // check if y==0
        let y_contains_1 = self.or_many(y.wires())?;
        let y_eq_0 = self.negate(&y_contains_1)?;

        // if x != 0, then x >= y, ... assuming x is not negative
        let x_contains_1 = self.or_many(x.wires())?;

        // y == 0 && x >= y
        let rhs = self.and(&y_eq_0, &x_contains_1)?;

        // (y != 0 && x >= y) || (y == 0 && x >= y)
        // => x >= y && (y != 0 || y == 0)\
        // => x >= y && 1
        // => x >= y
        let geq = self.or(&lhs, &rhs)?;
        self.negate(&geq)
    }

    /// Returns 1 if `x >= y`.
    fn bin_geq(
        &mut self,
        x: &BinaryBundle<Self::Item>,
        y: &BinaryBundle<Self::Item>,
    ) -> Result<Self::Item, Self::Error> {
        let z = self.bin_lt(x, y)?;
        self.negate(&z)
    }

    /// Compute the maximum bundle in `xs`.
    fn bin_max(
        &mut self,
        xs: &[BinaryBundle<Self::Item>],
    ) -> Result<BinaryBundle<Self::Item>, Self::Error> {
        if xs.len() < 2 {
            return Err(Self::Error::from(FancyError::InvalidArgNum {
                got: xs.len(),
                needed: 2,
            }));
        }
        xs.iter().skip(1).fold(Ok(xs[0].clone()), |x, y| {
            x.map(|x| {
                let pos = self.bin_lt(&x, y)?;
                let neg = self.negate(&pos)?;
                x.wires()
                    .iter()
                    .zip(y.wires().iter())
                    .map(|(x, y)| {
                        let xp = self.mul(x, &neg)?;
                        let yp = self.mul(y, &pos)?;
                        self.add(&xp, &yp)
                    })
                    .collect::<Result<Vec<Self::Item>, Self::Error>>()
                    .map(BinaryBundle::new)
            })?
        })
    }
}
