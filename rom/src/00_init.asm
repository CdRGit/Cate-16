.include "io.inc"

.import uart_send_char
.import uart_read_char
.import uart_setup
.import uart_flush

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
ram_check:
    ; scan memory
    REP #%00110000 ; large acc, large idx
    LDX #$0004
@loop:
    DEX
    DEX
    TXA
    STA a:$0000,X
    CMP a:$0000,X
    BNE @fail
    CPX #$0000
    BNE @loop
    BRA @success
@fail:
    SEP #%00100000 ; small acc, large idx
    LDA #$FF
    STA $7F00 ; store $FF into IO $00 on failure
    STP
@success:
    SEP #%00100000 ; small acc, large idx
    ; bank 0 has been tested and works correctly
uart_check:
    JSR uart_setup

    LDX #$0000
@loop:
    LDA hello_string,X
    BEQ @done
    INX
    PHX
    JSR uart_send_char
    PLX
    BRA @loop
@done:
    JSR uart_flush
    STP
tbd:
    STP

hello_string:
    .incbin "uart.asm"
    .asciiz "\n"

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