use seq_macro::seq;
use tir_core::Operand;

seq!(N in 0..=31 {
  pub enum Reg {
      #(
          X~N,
      )*
      #(
          F~N,
      )*
  }
});

impl Into<Operand> for Reg {
    fn into(self) -> Operand {
        seq!(N in 0..=31 {
          let result = match self {
              #(
              Reg::X~N => Operand::Register(N),
              )*
              #(
              Reg::F~N => Operand::Register(31 + N),
              )*
          };
        });

        result
    }
}
