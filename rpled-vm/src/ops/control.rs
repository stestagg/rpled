use crate::sync::Sync;
use crate::vm::{HaltReason, Result, VM, VMError, VmDebug};

#[inline]
fn do_jmp<const N: usize, S: Sync, D: VmDebug>(vm: &mut VM<N, S, D>, addr: i16) -> Result<()> {
    let new_pc = vm.pc as isize + addr as isize;
    if new_pc < 0 || new_pc as usize >= vm.memory.len() {
        return Err(crate::vm::VMError::InvalidJump);
    }
    vm.pc = new_pc as usize;
    Ok(())
}

pub fn jmp<const N: usize, S: Sync, D: VmDebug>(vm: &mut VM<N, S, D>, offset: i16) -> Result<()> {
    do_jmp(vm, offset)
}

pub fn jz<const N: usize, S: Sync, D: VmDebug>(vm: &mut VM<N, S, D>, offset: i16) -> Result<()> {
    let cond: i16 = vm.stack_pop()?;
    if cond == 0 {
        do_jmp(vm, offset)?;
    }
    Ok(())
}

pub fn jnz<const N: usize, S: Sync, D: VmDebug>(vm: &mut VM<N, S, D>, offset: i16) -> Result<()> {
    let cond: i16 = vm.stack_pop()?;
    if cond != 0 {
        do_jmp(vm, offset)?;
    }
    Ok(())
}

fn do_call<const N: usize, S: Sync, D: VmDebug>(vm: &mut VM<N, S, D>, addr: i16) -> Result<()> {
    let ret_addr = vm.pc;
    vm.stack_push(ret_addr as u16)?;
    do_jmp(vm, addr)
}

pub fn call<const N: usize, S: Sync, D: VmDebug>(vm: &mut VM<N, S, D>, offset: i16, _frame_entries: u8) -> Result<()> {
    do_call(vm, offset)
}

pub fn callz<const N: usize, S: Sync, D: VmDebug>(vm: &mut VM<N, S, D>, offset: i16, _frame_entries: u8) -> Result<()> {
    let cond: i16 = vm.stack_pop()?;
    if cond == 0 {
        do_call(vm, offset)?;
    }
    Ok(())
}
pub fn callnz<const N: usize, S: Sync, D: VmDebug>(vm: &mut VM<N, S, D>, offset: i16, _frame_entries: u8) -> Result<()> {
    let cond: i16 = vm.stack_pop()?;
    if cond != 0 {
        do_call(vm, offset)?;
    }
    Ok(())
}

pub fn ret<const N: usize, S: Sync, D: VmDebug>(vm: &mut VM<N, S, D>) -> Result<()> {
    let ret_addr: u16 = vm.stack_pop()?;
    vm.set_pc(ret_addr as usize)
}

pub fn halt<const N: usize, S: Sync, D: VmDebug>(_vm: &mut VM<N, S, D>) -> Result<()> {
    Err(VMError::Halt(HaltReason::HaltOp))
}

pub async fn sleep<const N: usize, S: Sync, D: VmDebug>(vm: &mut VM<N, S, D>) -> Result<()> {
    let duration_us: u16 = vm.stack_pop()?;
    vm.delay(duration_us).await;
    Ok(())
}
