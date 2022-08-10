.zeropage
readbuff_ptr: .res 3
readbuff_size: .res 2

.segment "LOWRAM_0"
tx_buffer: .res 256
tx_write_ptr: .res 2
tx_read_ptr: .res 2

.segment "FLASH_0"

.include "io.inc"

.export uart_setup
.export uart_read_char
.export uart_send_char
.export uart_flush
.export uart_read_line

.a8
.i16

uart_setup:
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

uart_read_char:
    @wait_loop:
        LDA UART_LINE_STATUS
        AND #%0000000_1 ; receive data ready
        BEQ @wait_loop
    LDA UART_DATA
    RTS

; [A:23-16][X:15-0] ptr
; [Y:15-0]
; --
; A = last char read
; X = size left
; Y = length read in
uart_read_line:
    STX readbuff_ptr
    STA readbuff_ptr + 2
    STY readbuff_size
    TYX ; y (size) -> x
    LDY #$0000    
    ; Y = index
    ; X = size left
    ; A = data
@read_loop:
    JSR uart_read_char
    ; check for special cases
    CMP #$7F ; 'delete' or backspace
    BNE @not_delete
        CPY #$0000
        BEQ @read_loop ; delete at start = do nothing
        DEY
        PHY ; preserve Y
        PHX ; preserve X
        ; clear out previous character on line
        LDA #$08 ; backspace character = go back one
        JSR uart_send_char
        LDA #$20 ; space character = clear it out
        JSR uart_send_char
        LDA #$08 ; backspace character = go back one
        JSR uart_send_char
        JSR uart_flush
        PLX ; restore X
        PLY ; restore Y
        BRA @read_loop
@not_delete:
    CMP #$0A
    BEQ @exit ; newline, exit immediately
    ; ok store & echo it
    STA [readbuff_ptr],Y
    INY
    PHY ; Y gets stored on the stack (uart_flush might corrupt it)
    PHX ; X gets stored on the stack (uart_flush might corrupt it)
    PHA ; A gets stored on the stack (uart_flush might corrupt it)
    JSR uart_send_char
    JSR uart_flush
    PLA ; A gets restored
    PLX ; X gets restored
    PLY ; Y gets restored
    DEX
    BNE @read_loop

@exit:
    RTS

uart_send_char:
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
    JSR uart_flush
    BRA @continue

uart_flush:
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
    BRA uart_flush

@exit:
    RTS

