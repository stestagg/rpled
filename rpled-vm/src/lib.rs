#![cfg_attr(not(test), no_std)]
// TODO: remove this when generic_const_exprs is stable
#![allow(incomplete_features)]
#![feature(f16)]
#![feature(generic_const_exprs)]
#![feature(never_type)]

mod modules;
pub mod ops;
pub mod program;
mod read;
pub mod sync;
pub mod vm;

#[cfg(test)]
mod fixture_parse;
