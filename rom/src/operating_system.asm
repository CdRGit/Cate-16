.export os_entry : far

.import uart_send_string : far
.import uart_flush : far

.a8
.i16

.segment "FLASH_2"

os_entry:
    LDA #^os_welcome_string
    LDX #.loword(os_welcome_string)
    JSL uart_send_string
    JSL uart_flush
    STP

os_welcome_string:
    .byte "\x1B[2J\x1B[1;1H"
    .asciiz "CATE-16 Operating System\n> "