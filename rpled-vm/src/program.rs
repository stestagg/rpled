use crate::modules::{self, ENABLED_MODULE_FLAGS};
use crate::read::{MemoryReader, Read, ReadError};
use bytemuck::{Pod, PodCastError, Zeroable, try_from_bytes};

#[derive(Debug)]
pub enum ProgramError {
    TooShort,
    UnreadableHeader,
    InvalidMagic,
    UnexpectedVersion(u8),
    UnknownModule(u8),
    InvalidName,
    MissingRequiredModules(modules::ModuleFlags),
}

type Result<T> = core::result::Result<T, ProgramError>;

impl From<PodCastError> for ProgramError {
    fn from(_err: PodCastError) -> Self {
        ProgramError::UnreadableHeader
    }
}

impl From<ReadError> for ProgramError {
    fn from(_err: ReadError) -> Self {
        ProgramError::UnreadableHeader
    }
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C, packed)]
struct HeaderPrelude {
    magic: [u8; 3],
    version: u8,
    heap_size: u16,
    header_len: u8,
    n_modules: u8,
}
const PRELUDE_SIZE: usize = core::mem::size_of::<HeaderPrelude>();
const HEADER_LEN_OFFSET: u16 = 7; // This + header_len = total header length (3 + 1 + 2 + 1);
const SUPPORTED_VERSIONS: [u8; 1] = [0];

pub trait Program {
    fn validate_program(&self) -> Result<()>;
    fn required_modules(&self) -> Result<modules::ModuleFlags>;
    fn program_name(&self) -> Result<&str>;
    fn program_start(&self) -> Result<u16>;
}

impl Program for &[u8] {
    fn validate_program(&self) -> Result<()> {
        if self.len() < PRELUDE_SIZE {
            return Err(ProgramError::TooShort);
        }
        let prelude: &HeaderPrelude = try_from_bytes(&self[0..PRELUDE_SIZE])?;
        if &prelude.magic != b"PXS" {
            return Err(ProgramError::InvalidMagic);
        }
        if !SUPPORTED_VERSIONS.contains(&prelude.version) {
            return Err(ProgramError::UnexpectedVersion(prelude.version));
        }
        let modules = self.required_modules()?;
        let not_enabled = modules.difference(ENABLED_MODULE_FLAGS);
        if !not_enabled.is_empty() {
            return Err(ProgramError::MissingRequiredModules(not_enabled));
        }
        Ok(())
    }

    fn required_modules(&self) -> Result<modules::ModuleFlags> {
        let mut read = MemoryReader::new(self);
        let prelude: HeaderPrelude = read.read()?;
        let mut modules_enabled = modules::ModuleFlags::empty();
        for _ in 0..prelude.n_modules {
            let module_id: u8 = read.read()?;
            let module_flag = modules::offset_to_flag(module_id)
                .ok_or(ProgramError::UnknownModule(module_id))?;
            modules_enabled |= module_flag;
        }
        Ok(modules_enabled)
    }

    fn program_name(&self) -> Result<&str> {
        let prelude: &HeaderPrelude = try_from_bytes(&self[0..PRELUDE_SIZE])?;
        let name_start = PRELUDE_SIZE + (prelude.n_modules as usize);
        let name_end = prelude.header_len as usize + HEADER_LEN_OFFSET as usize;
        let name_bytes = &self[name_start..name_end];
        let name_str = core::str::from_utf8(name_bytes).map_err(|_| ProgramError::InvalidName)?;
        Ok(name_str)
    }

    fn program_start(&self) -> Result<u16> {
        let prelude: &HeaderPrelude = try_from_bytes(&self[0..PRELUDE_SIZE])?;
        let program_start = prelude.header_len as u16 + HEADER_LEN_OFFSET;
        Ok(program_start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header() {
        let program: &[u8] = &[
            b'P', b'X', b'S', // Magic
            0x00, // Version
            0x10, 0x00, // Heap Size
            10,   // Header Length (1 n_mod, 1 mod_id,  8 name)
            0x01, // Number of Modules
            60,   // Module ID (TEST)
            b'T', b'e', b's', b't', b'P', b'r', b'o', b'g', // Program Name
            0xff, 0xff, // Program Start (dummy data)
        ];

        program.validate_program().unwrap();
        assert_eq!(
            program.required_modules().unwrap(),
            modules::ModuleFlags::TEST
        );
        assert_eq!(program.program_name().unwrap(), "TestProg");
        assert_eq!(program.program_start().unwrap(), program.len() as u16 - 2);
        assert_eq!(
            program[program.program_start().unwrap() as usize..],
            [0xff, 0xff]
        );
    }
}
