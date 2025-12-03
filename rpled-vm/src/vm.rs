use bytemuck::{NoUninit, Pod, bytes_of, pod_read_unaligned};
use paste::paste;

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

macro_rules! dispatch_op {
    (
        $( $num:literal $defn:tt),+,
    ) => {
        // Generate the run_op method
        pub async fn run_op(&mut self) -> Result<()> {
            let pc = self.pc;
            let opcode: u8 = self.read_pc()?;
            match opcode {
                $(
                    $num => dispatch_op!(@call $defn, self, opcode)
                ),*
                ,
                _ => return Err(VMError::InvalidOpcode(opcode, pc)),
            }
            Ok(())
        }

        // Generate the static opcode names method
        pub fn opcode_names() -> &'static [(u8, &'static str)] {
            &[
                $(
                    dispatch_op!(@name $defn, $num)
                ),+
            ]
        }
    };

    (@call {#[cfg($cfg:meta)]$rest:tt}, $vm:expr, $opcode:ident) => {
        {
            #[cfg($cfg)]
            dispatch_op!(@call $rest, $vm, $opcode);

            #[cfg(not($cfg))]
            {
                return Err(VMError::ModuleNotEnabled($opcode));
            }
        }
    };

    (@call {$name:ident => $path:path}, $vm:expr, $opcode:ident) => {
        $path($vm)?
    };

    (@call {MOD $name:ident $method:ident $var:literal}, $vm:expr, $opcode:ident) => {
        {
            let mod_op = $vm.read_pc()?;
            modules::$name::$method::<N, S, D>($vm, mod_op).await?
        }
    };

    (@call {async $name:ident => $path:path}, $vm:expr, $opcod:ident) => {
        $path($vm).await?
    };

    (@name {#[cfg($cfg:meta)]$rest:tt}, $opcode:literal) => {
        #[cfg($cfg)]
        dispatch_op!(@name $rest, $opcode)
    };

    (@name {$name:ident => $path:path}, $opcode:literal) => {
        ($opcode, stringify!($name))
    };

    (@name {MOD $name:ident $method:ident $var:literal}, $opcode:literal) => {
        paste!{
            ($opcode, stringify!([<$name:upper $var>]))
        }
    };

    (@name {async $name:ident => $path:path}, $opcode:literal) => {
        ($opcode, stringify!($name))
    };

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
    // Generate run_op and opcode_names methods using the dispatch_op macro
    dispatch_op!(
        1 {PUSH => ops::stack::push},
        2 {LOAD => ops::stack::load},
        3 {STORE => ops::stack::store},
        4 {POP => ops::stack::pop},
        5 {POPN => ops::stack::popn},
        6 {DUP => ops::stack::dup},
        7 {SWAP => ops::stack::swap},
        8 {OVER => ops::stack::over},
        9 {ROT => ops::stack::rot},
        10 {ZERO => ops::stack::zero},

        11 {ADD => ops::math::add},
        12 {SUB => ops::math::sub},
        13 {MUL => ops::math::mul},
        14 {DIV => ops::math::div},
        15 {MOD => ops::math::modulo},

        16 {EQ => ops::compare::eq},
        17 {NE => ops::compare::ne},
        18 {LT => ops::compare::lt},
        19 {GT => ops::compare::gt},
        20 {LE => ops::compare::le},
        21 {GE => ops::compare::ge},

        22 {AND => ops::bitwise::and},
        23 {OR => ops::bitwise::or},
        24 {XOR => ops::bitwise::xor},
        25 {NOT => ops::bitwise::not},

        26 {INC => ops::math::inc},
        27 {DEC => ops::math::dec},
        28 {NEG => ops::math::neg},
        29 {ABS => ops::math::abs},
        30 {CLAMP => ops::math::clamp},
        31 {JMP => ops::control::jmp},
        32 {JZ => ops::control::jz},
        33 {JNZ => ops::control::jnz},
        34 {CALL => ops::control::call},
        35 {CALLZ => ops::control::callz},
        36 {CALLNZ => ops::control::callnz},
        37 {RET => ops::control::ret},
        38 {HALT => ops::control::halt},
        39 { async SLEEP => ops::control::sleep},

        60 {#[cfg(test)]{MOD test call0 0 }},
        61 {#[cfg(test)]{MOD test call1 1 }},
        62 {#[cfg(test)]{MOD test call2 2 }},
        63 {#[cfg(test)]{MOD test calln "N" }},

        64 {#[cfg(feature = "led")]{MOD led call0 0 }},
        65 {#[cfg(feature = "led")]{MOD led call1 1 }},
        66 {#[cfg(feature = "led")]{MOD led call2 2 }},
        67 {#[cfg(feature = "led")]{MOD led calln "N" }},

    );

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
    where
        [(); size_of::<T>()]:,
    {
        let bytes = bytes_of(&value);
        let stack_slice = self.alloc_stack_space::<T>()?;
        stack_slice.copy_from_slice(&bytes);
        Ok(())
    }

    pub fn stack_pop_raw(&mut self, size: usize) -> Result<&[u8]> {
        let start = self.sp;
        let end = start + size;
        if end > N {
            return Err(VMError::StackUnderflow);
        }
        self.sp += size;
        Ok(&self.memory[start as usize..end as usize])
    }

    pub fn stack_pop<T: Pod>(&mut self) -> Result<T> {
        let slice = self.stack_pop_raw(size_of::<T>())?;
        Ok(pod_read_unaligned::<T>(slice).clone())
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

    pub fn write_heap<T: NoUninit>(&mut self, addr: usize, value: T) -> Result<()> {
        let bytes = bytes_of(&value);
        let start = self.heap_start + addr;
        let end = start + (size_of::<T>());
        if end > self.heap_end {
            return Err(VMError::HeapOverflow);
        }
        self.memory[start as usize..end as usize].copy_from_slice(&bytes);
        Ok(())
    }

    pub async fn delay(&self, us: u16) {
        S::delay(us).await;
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
    use std::{path::PathBuf, result};

    #[rstest]
    #[tokio::test]
    async fn test_fixtures(#[files("../testprogs/*.pxs.txt")] path: PathBuf) {
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
