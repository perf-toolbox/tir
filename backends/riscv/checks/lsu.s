# RUN: tir asm %s | filecheck %s

.text
entry:
lb x0, 10(x1)
lh x0, 0(x2)
lw x0, -1(x3)
ld x0, -4(x4)
lbu x5, 0(x0)
lhu x6, 0(x0)
lwu x7, 0(x1)
sb x1, 10(x4)
sh x2, 0(x5)
sw x3, -1(x6)

# CHECK: module {
# CHECK-NEXT: target.section ".text" {
# CHECK-NEXT: ^entry:
# CHECK-NEXT: riscv.lb rd = x0, rs1 = x1, attrs = {offset = <i16: 10>}
# CHECK-NEXT: riscv.lh rd = x0, rs1 = x2, attrs = {offset = <i16: 0>}
# CHECK-NEXT: riscv.lw rd = x0, rs1 = x3, attrs = {offset = <i16: -1>}
# CHECK-NEXT: riscv.ld rd = x0, rs1 = x4, attrs = {offset = <i16: -4>}
# CHECK-NEXT: riscv.lbu rd = x5, rs1 = x0, attrs = {offset = <i16: 0>}
# CHECK-NEXT: riscv.lhu rd = x6, rs1 = x0, attrs = {offset = <i16: 0>}
# CHECK-NEXT: riscv.lwu rd = x7, rs1 = x1, attrs = {offset = <i16: 0>}
# CHECK-NEXT: riscv.sb rs1 = x4, rs2 = x1, attrs = {offset = <i16: 10>}
# CHECK-NEXT: riscv.sh rs1 = x5, rs2 = x2, attrs = {offset = <i16: 0>}
# CHECK-NEXT: riscv.sw rs1 = x6, rs2 = x3, attrs = {offset = <i16: -1>}
# CHECK-NEXT: }
# CHECK-NEXT: }
