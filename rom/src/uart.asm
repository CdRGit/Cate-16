.segment "FLASH_0"

.include "io.inc"

.export uart_setup
.export uart_read_char
.export uart_send_char

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
    RTS

uart_read_char:
    @wait_loop:
        LDA UART_LINE_STATUS
        AND #%0000000_1 ; receive data ready
        BEQ @wait_loop
    LDA UART_DATA
    RTS

uart_send_char:
    PHA
    @wait_loop:
        LDA UART_LINE_STATUS
        AND #%00_1_00000 ; transmit hold register empty
        BEQ @wait_loop
    PLA
    STA UART_DATA
    RTS