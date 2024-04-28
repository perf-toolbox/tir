use seq_macro::seq;
use tir_core::Operand;

macro_rules! register {
    ($($case_name:ident => { abi_name = $abi_name:literal, encoding = $encoding:literal, num = $num:literal },)*) => {
        pub enum Reg {
            $($case_name,)*
        }

        impl Into<Operand> for Reg {
            fn into(self) -> Operand {
                match self {
                $(
                    Reg::$case_name => Operand::Register($num),
                )*
                }
            }
        }

        impl TryFrom<&Operand> for Reg {
            type Error = tir_core::Error;

            fn try_from(operand: &Operand) -> Result<Reg, Self::Error> {
                match operand {
                    Operand::Register(value) => match value {
                    $(
                        $num => Ok(Reg::$case_name),
                    )*
                        _ => Err(tir_core::Error::Unknown),
                    },
                    _ => Err(tir_core::Error::Unknown),
                }
            }
        }

        pub fn get_reg_name(reg: &Reg) -> &str {
            match reg {
            $(
                Reg::$case_name => { let name = stringify!($case_name); name }
            )*
            }
        }

        pub fn get_abi_reg_name(reg: &Reg) -> &str {
            match reg {
            $(
                Reg::$case_name => $abi_name,
            )*
            }
        }

        pub fn assemble_reg(reg: &Operand) -> tir_core::Result<u8> {
            let reg = Reg::try_from(reg)?;
            match reg {
            $(
                Reg::$case_name => Ok($encoding as u8),
            )*
            }
        }
    };
}

register! {
    // Hard-wired zero
    X0 => {abi_name = "zero", encoding = 0, num = 0 },
    // Return address, saved by caller
    X1 => { abi_name = "ra", encoding = 1, num = 1 },
    // Stack pointer, saved by callee
    X2 => { abi_name = "sp", encoding = 2, num = 2 },
    // Global pointer
    X3 => { abi_name = "gp", encoding = 3, num = 3 },
    // Thread pointer
    X4 => { abi_name = "tp", encoding = 4, num = 4 },
    // Temp registers, saved by caller
    X5 => { abi_name = "t0", encoding = 5, num = 5 },
    X6 => { abi_name = "t1", encoding = 6, num = 6 },
    X7 => { abi_name = "t2", encoding = 7, num = 7 },
    // Frame pointer, saved by callee
    // FIXME: this is also a saved register 0
    X8 => { abi_name = "fp", encoding = 8, num = 8 },
    // Saved register 1, saved by callee
    X9 => { abi_name = "s1", encoding = 9, num = 9 },
    // Function argument 0 / return value 0, saved by caller
    X10 => { abi_name = "a0", encoding = 10, num = 10 },
    // Function argument 1 / return value 1, saved by caller
    X11 => { abi_name = "a1", encoding = 11, num = 11 },
    // Function arguments 2-7, saved by caller
    X12 => { abi_name = "a2", encoding = 12, num = 12 },
    X13 => { abi_name = "a3", encoding = 13, num = 13 },
    X14 => { abi_name = "a4", encoding = 14, num = 14 },
    X15 => { abi_name = "a5", encoding = 15, num = 15 },
    X16 => { abi_name = "a6", encoding = 16, num = 16 },
    X17 => { abi_name = "a7", encoding = 17, num = 17 },
    // Saved registers 2-11, saved by callee
    X18 => { abi_name = "s2", encoding = 18, num = 18 },
    X19 => { abi_name = "s3", encoding = 19, num = 19 },
    X20 => { abi_name = "s4", encoding = 20, num = 20 },
    X21 => { abi_name = "s5", encoding = 21, num = 21 },
    X22 => { abi_name = "s6", encoding = 22, num = 22 },
    X23 => { abi_name = "s7", encoding = 23, num = 23 },
    X24 => { abi_name = "s8", encoding = 24, num = 24 },
    X25 => { abi_name = "s9", encoding = 25, num = 25 },
    X26 => { abi_name = "s10", encoding = 26, num = 26 },
    X27 => { abi_name = "s11", encoding = 27, num = 27 },
    // Temporary registers 3-6, saved by caller
    X28 => { abi_name = "t3", encoding = 28, num = 28 },
    X29 => { abi_name = "t4", encoding = 29, num = 29 },
    X30 => { abi_name = "t5", encoding = 30, num = 30 },
    X31 => { abi_name = "t6", encoding = 31, num = 31 },
}

seq!(N in 0..31 {
    pub fn disassemble_gpr(value: u8) -> Option<Operand> {
        match value {
        #(
            N => Some((Reg::X~N).into()),
        )*
            _ => None,
        }
    }
});
