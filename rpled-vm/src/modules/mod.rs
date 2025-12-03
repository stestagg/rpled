use crate::sync::Sync;
use crate::vm::{Result, VM, VmDebug};
use bitflags::bitflags;

#[macro_use]
mod define_module;

#[cfg(test)]
pub mod test;

#[cfg(feature = "led")]
pub mod led;

#[derive(Debug)]
pub enum ModuleError {
    InvalidModuleOpcode,
    IncorrectCallVariant,
}

pub const TEST_OPCODE_OFFSET: u8 = 60;
pub const LED_OPCODE_OFFSET: u8 = 64;

pub const ENABLED_MODULE_IDS: &[u8] = &[
    #[cfg(test)]
    TEST_OPCODE_OFFSET,
    #[cfg(feature = "led")]
    LED_OPCODE_OFFSET,
];

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ModuleFlags: u8 {
        const LED = 0b00000001;
        const TEST = 0b10000000;
    }
}

pub const fn offset_to_flag(offset: u8) -> Option<ModuleFlags> {
    match offset {
        LED_OPCODE_OFFSET => Some(ModuleFlags::LED),
        TEST_OPCODE_OFFSET => Some(ModuleFlags::TEST),
        _ => None,
    }
}

pub const ENABLED_MODULE_FLAGS: ModuleFlags = {
    let mut flags: u8 = 0;
    let mut i = 0;
    while i < ENABLED_MODULE_IDS.len() {
        flags |= offset_to_flag(ENABLED_MODULE_IDS[i]).unwrap().bits();
        i += 1;
    }
    ModuleFlags::from_bits(flags).unwrap()
};

trait ModuleInit {
    async fn init() -> Self
    where
        Self: Sized;
    async fn reset(&mut self) -> Result<()>;
}

#[allow(dead_code)]
pub struct Modules {
    #[cfg(test)]
    pub test: test::TestModule,

    #[cfg(feature = "led")]
    pub led: led::LedModule,
}

#[allow(dead_code)]
impl Modules {
    pub async fn init() -> Self {
        Self {
            #[cfg(test)]
            test: test::TestModule::init().await,

            #[cfg(feature = "led")]
            led: led::LedModule::init().await,
        }
    }

    pub async fn reset<const N: usize, S: Sync, D: VmDebug>(
        &mut self,
        _vm: &mut VM<N, S, D>,
    ) -> Result<()> {
        #[cfg(test)]
        test::TestModule::reset(&mut self.test).await?;

        #[cfg(feature = "led")]
        led::LedModule::reset(&mut self.led).await?;
        Ok(())
    }
}
