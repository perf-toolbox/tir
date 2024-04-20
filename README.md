[![Coverage Status](https://coveralls.io/repos/github/perf-toolbox/tir/badge.svg?branch=main)](https://coveralls.io/github/perf-toolbox/tir?branch=main)

# Target Intermediate Representation

TIR  is a collection of modular tools designed for efficient and flexible
binary analysis, particularly for ELF binaries. Inspired by projects like
LLVM and MLIR, TIR aims to provide a comprehensive and extensible platform for
various binary analysis tasks.

## Goals and non-goals

### Goals

- **Flexible Binary Analysis:** Provide a modular and extensible framework for
  analyzing ELF binaries, enabling developers to create custom analysis tools
  and algorithms.
- **Integration with Existing Projects:** Seamlessly integrate with other
  projects, leveraging their capabilities and contributing modular tools and
  algorithms.
- **Simplicity and Accessibility:** Maintain a codebase that is simple to
  understand and work on, lowering the barrier to entry for contributors.
- **Reasonable Performance:** Ensure that TIR delivers acceptable performance
  for binary analysis tasks, without sacrificing code simplicity.

### Non-goals

- **Replacing LLVM:** TIR is not intended to replace LLVM in all capacities,
  but rather to complement it by providing specialized tools for binary
  analysis.
- **One-Size-Fits-All Solution:** TIR does not aim to be a one-size-fits-all
  tool for all binary analysis needs, but rather a modular framework that can
  be extended and customized as needed.
