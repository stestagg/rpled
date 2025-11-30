use bytemuck::{Pod, Zeroable};
use crate::vm::{VM, Result, VMError};
use crate::sync::Signal;

macro_rules! bin_op {
    ($name: ident, $meth:ident) => {
        pub fn $name<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
            let b: i16 = vm.stack_pop()?;
            let a: i16 = vm.stack_pop()?;
            let result = a.$meth(b);
            vm.stack_push(result)
        }
    };
}

bin_op!(add, wrapping_add);
bin_op!(sub, wrapping_sub);
bin_op!(mul, wrapping_mul);

pub fn div<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    let b: i16 = vm.stack_pop()?;
    let a: i16 = vm.stack_pop()?;
    if b == 0 {
        return Err(VMError::DivisionByZero);
    }
    let result = a.wrapping_div(b);
    vm.stack_push(result)
}

pub fn modulo<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    let b: i16 = vm.stack_pop()?;
    let a: i16 = vm.stack_pop()?;
    if b == 0 {
        return Err(VMError::DivisionByZero);
    }
    let result = a.wrapping_rem(b);
    vm.stack_push(result)
}

pub fn inc<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    let a: i16 = vm.stack_pop()?;
    let result = a.wrapping_add(1);
    vm.stack_push(result)
}

pub fn dec<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    let a: i16 = vm.stack_pop()?;
    let result = a.wrapping_sub(1);
    vm.stack_push(result)
}

pub fn neg<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    let a: i16 = vm.stack_pop()?;
    let result = a.wrapping_neg();
    vm.stack_push(result)
}

pub fn abs<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    let a: i16 = vm.stack_pop()?;
    let result = a.wrapping_abs();
    vm.stack_push(result)
}
 

#[derive(Pod, Copy, Clone, Zeroable)]
#[repr(packed, C)]
struct ClampVals {
    max: i16,
    min: i16,
    val: i16,
}
impl ClampVals {
    pub fn clamp(self) -> i16 {
        if self.val < self.min {
            self.min
        } else if self.val > self.max {
            self.max    
        } else {
            self.val
        }
    }
}

pub fn clamp<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    let vals : ClampVals = vm.stack_pop()?;
    vm.stack_push(vals.clamp())
}