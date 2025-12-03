use crate::sync::Sync;
use crate::vm::{Result, VM, VmDebug};

macro_rules! bin_op {
    ($name: ident, $op:tt) => {
        pub fn $name<const N: usize, S: Sync, D: VmDebug>(vm: &mut VM<N, S, D>) -> Result<()> {
            let b: i16 = vm.stack_pop()?;
            let a: i16 = vm.stack_pop()?;
            let result = a $op b;
            vm.stack_push(result)
        }
    };
}

bin_op!(and, &);
bin_op!(or, |);
bin_op!(xor, ^);

pub fn not<const N: usize, S: Sync, D: VmDebug>(vm: &mut VM<N, S, D>) -> Result<()> {
    let a: i16 = vm.stack_pop()?;
    let result = !a;
    vm.stack_push(result)
}
