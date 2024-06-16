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

# CHECK: module {
# CHECK-NEXT: target.section ".text" {
# CHECK-NEXT: ^example:
# CHECK-NEXT: riscv.add rd = t3, rs1 = t1, rs2 = t2, attrs = {}
# CHECK-NEXT: riscv.sub rd = t3, rs1 = t1, rs2 = t2, attrs = {}
# CHECK-NEXT: riscv.sll rd = t3, rs1 = t1, rs2 = t2, attrs = {}
# CHECK-NEXT: riscv.slt rd = t3, rs1 = t1, rs2 = t2, attrs = {}
# CHECK-NEXT: riscv.sltu rd = t3, rs1 = t1, rs2 = t2, attrs = {}
# CHECK-NEXT: riscv.srl rd = t3, rs1 = t1, rs2 = t2, attrs = {}
# CHECK-NEXT: riscv.sra rd = t3, rs1 = t1, rs2 = t2, attrs = {}
# CHECK-NEXT: riscv.or rd = t3, rs1 = t1, rs2 = t2, attrs = {}
# CHECK-NEXT: riscv.and rd = t3, rs1 = t1, rs2 = t2, attrs = {}
# CHECK-NEXT: }
# CHECK-NEXT: }
