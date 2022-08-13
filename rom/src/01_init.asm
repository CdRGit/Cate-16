.include "io.inc"

.import uart_send_char : far
.import uart_read_char : far
.import uart_setup : far
.import uart_flush : far
.import uart_read_line : far

.import monitor_start : far

.zeropage
.res 192 ; reserve user direct page area

.segment "LOWRAM_0"
.res $4000 ; reserve memory for direct pages and stack
.res $3000 ; reserve memory for user

.segment "FLASH_0"
reset:
    CLC
    XCE
    REP #%00010000 ; small acc and large idx
    LDX #$3FFF
    TXS
    
    JSL uart_setup

    JML monitor_start
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