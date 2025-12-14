pub mod bitwise;
pub mod compare;
pub mod control;
pub mod math;
pub mod stack;

use ops_derive::DecodeOps;
use strum::{Display, EnumDiscriminants, EnumIter, EnumString, FromRepr, VariantArray, VariantNames};
use crate::sync::Sync;
use crate::vm::{Result, VM, VmDebug};


#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, FromRepr)]
#[derive(DecodeOps)]
pub enum Ops {
    // Stack operations
    Push { value: u16 } = 1,
    Load { heap_addr: u16 } = 2,
    Store { heap_addr: u16 } = 3,
    Pop = 4,
    PopN { count: u8 } = 5,
    Dup = 6,
    Swap = 7,
    Over = 8,
    Rot = 9,
    Zero = 10,
    LoadFrame { frame_addr: u16 } = 11,
    StoreFrame { frame_addr: u16 } = 12,

    // Math operations
    Add = 13,
    Sub = 14,
    Mul = 15,
    Div = 16,
    Mod = 17,

    // Comparison operations
    Eq = 18,
    Ne = 19,
    Lt = 20,
    Gt = 21,
    Le = 22,
    Ge = 23,

    // Bitwise operations
    And = 24,
    Or = 25,
    Xor = 26,
    Not = 27,

    // More math operations
    Inc = 28,
    Dec = 29,
    Neg = 30,
    Abs = 31,
    Clamp = 32,

    // Control flow operations
    Jmp { offset: i16 } = 33,
    Jz { offset: i16 } = 34,
    Jnz { offset: i16 } = 35,
    Call { offset: i16, frame_entries: u8 } = 36,
    CallZ { offset: i16, frame_entries: u8 } = 37,
    CallNz { offset: i16, frame_entries: u8 } = 38,
    Ret = 39,
    Halt = 40,
    Sleep = 41,

    // Module operations (test)
    #[cfg(test)]
    TestCall0 = 60,
    #[cfg(test)]
    TestCall1 = 61,
    #[cfg(test)]
    TestCall2 = 62,
    #[cfg(test)]
    TestCallN = 63,

    // Module operations (led)
    #[cfg(feature = "led")]
    LedCall0 = 64,
    #[cfg(feature = "led")]
    LedCall1 = 65,
    #[cfg(feature = "led")]
    LedCall2 = 66,
    #[cfg(feature = "led")]
    LedCallN = 67,
}
