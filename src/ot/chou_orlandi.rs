// -*- mode: rust; -*-
//
// This file is part of ocelot.
// Copyright © 2019 Galois, Inc.
// See LICENSE for licensing information.

use crate::stream;
use crate::utils;
use crate::{Block, BlockObliviousTransfer};
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_TABLE;
use curve25519_dalek::scalar::Scalar;
use failure::Error;
use rand::rngs::ThreadRng;
use std::io::{Read, Write};
use std::marker::PhantomData;

/// Implementation of the Chou-Orlandi semi-honest secure oblivious transfer
/// protocol (cf. <https://eprint.iacr.org/2015/267>).
///
/// This implementation uses the Ristretto prime order elliptic curve group from
/// the `curve25519-dalek` library and works over blocks rather than arbitrary
/// length messages.
pub struct ChouOrlandiOT<S: Read + Write + Send> {
    _s: PhantomData<S>,
    rng: ThreadRng,
}

impl<S: Read + Write + Send> BlockObliviousTransfer<S> for ChouOrlandiOT<S> {
    fn new() -> Self {
        let rng = rand::thread_rng();
        Self {
            _s: PhantomData::<S>,
            rng,
        }
    }

    fn send(&mut self, stream: &mut S, inputs: &[(Block, Block)]) -> Result<(), Error> {
        let y = Scalar::random(&mut self.rng);
        let s = &y * &RISTRETTO_BASEPOINT_TABLE;
        stream::write_pt(stream, &s)?;
        for input in inputs.iter() {
            let r = stream::read_pt(stream)?;
            let k0 = utils::hash_pt_block(&(r * y));
            let k1 = utils::hash_pt_block(&((r - s) * y));
            let c0 = encrypt(&k0, &input.0);
            let c1 = encrypt(&k1, &input.1);
            stream::write_block(stream, &c0)?;
            stream::write_block(stream, &c1)?;
        }
        Ok(())
    }

    fn receive(&mut self, stream: &mut S, inputs: &[bool]) -> Result<Vec<Block>, Error> {
        let s = stream::read_pt(stream)?;
        inputs
            .iter()
            .map(|b| {
                let x = Scalar::random(&mut self.rng);
                let c = if *b { Scalar::one() } else { Scalar::zero() };
                let r = c * s + &x * &RISTRETTO_BASEPOINT_TABLE;
                stream::write_pt(stream, &r)?;
                let k = utils::hash_pt_block(&(x * s));
                let c0 = stream::read_block(stream)?;
                let c1 = stream::read_block(stream)?;
                let c = if *b { &c1 } else { &c0 };
                let c = decrypt(&k, &c);
                Ok(c)
            })
            .collect()
    }
}

#[inline(always)]
fn encrypt(k: &Block, m: &Block) -> Block {
    utils::xor_block(&k, &m)
}
#[inline(always)]
fn decrypt(k: &Block, c: &Block) -> Block {
    utils::xor_block(&k, &c)
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use std::os::unix::net::UnixStream;

    #[test]
    fn test() {
        let m0 = rand::random::<Block>();
        let m1 = rand::random::<Block>();
        let b = rand::random::<bool>();
        let m0_ = m0.clone();
        let m1_ = m1.clone();
        let (mut sender, mut receiver) = match UnixStream::pair() {
            Ok((s1, s2)) => (s1, s2),
            Err(e) => panic!("Couldn't create pair of sockets: {:?}", e),
        };
        let handle = std::thread::spawn(move || {
            let mut ot = ChouOrlandiOT::new();
            ot.send(&mut sender, &[(m0, m1)]).unwrap();
        });
        let mut ot = ChouOrlandiOT::new();
        let results = ot.receive(&mut receiver, &[b]).unwrap();
        assert_eq!(results[0], if b { m1_ } else { m0_ });
        handle.join().unwrap();
    }
}
