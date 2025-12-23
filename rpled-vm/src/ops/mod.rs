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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, FromRepr, DecodeOps)]
pub enum Op {
    // Stack operations
    #[handler(crate::ops::stack::push)]
    Push { value: u16 } = 1,
    #[handler(crate::ops::stack::load)]
    Load { heap_addr: u16 } = 2,
    #[handler(crate::ops::stack::store)]
    Store { heap_addr: u16 } = 3,
    #[handler(crate::ops::stack::pop)]
    Pop = 4,
    #[handler(crate::ops::stack::popn)]
    PopN { count: u8 } = 5,
    #[handler(crate::ops::stack::dup)]
    Dup = 6,
    #[handler(crate::ops::stack::swap)]
    Swap = 7,
    #[handler(crate::ops::stack::over)]
    Over = 8,
    #[handler(crate::ops::stack::rot)]
    Rot = 9,
    #[handler(crate::ops::stack::zero)]
    Zero = 10,
    #[handler(crate::ops::stack::load_frame)]    
    LoadFrame { frame_addr: u16 } = 11,
    #[handler(crate::ops::stack::store_frame)]
    StoreFrame { frame_addr: u16 } = 12,

    // Math operations
    #[handler(crate::ops::math::add)]
    Add = 13,
    #[handler(crate::ops::math::sub)]
    Sub = 14,
    #[handler(crate::ops::math::mul)]
    Mul = 15,
    #[handler(crate::ops::math::div)]
    Div = 16,
    #[handler(crate::ops::math::modulo)]
    Mod = 17,

    // Comparison operations
    #[handler(crate::ops::compare::eq)]
    Eq = 18,
    #[handler(crate::ops::compare::ne)]
    Ne = 19,
    #[handler(crate::ops::compare::lt)]
    Lt = 20,
    #[handler(crate::ops::compare::gt)]
    Gt = 21,
    #[handler(crate::ops::compare::le)]
    Le = 22,
    #[handler(crate::ops::compare::ge)]
    Ge = 23,

    // Bitwise operations
    #[handler(crate::ops::bitwise::and)]
    And = 24,
    #[handler(crate::ops::bitwise::or)]
    Or = 25,
    #[handler(crate::ops::bitwise::xor)]
    Xor = 26,
    #[handler(crate::ops::bitwise::not)]
    Not = 27,

    // More math operations
    #[handler(crate::ops::math::inc)]
    Inc = 28,
    #[handler(crate::ops::math::dec)]
    Dec = 29,
    #[handler(crate::ops::math::neg)]
    Neg = 30,
    #[handler(crate::ops::math::abs)]
    Abs = 31,
    #[handler(crate::ops::math::clamp)]
    Clamp = 32,

    // Control flow operations
    #[handler(crate::ops::control::jmp)]
    Jmp { offset: i16 } = 33,
    #[handler(crate::ops::control::jz)]
    Jz { offset: i16 } = 34,
    #[handler(crate::ops::control::jnz)]
    Jnz { offset: i16 } = 35,
    #[handler(crate::ops::control::call)]
    Call { offset: i16, frame_entries: u8 } = 36,
    #[handler(crate::ops::control::callz)]
    CallZ { offset: i16, frame_entries: u8 } = 37,
    #[handler(crate::ops::control::callnz)]
    CallNz { offset: i16, frame_entries: u8 } = 38,
    #[handler(crate::ops::control::ret)]
    Ret = 39,
    #[handler(crate::ops::control::halt)]
    Halt = 40,
    #[handler(crate::ops::control::sleep)]
    Sleep = 41,

    // Module operations (test)
    #[cfg(test)]
    #[handler(crate::modules::test::call0)]
    TestCall0 = 60,
    #[cfg(test)]
    #[handler(crate::modules::test::call1)]
    TestCall1 = 61,
    #[cfg(test)]
    #[handler(crate::modules::test::call2)]
    TestCall2 = 62,
    #[cfg(test)]
    #[handler(crate::modules::test::calln)]
    TestCallN = 63,

    // Module operations (led)
    #[cfg(feature = "led")]
    #[handler(crate::modules::led::call0)]
    LedCall0 = 64,
    #[cfg(feature = "led")]
    #[handler(crate::modules::led::call1)]
    LedCall1 = 65,
    #[cfg(feature = "led")]
    #[handler(crate::modules::led::call2)]
    LedCall2 = 66,
    #[cfg(feature = "led")]
    #[handler(crate::modules::led::calln)]
    LedCallN = 67,
}
