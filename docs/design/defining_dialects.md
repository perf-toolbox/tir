# Defining Dialects

## Intro

TBD

## Defining Operations

TIR provides a crate to help developers easily define new operations.

Below is an example of a simple operation:

```rust
use crate::builtin::DIALECT_NAME;
use tir_core::{Op, OpImpl, Type};
use tir_macros::{Assembly, Op};

#[derive(Op, Assembly)]
#[operation(name = "super", known_attrs(value: IntegerAttr))]
pub struct SuperOp {
    #[operand]
    operand1: Register,
    #[ret_type]
    return_type: Type,
    r#impl: OpImpl,
}

```

The helper macros would implement the following things for you:

- Getters and setters for operands (i.e., `get_operand1`, `set_operand1`)
- Getters for regions
- Default assembly parser and printer.

Additional methods can be defined manually by implementing `impl SuperOp {...}`.

### Field configurations

**`#[operation(..., known_attrs(attr1: AttrType))]`**

Defines an attribute of a specific type. Any type used in attributes must be convertible
to `tir_core::Attribute` enum. Also implements basic attribute getters and setters.

**`#[operand]`**

Defines an operand of a specific type. Also implements basic setters and getters
for the operand.

**`#[region(single_block, no_args)]`**

Defines a region. Also defines a basic getter `get_<field_name>_region`. If
`single_block` argument is passed, also defines a `get_<field_name>` single block
getter. If both `single_block` and `no_args` are passed, a default region will be
created during operation building.
