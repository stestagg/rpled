#![cfg_attr(not(any(test, feature = "std")), no_std)]
// TODO: remove this when generic_const_exprs is stable
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(never_type)]

mod modules;
pub mod ops;
pub mod program;
mod read;
pub mod sync;
pub mod vm;

#[cfg(any(test, feature = "std"))]
pub mod fixture_parse;
