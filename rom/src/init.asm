.segment "LOWRAM_0"
.res $4000 ; reserve memory for direct pages and stack

.segment "FLASH_0"
reset:
    CLC
    XCE
    REP #%00010000 ; small acc and large idx
.i16
.a8
    LDX #$3FFF
    TXS
    ; stack ready
    LDA #$55
    STA $7F00 ; write into IO register 0 (crashes rn)
    STP
tbd:
    STP

.segment "ROM_VEC"
; native PRI_IRQ
.repeat 8
    .word $EAEA
.endrepeat
; emulation PRI_IRQ
.repeat 8
    .word $EAEA
.endrepeat
; native vectors
.word $0000
.word $0000 ; padding
.word tbd   ; COP
.word tbd   ; BRK
.word tbd   ; ABORT
.word tbd   ; NMI
.word $0000 ; (RESET)
.word tbd   ; IRQ
; emulation vectors
.word $0000
.word $0000 ; padding
.word tbd   ; COP
.word $0000 ; (BRK)
.word tbd   ; ABORT
.word tbd   ; NMI
.word reset ; RESET
.word tbd   ; IRQ