use super::ModuleInit;
use crate::vm::{Result, VM};
use paste::paste;

extern crate std;

use std::format;
use std::string::{String, ToString};
use std::vec::Vec;

pub struct TestModule {
    pub messages: Vec<String>,
}

impl super::ModuleInit for TestModule {
    async fn init() -> Self {
        TestModule {
            messages: Vec::new(),
        }
    }

    async fn reset(&mut self) -> Result<()> {
        self.messages.clear();
        Ok(())
    }
}

define_module! {
    test (vm) {
        1 => async fn test_no_args(&mut vm) -> Result<()> {
            std::println!("TEST_NO_ARGS called");
            vm.modules.test.messages.push("TEST_NO_ARGS".to_string());
            Ok(())
        },
        2 => async fn test_one_arg(&mut vm, arg1: i16) -> Result<()> {
            std::println!("TEST_ONE_ARG called with arg1: {}", arg1);
            vm.modules.test.messages.push(format!("TEST_ONE_ARG: {}", arg1));
            Ok(())
        },
        3 => async fn test_two_args(&mut vm, arg1: i16, arg2: i16) -> Result<()> {
            std::println!("TEST_TWO_ARGS called with arg1: {}, arg2: {}", arg1, arg2);
            vm.modules.test.messages.push(format!("TEST_TWO_ARGS: {}, {}", arg1, arg2));
            Ok(())
        },
        4 => async fn test_four_u8(&mut vm, a: u8, b: u8, c: u8, d: u8) -> Result<()> {
            std::println!("TEST_FOUR_U8 called with a: {}, b: {}, c: {}, d: {}", a, b, c, d);
            vm.modules.test.messages.push(format!("TEST_FOUR_U8: {}, {}, {}, {}", a, b, c, d));
            Ok(())
        },
        5 => async fn test_print(&mut vm, msg_ptr: u16, msg_len: u16) -> Result<()> {
            let msg_bytes = vm.memory[msg_ptr as usize..(msg_ptr + msg_len) as usize].to_vec();
            let msg = String::from_utf8_lossy(&msg_bytes).to_string();
            std::println!("TEST_PRINT called with message: {} (*{}, {})", msg, msg_ptr, msg_len);
            vm.modules.test.messages.push(format!("TEST_PRINT: {:?}", msg));
            Ok(())
        },
    }
}
