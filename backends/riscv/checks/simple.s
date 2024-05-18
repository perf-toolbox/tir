# RUN: tir-asm %s | filecheck %s

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
# CHECK-NEXT: riscv.add rs1 = t3, rs2 = t1, rd = t2, attrs = {}
# CHECK-NEXT: riscv.sub rs1 = t3, rs2 = t1, rd = t2, attrs = {}
# CHECK-NEXT: riscv.sll rs1 = t3, rs2 = t1, rd = t2, attrs = {}
# CHECK-NEXT: riscv.slt rs1 = t3, rs2 = t1, rd = t2, attrs = {}
# CHECK-NEXT: riscv.sltu rs1 = t3, rs2 = t1, rd = t2, attrs = {}
# CHECK-NEXT: riscv.srl rs1 = t3, rs2 = t1, rd = t2, attrs = {}
# CHECK-NEXT: riscv.sra rs1 = t3, rs2 = t1, rd = t2, attrs = {}
# CHECK-NEXT: riscv.or rs1 = t3, rs2 = t1, rd = t2, attrs = {}
# CHECK-NEXT: riscv.and rs1 = t3, rs2 = t1, rd = t2, attrs = {}
# CHECK-NEXT: }
