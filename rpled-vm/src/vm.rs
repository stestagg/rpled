use bytemuck::{NoUninit, Pod, bytes_of, pod_read_unaligned};

use crate::ops;
use crate::program::{Program, ProgramError};
use crate::sync::Signal;

pub enum VMError {
    ProgramError(ProgramError),
    ProgramTooLarge,
    PCOverflow,
    InvalidOpcode(u8, usize),
    StackOverflow,
    StackUnderflow,
    HeapOverflow,
    DivisionByZero,
    InvalidJump,
    Halt(HaltReason),
}

impl From<ProgramError> for VMError {
    fn from(err: ProgramError) -> Self {
        VMError::ProgramError(err)
    }
}

pub type Result<T> = core::result::Result<T, VMError>;

const MIN_STACK_SIZE: usize = 8;

pub enum HaltReason {
    Signal,
    HaltOp,
    ProgramEnd,
}

macro_rules! dispatch_op {
    (
        $value:expr, $vm:expr, $pc:expr;
        $( $num:literal ($name:ident) => $mode:ident $path:path ),+ $(,)?
    ) => {
        match $value {
            $(
                $num => {
                    dispatch_op!(@call $mode, $path, $vm)
                }
            ),+
            ,
            _ => return Err(VMError::InvalidOpcode($value, $pc)),
        }
    };

    // sync → direct call
    (@call sync, $path:path, $vm:expr) => {
        $path($vm)
    };

    // async → call + await
    (@call async, $path:path, $vm:expr) => {
        $path($vm).await
    };
}



pub struct VM<const N: usize, S: Signal> {
    pub memory: [u8; N],
    pub heap_start: usize,
    pub max_pc: usize,
    pub heap_end: usize,

    pub halt_signal: S,

    pub pc: usize,
    pub sp: usize,
}

impl<const N: usize, S: Signal> VM<N, S> {
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

    pub fn set_pc(&mut self, pc: usize)-> Result<()> {
        self.pc = pc;
        if self.pc >= self.max_pc {
            self.pc = 0;
            return Err(VMError::PCOverflow);
        }
        Ok(())
    }

    pub fn read_pc<T: Pod>(&mut self) -> Result<T> {
        let size = size_of::<T>();
        let start = self.pc;
        self.pc += size;
        if self.pc >= self.max_pc {
            self.pc = 0;
            return Err(VMError::PCOverflow);
        }
        Ok(pod_read_unaligned::<T>(&self.memory[start.into()..(self.pc).into()]).clone())
    }

    pub fn alloc_stack_space<T>(&mut self) -> Result<&mut [u8; size_of::<T>()]> {
        let size = size_of::<T>();
        let new_sp = self.sp.checked_sub(size).ok_or(VMError::StackOverflow)?;
        if new_sp < self.heap_end {
            return Err(VMError::StackOverflow);
        }
        self.sp = new_sp;
        let slice = &mut self.memory[self.sp as usize..(self.sp + size) as usize];
        Ok(slice.try_into().unwrap())
    }

    pub fn stack_push<T: NoUninit>(&mut self, value: T) -> Result<()>
     where [(); size_of::<T>()]:
    {
        let bytes = bytes_of(&value);
        let stack_slice = self.alloc_stack_space::<T>()?;
        stack_slice.copy_from_slice(&bytes);
        Ok(())
    }

    pub fn stack_pop<T: Pod>(&mut self) -> Result<T>
    {
        let start = self.sp;
        let end = start + (size_of::<T>());
        if end > N {
            return Err(VMError::StackUnderflow);
        }
        self.sp += size_of::<T>();
        Ok(pod_read_unaligned::<T>(&self.memory[start as usize..end as usize]).clone())
    }

    pub fn read_heap<T: Pod>(&self, addr: usize) -> Result<T> {
        let size = size_of::<T>();
        let start = self.heap_start + addr;
        let end = start + size;
        if end > self.heap_end {
            return Err(VMError::HeapOverflow);
        }
        Ok(pod_read_unaligned::<T>(&self.memory[start.into()..end.into()]).clone())
    }

    pub fn write_heap<T: NoUninit>(&mut self, addr: usize, value: T) -> Result<()> 
    {
        let bytes = bytes_of(&value);
        let start = self.heap_start + addr;
        let end = start + (size_of::<T>());
        if end > self.heap_end {
            return Err(VMError::HeapOverflow);
        }
        self.memory[start as usize..end as usize].copy_from_slice(&bytes);
        Ok(())
    }

    pub async fn run(&mut self) -> Result<!> {
        self.halt_signal.reset();

        let mut op_counter: u32 = 0;
        loop {
            if op_counter % 1024 == 0 {
                if self.halt_signal.is_signaled() {
                    self.halt_signal.reset();
                    return Err(VMError::Halt(HaltReason::Signal));
                }
            }
            op_counter = op_counter.wrapping_add(1);

            let pc = self.pc;
            let opcode: u8 = self.read_pc()?;

            dispatch_op!(opcode, self, pc;
                1 (PUSH) => sync ops::stack::push,
                2 (LOAD) => sync ops::stack::load,
                3 (STORE) => sync ops::stack::store,
                4 (POP) => sync ops::stack::pop,
                5 (POPN) => sync ops::stack::popn,
                6 (DUP) => sync ops::stack::dup,
                7 (SWAP) => sync ops::stack::swap,
                8 (OVER) => sync ops::stack::over,
                9 (ROT) => sync ops::stack::rot,
                10 (ZERO) => sync ops::stack::zero,

                11 (ADD) => sync ops::math::add,
                12 (SUB) => sync ops::math::sub,
                13 (MUL) => sync ops::math::mul,
                14 (DIV) => sync ops::math::div,
                15 (MOD) => sync ops::math::modulo,

                16 (EQ) => sync ops::compare::eq,
                17 (NE) => sync ops::compare::ne,
                18 (LT) => sync ops::compare::lt,
                19 (GT) => sync ops::compare::gt,
                20 (LE) => sync ops::compare::le,
                21 (GE) => sync ops::compare::ge,

                22 (AND) => sync ops::bitwise::and,
                23 (OR) => sync ops::bitwise::or,
                24 (XOR) => sync ops::bitwise::xor,
                25 (NOT) => sync ops::bitwise::not,
                
                26 (INC) => sync ops::math::inc,
                27 (DEC) => sync ops::math::dec,
                28 (NEG) => sync ops::math::neg,
                29 (ABS) => sync ops::math::abs,
                30 (CLAMP) => sync ops::math::clamp,
                31 (JMP) => sync ops::control::jmp,
                32 (JZ) => sync ops::control::jz,
                33 (JNZ) => sync ops::control::jnz,
                34 (CALL) => sync ops::control::call,
                35 (CALLZ) => sync ops::control::callz,
                36 (CALLNZ) => sync ops::control::callnz,
                37 (RET) => sync ops::control::ret,
                38 (HALT) => sync ops::control::halt,
                39 (SLEEP) => async ops::control::sleep,
            );
        }
    }
}

impl<const N: usize, S: Signal + Default> Default for VM<N, S> {
    fn default() -> Self {
        VM {
            memory: [0; N],
            heap_start: 0,
            heap_end: 0,
            halt_signal: S::default(),
            pc: 0,
            sp: N - 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_load() {
    //     let mut vm: VM<256> = VM::new();
    //     let program = [0x01, 0x02, 0x03, 0x04];
    //     vm.load(&program);
    //     assert_eq!(result, 4);
    // }
}
