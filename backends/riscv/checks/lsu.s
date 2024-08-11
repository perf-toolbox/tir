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
# CHECK-NEXT: riscv.lb rd = zero, rs1 = ra, attrs = {offset = <i16: 10>}
# CHECK-NEXT: riscv.lh rd = zero, rs1 = sp, attrs = {offset = <i16: 0>}
# CHECK-NEXT: riscv.lw rd = zero, rs1 = gp, attrs = {offset = <i16: -1>}
# CHECK-NEXT: riscv.ld rd = zero, rs1 = tp, attrs = {offset = <i16: -4>}
# CHECK-NEXT: riscv.lbu rd = t0, rs1 = zero, attrs = {offset = <i16: 0>}
# CHECK-NEXT: riscv.lhu rd = t1, rs1 = zero, attrs = {offset = <i16: 0>}
# CHECK-NEXT: riscv.lwu rd = t2, rs1 = ra, attrs = {offset = <i16: 0>}
# CHECK-NEXT: riscv.sb rs1 = tp, rs2 = ra, attrs = {offset = <i16: 10>}
# CHECK-NEXT: riscv.sh rs1 = t0, rs2 = sp, attrs = {offset = <i16: 0>}
# CHECK-NEXT: riscv.sw rs1 = t1, rs2 = gp, attrs = {offset = <i16: -1>}
# CHECK-NEXT: }
# CHECK-NEXT: }
