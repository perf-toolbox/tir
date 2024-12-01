# RUN: tir asm %s | filecheck %s

.text
example:
    add x28, x6, x7
    sub x28, x6, x7
    sll x28, x6, x7
    slt x28, x6, x7
    sltu x28, x6, x7
    srl x28, x6, x7
    sra x28, x6, x7
    or x28, x6, x7
    and x28, x6, x7
    addi x28, x6, 42
    slli x28, x6, 9
    slti x28, x6, 8
    sltiu x28, x6, 25
    srli x28, x6, -3
    srai x28, x6, 0
    ori x28, x6, 1
    andi x28, x6, 1

# CHECK: module {
# CHECK-NEXT: target.section ".text" {
# CHECK-NEXT: ^example:
# CHECK-NEXT: riscv.add rd = x28, rs1 = x6, rs2 = x7, attrs = {}
# CHECK-NEXT: riscv.sub rd = x28, rs1 = x6, rs2 = x7, attrs = {}
# CHECK-NEXT: riscv.sll rd = x28, rs1 = x6, rs2 = x7, attrs = {}
# CHECK-NEXT: riscv.slt rd = x28, rs1 = x6, rs2 = x7, attrs = {}
# CHECK-NEXT: riscv.sltu rd = x28, rs1 = x6, rs2 = x7, attrs = {}
# CHECK-NEXT: riscv.srl rd = x28, rs1 = x6, rs2 = x7, attrs = {}
# CHECK-NEXT: riscv.sra rd = x28, rs1 = x6, rs2 = x7, attrs = {}
# CHECK-NEXT: riscv.or rd = x28, rs1 = x6, rs2 = x7, attrs = {}
# CHECK-NEXT: riscv.and rd = x28, rs1 = x6, rs2 = x7, attrs = {}
# CHECK-NEXT: }
# CHECK-NEXT: }
