use crate::vm::Result;
use paste::paste;

extern crate std;

pub struct LedModule {}

impl super::ModuleInit for LedModule {
    async fn init() -> Self {
        LedModule {}
    }

    async fn reset(&mut self) -> Result<()> {
        Ok(())
    }
}

define_module! {
    led (vm) {
        1 => async fn test_no_args(&mut vm) -> Result<()> {
            Ok(())
        },
        // 3 => async fn test_two_args(&module, &vm, arg1: i16, arg2: i16) -> Result<()> {
        //     module.messages.push(format!("Two Args: {}, {}", arg1, arg2));
        //     Ok(())
        // },
        // 4 => async fn test_four_u8(&module, &vm, a: u8, b: u8, c: u8, d: u8) -> Result<()> {
        //     module.messages.push(format!("Four u8 Args: {}, {}, {}, {}", a, b, c, d));
        //     Ok(())
        // },
    }
}
