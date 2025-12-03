use paste::paste;

macro_rules! define_module {
    (
        $mod_name:ident ( $vm_ident:ident ) {
            $(
                $opcode:literal => async fn $name:ident $args:tt -> Result<()> $body:block
            ),* $(,)?
        }
    ) => {
        paste! {
            // Arg structs for each opcode
            mod args {
                use paste::paste;
                $(
                    define_module!(@args_struct $name, $args);
                )*
            }

            // Implementations as functions
            mod impls {
                $(
                    define_module!(@fn_impl $name, $args $body);
                )*
            }

            pub(crate) async fn call0<const N: usize, S: crate::sync::Sync, D: crate::vm::VmDebug>(
                $vm_ident: &mut crate::vm::VM<N, S, D>,
                opcode: u8
            ) -> crate::vm::Result<()> {
                match opcode {
                    $(
                        $opcode => define_module!(@call0 $vm_ident, $name, $args),
                    )*
                    _ => Err(crate::modules::ModuleError::InvalidModuleOpcode.into()),
                }
            }

            pub(crate) async fn call1<const N: usize, S: crate::sync::Sync, D: crate::vm::VmDebug>(
                $vm_ident: &mut crate::vm::VM<N, S, D>,
                opcode: u8
            ) -> crate::vm::Result<()> {
                match opcode {
                    $(
                        $opcode => define_module!(@calln $vm_ident,1, $name, $args),
                    )*
                    _ => Err(crate::modules::ModuleError::InvalidModuleOpcode.into()),
                }
            }

            pub(crate) async fn call2<const N: usize, S: crate::sync::Sync, D: crate::vm::VmDebug>(
                $vm_ident: &mut crate::vm::VM<N, S, D>,
                opcode: u8
            ) -> crate::vm::Result<()> {
                match opcode {
                    $(
                        $opcode => define_module!(@calln $vm_ident,2, $name, $args),
                    )*
                    _ => Err(crate::modules::ModuleError::InvalidModuleOpcode.into()),
                }
            }

            pub(crate) async fn calln<const N: usize, S: crate::sync::Sync, D: crate::vm::VmDebug>(
                $vm_ident: &mut crate::vm::VM<N, S, D>,
                opcode: u8
            ) -> crate::vm::Result<()> {
                let n_arg: u8 = $vm_ident.read_pc()?; // Number of arguments
                match opcode {
                    $(
                        $opcode => define_module!(@calln $vm_ident,n_arg, $name, $args),
                    )*
                    _ => Err(crate::modules::ModuleError::InvalidModuleOpcode.into()),
                }
            }
        }
    };

    (@args_struct $name:ident, (&mut  $vm_name:ident $(, $arg:ident : $arg_ty:ty )* ) ) => {
        paste! {
            #[repr(C)]
            #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
            pub struct [<$name:camel>] {
                $( pub $arg : $arg_ty, )*
            }
        }
    };

    (@fn_impl $name:ident, ( &mut $vm_name:ident $(, $arg:ident : $arg_ty:ty )* ) $body:block) => {
        pub async fn $name<const N: usize, S: crate::sync::Sync, D: crate::vm::VmDebug>(
            $vm_name: &mut crate::vm::VM<N, S, D>,
            $( $arg : $arg_ty ),*
        ) -> crate::vm::Result<()> {
            $body
        }
    };

    (@call0 $vm_ident:ident, $name:ident, (&mut  $vm_name:ident) ) => {
        {
            impls::$name( $vm_ident ).await
        }
    };

    (@call0 $vm_ident:ident, $name:ident, (&mut  $vm_name:ident , $( $arg:ident : $arg_ty:ty ),+ ) ) => {
        {
            return Err(crate::modules::ModuleError::IncorrectCallVariant.into());
        }
    };

    (@calln $vm_ident:ident, $num:expr, $name:ident, (&mut $vm_name:ident ) ) => {
        {
            return Err(crate::modules::ModuleError::IncorrectCallVariant.into());
        }
    };

    (@calln $vm_ident:ident, $num:expr, $name:ident, (&mut $vm_name:ident , $( $arg:ident : $arg_ty:ty ),+ ) ) => {
        paste! {
            {
                if (size_of::<i16>() * ($num as usize)) != size_of::<args::[<$name:camel>]>() {
                    return Err(crate::modules::ModuleError::IncorrectCallVariant.into());
                }
                let args: args::[<$name:camel>] = $vm_name.stack_pop()?;
                impls::$name(
                    $vm_ident,
                    $( args.$arg ),*
                ).await
            }
        }
    };

}
