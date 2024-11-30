use crate::DiagKind;
use lpl::{
    combinators::{lang::ident, NotTuple},
    Diagnostic, ParseResult, Parser,
};
use seq_macro::seq;
use tir_core::{parser::Parsable, IRFormatter, IRStrStream, Printable};
use tir_macros::{lowercase, uppercase};

include!(concat!(env!("OUT_DIR"), "/defs/registers.rs"));

macro_rules! register {
    ($($case_name:ident => { abi_name = $abi_name:literal, encoding = $encoding:literal, num = $num:literal },)*) => {
        #[derive(Debug, Clone, Copy)]
        pub enum Register {
            $($case_name,)*
        }

        #[allow(clippy::from_over_into)]
        impl Into<tir_core::Attr> for Register {
            fn into(self) -> tir_core::Attr {
                tir_core::Attr::String(get_reg_name(&self).to_string())
            }
        }

        impl TryFrom<usize> for Register {
            type Error = ();
            fn try_from(value: usize) -> Result<Self, Self::Error> {
                match value {
                $(
                    $num => Ok(Register::$case_name),
                )*
                    _ => Err(())
                }
            }
        }

        pub fn get_reg_name(reg: &Register) -> &str {
            match reg {
            $(
                Register::$case_name => { let name = lowercase!($case_name); name }
            )*
            }
        }

        pub fn get_abi_reg_name(reg: &Register) -> &str {
            match reg {
            $(
                Register::$case_name => $abi_name,
            )*
            }
        }

        pub fn get_reg_num(reg: &Register) -> usize {
            match reg {
            $(
                Register::$case_name => $num,
            )*
            }
        }

        pub fn assemble_reg<T>(reg: T) -> tir_core::Result<u8> where Register: TryFrom<T> {
            let reg = Register::try_from(reg).map_err(|_| tir_core::Error::Unknown)?;
            match reg {
            $(
                Register::$case_name => Ok($encoding as u8),
            )*
            }
        }

        impl Printable for Register {
            fn print(&self, fmt: &mut dyn IRFormatter) {
                match self {
                $(
                    Register::$case_name => fmt.write_direct($abi_name),
                )*
                }
            }
        }

        pub fn register_parser(input: &str) -> Option<Register>
        {
            match input {
                $(
                    $abi_name => Some(Register::$case_name),
                    uppercase!($abi_name) => Some(Register::$case_name),
                    stringify!($case_name) => Some(Register::$case_name),
                    lowercase!($case_name) => Some(Register::$case_name),
                )*
                _ => None,
            }
        }
    };
}

impl Parsable<Register> for Register {
    fn parse(input: IRStrStream) -> ParseResult<IRStrStream, Register> {
        let parser = ident(|_| false).try_map(|r, s| {
            register_parser(r).ok_or(Into::<Diagnostic>::into(DiagKind::UnknownRegister(
                r.to_string(),
                s,
            )))
        });
        parser.parse(input)
    }
}

impl NotTuple for Register {}

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
    X8 => { abi_name = "s0", encoding = 8, num = 8 },
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
    pub fn disassemble_gpr(value: u8) -> Option<Register> {
        match value {
        #(
            N => Some(Register::X~N),
        )*
            _ => None,
        }
    }
});

#[cfg(test)]
mod tests {
    use crate::{disassemble_gpr, get_abi_reg_name, get_reg_name, Register};

    #[test]
    fn disassemble() {
        assert!(disassemble_gpr(33).is_none());
    }

    #[test]
    fn reg_name() {
        assert_eq!(get_abi_reg_name(&Register::X0), "zero");
        // TODO this should be lower case
        assert_eq!(get_reg_name(&Register::X0), "x0");
    }
}
