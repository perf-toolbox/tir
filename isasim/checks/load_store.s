; RUN: isasim --experiment %S/Inputs/load_store.yaml %s | filecheck %s
; RUN: isasim --experiment %S/Inputs/load_store_faults.yaml %s | filecheck %s

.text
entry:
lw x3, 32(x1)
add x4, x3, x2
sw x4, 48(x1)
lw x5, 48(x1)
ld x6, 64(x1)
addi x6, x6, 1
sd x6, 64(x1)

; CHECK: "x1": 4096,
; CHECK: "x2": 42,
; CHECK: "x3": 36,
; CHECK: "x4": 78,
; CHECK: "x5": 78,
; CHECK: "x6": 40,
