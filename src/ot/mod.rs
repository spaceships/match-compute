// -*- mode: rust; -*-
//
// This file is part of ocelot.
// Copyright © 2019 Galois, Inc.
// See LICENSE for licensing information.

mod alsz;
mod chou_orlandi;
mod dummy;
mod iknp;
mod naor_pinkas;

pub use alsz::AlszOT;
pub use chou_orlandi::ChouOrlandiOT;
pub use dummy::DummyOT;
pub use iknp::IknpOT;
pub use naor_pinkas::NaorPinkasOT;

use crate::Block;
use failure::Error;
use std::io::{Read, Write};

/// Oblivious transfer trait.
pub trait ObliviousTransfer<T: Read + Write + Send> {
    /// Creates a new oblivious transfer instance using `stream` for I/O.
    fn new() -> Self;
    /// Sends values of `nbytes` each.
    fn send(
        &mut self,
        stream: &mut T,
        inputs: &[(Vec<u8>, Vec<u8>)],
        nbytes: usize,
    ) -> Result<(), Error>;
    /// Receives values of `nbytes` each.
    fn receive(
        &mut self,
        stream: &mut T,
        inputs: &[bool],
        nbytes: usize,
    ) -> Result<Vec<Vec<u8>>, Error>;
}

/// Oblivious transfer trait for 128-bit inputs.
pub trait BlockObliviousTransfer<T: Read + Write + Send> {
    /// Creates a new oblivious transfer instance using `stream` for I/O.
    fn new() -> Self;
    /// Sends values of `nbytes` each.
    fn send(&mut self, stream: &mut T, inputs: &[(Block, Block)]) -> Result<(), Error>;
    /// Receives values of `nbytes` each.
    fn receive(&mut self, stream: &mut T, inputs: &[bool]) -> Result<Vec<Block>, Error>;
}
