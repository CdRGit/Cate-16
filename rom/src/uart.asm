.zeropage
readbuff_ptr: .res 3
readbuff_size: .res 2
readbuff_left_steps: .res 2
writebuff_ptr: .res 3

.segment "LOWRAM_0"
tx_buffer: .res 256
tx_write_ptr: .res 2
tx_read_ptr: .res 2

.segment "FLASH_1"

.include "io.inc"
.macpack longbranch

.a8
.i16

.export d_uart_setup
d_uart_setup:
    LDA UART_LINE_CONTROL
    ORA #$80
    STA UART_LINE_CONTROL
; BRG registers now available
    LDX #$0001
    STX UART_DIV_LATCH
; UART speed set to 115.2k
    ; UART_LINE_CONTROL for 8-N-1
    ; bits 1,0 for word length (11 = 8, 10 = 7, 01 = 7, 00 = 5)
    ; bit 2 for stop bit (with length 8 0 = 1 stop bit)
    ; bit 3 for parity (0 = no parity)
    ; bit 4 odd/even parity (not relevant)
    ; bit 5 parity forced (not relevant)
    ; bit 6 break condition (0 for off)
    ; bit 7 BRG registers visible (0 for off)
    LDA #%0_0_0_0_0_0_11
    STA UART_LINE_CONTROL
    ; UART_FIFO_CONTROL for FIFO buffer
    ; bit 0 enable fifo
    ; bit 1 clear receive buffer
    ; bit 2 clear transmit buffer
    ; bit 3 DMA mode (0 seems fine)
    ; bit 4,5 unused
    ; bit 6,7 for receive fifo trigger level (11 = 14, 10 = 8, 01 = 4, 00 = 1)
    LDA #%11_00_0_1_1_1
    STA UART_FIFO_CONTROL
    ; should now be 8-N-1
    LDX #$0000
    STX tx_read_ptr
    STX tx_write_ptr
    RTS

.export d_uart_read_char
d_uart_read_char:
    @wait_loop:
        LDA UART_LINE_STATUS
        AND #%0000000_1 ; receive data ready
        BEQ @wait_loop
    LDA UART_DATA
    RTS

; [A:23-15][X:15-0] ptr
; null terminated string
; clobbers all registers
.export d_uart_send_string
d_uart_send_string:
    STX writebuff_ptr
    STA writebuff_ptr + 2
    LDY #$0000
@send_loop:
    LDA [writebuff_ptr],Y
    BEQ @done
    PHY
    JSR d_uart_send_char
    PLY
    INY
    BRA @send_loop
@done:
    RTS


; [A:23-16][X:15-0] ptr
; [Y:15-0]
; --
; A = last char read
; X = size left
; Y = length read in
.export d_uart_read_line
d_uart_read_line:
    STX readbuff_ptr
    STA readbuff_ptr + 2
    STY readbuff_size

    LDX #$0000
    STX readbuff_left_steps

    TYX ; y (size) -> x
    LDY #$0000    
    ; Y = index
    ; X = size left
    ; A = data
@read_loop:
    JSR d_uart_read_char
    ; check for special cases
    CMP #$7F ; 'delete' or backspace
    BNE @not_delete
        CPY #$0000
        BEQ @read_loop ; delete at start = do nothing
        DEY
        PHY
        LDY readbuff_left_steps
        BNE :+
        PLY
        BRA :++
    :
        INY
        STY readbuff_left_steps
        PLY
    :
        PHX ; preserve X
        PHY ; preserve Y
        ; clear out previous character on line
        LDA #$08 ; backspace character = go back one
        JSR d_uart_send_char
        LDA #$20 ; space character = clear it out
        PLY
        STA [readbuff_ptr],Y
        PHY
        JSR d_uart_send_char
        LDA #$08 ; backspace character = go back one
        JSR d_uart_send_char
        JSR d_uart_flush
        PLY ; restore Y
        PLX ; restore X
        BRA @read_loop
@not_delete:
    CMP #$1B
    BNE @not_escape
        JSR d_uart_read_char
        CMP #'['
        BNE @read_loop
        JSR d_uart_read_char
        CMP #'A'
        BNE @not_up
        BRA @read_loop
    @not_up:
        CMP #'B'
        BNE @not_down
        BRA @read_loop
    @not_down:
        CMP #'C'
        BNE @not_right
        PHY
        LDY readbuff_left_steps
        BNE :+
        PLY
        BRA @read_loop
    :
        DEY
        STY readbuff_left_steps
        PLY
        INY
        PHY
        PHX
        LDA #$1B
        JSR d_uart_send_char
        LDA #'['
        JSR d_uart_send_char
        LDA #'C'
        JSR d_uart_send_char
        JSR d_uart_flush
        PLX
        PLY
        BRA @read_loop
    @not_right:
        CMP #'D'
        BNE @not_left
        CPY #$0000
        jeq @read_loop
        DEY
        PHY
        LDY readbuff_left_steps
        INY
        STY readbuff_left_steps
        PLY
        PHY
        PHX
        LDA #$1B
        JSR d_uart_send_char
        LDA #'['
        JSR d_uart_send_char
        LDA #'D'
        JSR d_uart_send_char
        JSR d_uart_flush
        PLX
        PLY
        BRL @read_loop
    @not_left:
        BRL @read_loop
@not_escape:
    CMP #$0A
    BEQ @exit ; newline, exit immediately
    ; ok store & echo it
    STA [readbuff_ptr],Y
    INY
    PHY ; Y gets stored on the stack (uart_flush might corrupt it)
    PHX ; X gets stored on the stack (uart_flush might corrupt it)
    PHA ; A gets stored on the stack (uart_flush might corrupt it)
    JSR d_uart_send_char
    JSR d_uart_flush
    PLA ; A gets restored
    PLX ; X gets restored
    PLY ; Y gets restored
    DEX
    jne @read_loop

@exit:
    PHX
    LDX readbuff_left_steps
:
    CPX #$0000
    BEQ :+
    DEX
    INY
    BRA :-
:
    PLX
    PHA
    LDA #$00
    STA [readbuff_ptr],Y
    PLA
    RTS

.export d_uart_send_char
d_uart_send_char:
    PHA
    LDA tx_write_ptr
    CMP tx_read_ptr
    BMI @flush
@continue:
    PLA
    LDX tx_write_ptr
    INC tx_write_ptr
    STA tx_buffer,X
    RTS
@flush:
    JSR d_uart_flush
    BRA @continue

.export d_uart_flush
d_uart_flush:
    LDA tx_read_ptr
    CMP tx_write_ptr
    BEQ @exit
    @wait_tx_empty:
        LDA UART_LINE_STATUS
        AND #%00_1_00000 ; transmit hold register empty
        BEQ @wait_tx_empty
    ; tx is empty, we can now send 16 bytes
    LDY #16
    @send_loop:
        LDX tx_read_ptr
        INC tx_read_ptr
        LDA tx_buffer,X
        STA UART_DATA
        LDA tx_write_ptr
        LDA tx_read_ptr
        CMP tx_write_ptr
        BEQ @exit
        DEY
        BNE @send_loop
    BRA d_uart_flush

@exit:
    RTS

