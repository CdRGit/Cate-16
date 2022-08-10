.include "io.inc"

.import uart_send_char
.import uart_read_char
.import uart_setup
.import uart_flush
.import uart_read_line

.segment "LOWRAM_0"
.res $4000 ; reserve memory for direct pages and stack
.res $3000 ; reserve memory for user
read_buff: ; readline buffer
    .res 257

.segment "FLASH_0"
reset:
    CLC
    XCE
    REP #%00010000 ; small acc and large idx
    LDX #$3FFF
    TXS
uart:
    JSR uart_setup

    LDA #'>'
    JSR uart_send_char
    LDA #' '
    JSR uart_send_char
    JSR uart_flush
    LDA #0
    LDX #read_buff
    LDY #256
    JSR uart_read_line
    LDA #$00
    STA read_buff,Y

    LDA #$0A
    JSR uart_send_char
    LDA #$0A
    JSR uart_send_char
    LDA #'-'
    JSR uart_send_char
    LDA #' '
    JSR uart_send_char
    LDX #$0000
@send_loop:
    LDA read_buff,X
    BEQ @done
    PHX ; preserve X
    JSR uart_send_char
    PLX ; restore X
    INX
    BRA @send_loop

@done:
    LDA #$0A
    JSR uart_send_char
    JSR uart_flush
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