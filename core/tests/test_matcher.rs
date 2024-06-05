use tir_core::{
    builtin::{ConstOp, ModuleOp},
    Context, OpRef,
};
use tir_macros::match_op;

#[test]
fn match_ops() {
    let context = Context::new();
    let module = ModuleOp::builder(&context).build();
    let module: OpRef = module;
    let module2 = module.clone();
    let module3 = module.clone();
    let res = match_op!(module {
      ModuleOp => |_| true,
      _ => || false,
    });
    assert_eq!(res, true);

    let res = match_op!(module2 {
      ConstOp => |_| true,
      _ => || false,
    });
    assert_eq!(res, false);

    let res = match_op!(module3 {
      ConstOp => |_| false,
      ModuleOp => |_| true,
      _ => || false,
    });
    assert_eq!(res, true);
}
