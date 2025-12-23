use bytemuck::{NoUninit, Pod, bytes_of, pod_read_unaligned};

use crate::modules::{self, Modules};
use crate::ops;
use crate::program::{Program, ProgramError};
use crate::sync::{Signal, Sync};

#[derive(Debug)]
pub enum VMError {
    ProgramError(ProgramError),
    ProgramTooLarge,
    PCOverflow(u16),
    InvalidOpcode(u8, usize),
    StackOverflow,
    StackUnderflow,
    HeapOverflow,
    DivisionByZero,
    InvalidJump,
    Halt(HaltReason),
    ModuleNotEnabled(u8),
    ModuleError(crate::modules::ModuleError),
}

impl From<ProgramError> for VMError {
    fn from(err: ProgramError) -> Self {
        VMError::ProgramError(err)
    }
}

impl From<crate::modules::ModuleError> for VMError {
    fn from(err: crate::modules::ModuleError) -> Self {
        VMError::ModuleError(err)
    }
}

pub type Result<T> = core::result::Result<T, VMError>;

const MIN_STACK_SIZE: usize = 8;

#[derive(Debug)]
pub enum HaltReason {
    Signal,
    HaltOp,
    ProgramEnd,
}

pub trait VmDebug {
    fn will_run_op(&self) -> impl core::future::Future<Output = ()> + Send;
    fn did_run_op(&self) -> impl core::future::Future<Output = ()> + Send;
}

pub struct NoVmDebug;

impl VmDebug for NoVmDebug {
    async fn will_run_op(&self) {}
    async fn did_run_op(&self) {}
}

pub struct VM<const N: usize, S: Sync, D: VmDebug> {
    pub memory: [u8; N],
    pub heap_start: usize,
    pub max_pc: usize,
    pub heap_end: usize,

    pub halt_signal: S::Signal,

    pub pc: usize,
    pub sp: usize,

    pub modules: Modules,
    pub debug: D,
}

pub async fn make_vm<const N: usize, S: Sync>() -> VM<N, S, NoVmDebug> {
    VM::new(NoVmDebug).await
}

impl<const N: usize, S: Sync, D: VmDebug> VM<N, S, D> {
    // Use the generated Ops::run_op method
    pub async fn run_op(&mut self) -> Result<()> {
        let opcode: u8 = self.read_pc()?;
        ops::Ops::run_op(opcode, self).await
    }

    pub async fn new(debug: D) -> Self {
        VM {
            memory: [0; N],
            heap_start: 0,
            heap_end: 0,
            max_pc: 0,
            halt_signal: S::create_signal(),
            pc: 0,
            sp: N - 1,

            modules: Modules::init().await,
            debug,
        }
    }

    pub fn load(&mut self, program: &[u8]) -> Result<()> {
        self.memory.fill(0);

        program.validate_program()?;
        let program_start = program.program_start()?;
        let program_slice = &program[program_start as usize..];
        let program_len = program_slice.len();
        let heap_size = program_len;
        if program_len + heap_size > N - MIN_STACK_SIZE {
            return Err(VMError::ProgramTooLarge);
        }

        self.memory[0..program_len].copy_from_slice(program_slice);
        self.heap_start = program_len;
        self.max_pc = core::cmp::min(self.heap_start, u16::MAX as usize);
        self.heap_end = program_len + heap_size;
        self.pc = 0;
        self.sp = N - 1;
        Ok(())
    }

    pub fn signal_halt(&self) {
        self.halt_signal.signal();
    }

    pub async fn pause(&self) {
        self.signal_halt();
        self.halt_signal.wait_signal().await;
    }

    pub async fn reset_program(&mut self) {
        // Pause the VM
        self.pause().await;

        self.pc = 0;
        self.sp = N - 1;
    }

    pub fn set_pc(&mut self, pc: usize) -> Result<()> {
        self.pc = pc;
        if self.pc > self.max_pc {
            let pc_u16 = self.pc as u16;
            self.pc = 0;
            return Err(VMError::PCOverflow(pc_u16));
        }
        Ok(())
    }

    pub fn read_pc<T: Pod>(&mut self) -> Result<T> {
        let size = size_of::<T>();
        let start = self.pc;
        self.pc += size;
        if self.pc > self.max_pc {
            let pc_u16 = self.pc as u16;
            self.pc = 0;
            return Err(VMError::PCOverflow(pc_u16));
        }
        Ok(pod_read_unaligned::<T>(&self.memory[start..(self.pc)]))
    }

    pub fn alloc_stack_space<T>(&mut self) -> Result<&mut [u8; size_of::<T>()]> {
        let size = size_of::<T>();
        let new_sp = self.sp.checked_sub(size).ok_or(VMError::StackOverflow)?;
        if new_sp < self.heap_end {
            return Err(VMError::StackOverflow);
        }
        self.sp = new_sp;
        let slice = &mut self.memory[self.sp..(self.sp + size)];
        Ok(slice.try_into().unwrap())
    }

    pub fn stack_push<T: NoUninit>(&mut self, value: T) -> Result<()>
    where
        [(); size_of::<T>()]:,
    {
        let bytes = bytes_of(&value);
        let stack_slice = self.alloc_stack_space::<T>()?;
        stack_slice.copy_from_slice(bytes);
        Ok(())
    }

    pub fn stack_pop_raw(&mut self, size: usize) -> Result<&[u8]> {
        let start = self.sp;
        let end = start + size;
        if end > N {
            return Err(VMError::StackUnderflow);
        }
        self.sp += size;
        Ok(&self.memory[start..end])
    }

    pub fn stack_pop<T: Pod>(&mut self) -> Result<T> {
        let slice = self.stack_pop_raw(size_of::<T>())?;
        Ok(pod_read_unaligned::<T>(slice))
    }

    pub fn read_heap<T: Pod>(&self, addr: usize) -> Result<T> {
        let size = size_of::<T>();
        let start = self.heap_start + addr;
        let end = start + size;
        if end > self.heap_end {
            return Err(VMError::HeapOverflow);
        }
        Ok(pod_read_unaligned::<T>(&self.memory[start..end]))
    }

    pub fn write_heap<T: NoUninit>(&mut self, addr: usize, value: T) -> Result<()> {
        let bytes = bytes_of(&value);
        let start = self.heap_start + addr;
        let end = start + (size_of::<T>());
        if end > self.heap_end {
            return Err(VMError::HeapOverflow);
        }
        self.memory[start..end].copy_from_slice(bytes);
        Ok(())
    }

    pub async fn delay(&self, us: u16) {
        S::delay(us).await;
    }

    pub async fn run(&mut self) -> Result<!> {
        self.halt_signal.reset();

        let mut op_counter: u32 = 0;
        loop {
            if op_counter.is_multiple_of(1024)
                && self.halt_signal.is_signaled() {
                    self.halt_signal.reset();
                    return Err(VMError::Halt(HaltReason::Signal));
                }
            op_counter = op_counter.wrapping_add(1);

            self.debug.will_run_op().await;
            self.run_op().await?;
            self.debug.did_run_op().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture_parse::parse_fixture_with_output;
    use rstest::*;
    use std::path::PathBuf;

    #[rstest]
    #[tokio::test]
    async fn test_fixtures(#[files("../testprogs/*/script.pxl")] path: PathBuf) {
        let fixture_data = std::fs::read_to_string(&path).unwrap();
        let parsed = parse_fixture_with_output(&fixture_data);

        let mut actual_output = vec![];

        println!("Fixture Contents:\n{:?}", parsed.program);
        let mut vm: VM<4096, crate::sync::TokioSync, crate::vm::NoVmDebug> =
            make_vm::<4096, crate::sync::TokioSync>().await;
        match vm.load(&parsed.program) {
            Ok(()) => {
                let run_result = vm.run().await;

                vm.modules.test.messages.iter().for_each(|msg: &String| {
                    actual_output.push(msg.clone());
                });
                let result_desc = match &run_result {
                    Ok(_) => panic!("VM should never return OK from run()"),
                    Err(VMError::Halt(HaltReason::HaltOp)) => "*HALT".to_string(),
                    Err(err) => format!("Error: {:?}", err),
                };
                actual_output.push(result_desc);
            }
            Err(err) => {
                actual_output.push(format!("Load Error: {:?}", err));
            }
        }

        let actual = actual_output.join("\n");
        assert_eq!(
            actual.trim(),
            parsed.expected_output.trim(),
            "Output did not match for fixture {:?}",
            path
        );
    }
    //     let mut vm: VM<256> = VM::new();
    //     let program = [0x01, 0x02, 0x03, 0x04];
    //     vm.load(&program);
    //     assert_eq!(result, 4);
    // }
}
