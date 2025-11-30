use bitflags::bitflags;

#[cfg(feature = "led")]
pub mod led;

pub const LED_OPCODE_OFFSET: u8 = 64;

pub const ENABLED_MODULE_IDS: &[u8] = &[
    #[cfg(feature = "led")]
    LED_OPCODE_OFFSET,
];


bitflags! {
    pub struct ModuleFlags: u8 {
        const LED = 0b00000001;
    }
}

pub fn offset_to_flag(offset: u8) -> Option<ModuleFlags> {
    match offset {
        LED_OPCODE_OFFSET => Some(ModuleFlags::LED),
        _ => None,
    }
}


pub trait Module {}
