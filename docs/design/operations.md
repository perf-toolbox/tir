# Defining Operations

TIR provides a crate to help developers easily define new operations.

Below is an example of a simple operation:

```rust
use crate::super_dialect::DIALECT_NAME;
use tir_core::{Attr, Context, Op, Operation, OperationImpl, Region, Type};
use tir_macros::operation;

#[operation(name = "super")]
pub struct SuperOp {
    #[cfg(operand = true)]
    operand1: Integer,
    #[cfg(operand = true)]
    operand2: Integer,
}

```

The helper macro would implement the following things for you:

- Getters and setters for operands (i.e., `get_operand1`, `set_operand1`)
- Getters for regions
- `Op`, `Into<Operation>`, `TryFrom<Operation>` traits implementations

Additional methods can be defined manually by implementing `impl SuperOp {...}`.

## Field configurations

### `#[cfg(attribute = true)]`

Defines an attribute of a specific type. Any type used in attributes must be convertible
to `tir_core::Attribute` enum. Also implements basic attribute getters and setters.

### `#[cfg(operand = true)]`

Defines an operand of a specific type. Also implements basic setters and getters
for the operand.

### `#[cfg(region = true, single_block = true)]`

Defines a region. Also defines a basic getter `get_<field_name>_region`. If
`single_block` argument is passed, also defines a `get_<field_name>` single block
getter.
