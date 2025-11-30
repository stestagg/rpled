use bytemuck::checked::pod_read_unaligned;
use bytemuck::{cast_mut, from_bytes_mut};

use crate::vm::{VM, Result, VMError};
use crate::sync::Signal;

pub fn push<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    let value: u16 = vm.read_pc()?;
    vm.stack_push(value)
}

pub fn load<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    let addr: u16 = vm.read_pc()?;
    let value: u16 = vm.read_heap(addr as usize)?;
    vm.stack_push(value)
}

pub fn store<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    let addr: u16 = vm.read_pc()?;
    let stack_value: u16 = vm.stack_pop()?;
    vm.write_heap(addr as usize, stack_value)
}

pub fn pop<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    let _value: u16 = vm.stack_pop()?;
    Ok(())
}

pub fn popn<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    let count: u8 = vm.read_pc()?;
    let new_sp = vm.sp.checked_add(count as usize).ok_or(VMError::StackUnderflow)?;
    if new_sp > vm.memory.len() {
        return Err(VMError::StackUnderflow);
    }
    vm.sp = new_sp;
    Ok(())
}

pub fn dup<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    let value: u16 = pod_read_unaligned(&vm.memory[vm.sp..(vm.sp + 2)]);
    vm.stack_push(value)
}

pub fn swap<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    if vm.sp + 4 > vm.memory.len() {
        return Err(VMError::StackUnderflow);
    }
    let bytes_arr: &mut [u8; 4] = from_bytes_mut(&mut vm.memory[vm.sp..(vm.sp + 4)]);
    let halves: &mut [u16; 2] = cast_mut(bytes_arr);
    halves.swap(0, 1);
    Ok(())
}

pub fn over<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    if vm.sp + 4 > vm.memory.len() {
        return Err(VMError::StackUnderflow);
    }
    let value: u16 = pod_read_unaligned(&vm.memory[(vm.sp + 2)..(vm.sp + 4)]);
    vm.stack_push(value)
}

pub fn rot<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    if vm.sp + 6 > vm.memory.len() {
        return Err(VMError::StackUnderflow);
    }
    let bytes_arr: &mut [u8; 6] = from_bytes_mut(&mut vm.memory[vm.sp..(vm.sp + 6)]);
    let thirds: &mut [u16; 3] = cast_mut(bytes_arr);
    thirds.rotate_left(1);
    Ok(())
}

pub fn zero<const N: usize, S: Signal>(vm: &mut VM<N, S>) -> Result<()> {
    vm.stack_push(0)
}