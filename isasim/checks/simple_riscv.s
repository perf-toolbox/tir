; RUN: isasim --experiment %S/Inputs/simple_riscv.yaml %s | filecheck %s

; CHECK: "x0": 0,
; CHECK: "x1": 42,
; CHECK: "x2": 31,
; CHECK: "x3": 73,
; CHECK: "x4": 11,
; CHECK: "x5": 10,
; CHECK: "x6": 63,
; CHECK: "x7": 42,

.text
entry:
add x0, x0, x0
add x3, x1, x2
sub x4, x1, x2
and x5, x1, x2
or x6, x1, x2
add x7, x1, x0
