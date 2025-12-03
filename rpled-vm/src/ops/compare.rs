use crate::sync::Sync;
use crate::vm::{Result, VM, VmDebug};

macro_rules! compare_op {
    ($name: ident, $op:tt) => {
        pub fn $name<const N: usize, S: Sync, D: VmDebug>(vm: &mut VM<N, S, D>) -> Result<()> {
            let b: i16 = vm.stack_pop()?;
            let a: i16 = vm.stack_pop()?;
            let result = if a $op b { 1 } else { 0 };
            vm.stack_push(result)
        }
    };
}

compare_op!(eq, ==);
compare_op!(ne, !=);
compare_op!(lt, <);
compare_op!(gt, >);
compare_op!(le, <=);
compare_op!(ge, >=);
