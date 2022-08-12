.export monitor_start : far

.import uart_read_char : far
.import uart_read_line : far
.import uart_send_char : far
.import uart_send_string : far
.import uart_flush : far

.import os_entry : far

.zeropage
ptr_0:
    .res 3
scratch:
    .res 16

.segment "FLASH_1"
.a8
.i16

monitor_start:
    LDA #^welcome_msg
    LDX #.loword(welcome_msg)
    JSR uart_send_string
    JSR uart_flush
@command_loop:
    JSR prompt
    JSR execute_command
    BRA @command_loop

execute_command:
    JSR uart_read_char
    PHA
    JSR uart_send_char
    JSR uart_flush
    PLA
    CMP #'R'
    BNE @not_read
    JMP read
@not_read:
    CMP #'H'
    BNE @not_halt
    JMP halt
@not_halt:
    CMP #'O'
    BNE @not_operating_sytem
    JMP os_entry
@not_operating_sytem:
    JMP error

halt:
    LDA #$0A
    JSR uart_send_char
    JSR uart_flush
    STP

read:
    JSR read_ptr_0

    LDY #$0000
    @read_loop:
        LDA #':'
        JSR uart_send_char
        LDA [ptr_0],Y
        PHY
        JSR write_hex_8
        JSR uart_flush
        JSR uart_read_char
        PLY
        INY
        CMP #$0A
        BNE @read_loop ; while the character is not '\n' we loop
    LDA #$0A
    JSR uart_send_char

    RTS
@error:
    JMP error

write_hex_8:
    XBA
    LDA #$00
    XBA ; clear out B
    PHA
    AND #$F0
    LSR
    LSR
    LSR
    LSR
    TAX
    LDA hex_table,X
    JSR uart_send_char
    PLA
    AND #$0F
    TAX
    LDA hex_table,X
    JSR uart_send_char
    RTS
error:
    LDA #^error_msg
    LDX #.loword(error_msg)
    JSR uart_send_string
    JSR uart_flush
    RTS

read_ptr_0:
    LDY #$0003 ; counter
    @ptr_loop:
        PHY
        JSR read_hex_8
        PLY
        STA ptr_0-1,Y
        DEY
        BNE @ptr_loop
    RTS


read_hex_8:
    ; X = offset
    ; increments X by 2 before returning
    ; value in A
    JSR read_hex_4
    ASL
    ASL
    ASL
    ASL
    STA scratch
    JSR read_hex_4
    ORA scratch
    RTS

read_hex_4:
    JSR uart_read_char
    PHA
    JSR uart_send_char
    JSR uart_flush
    PLA
    CMP #':' ; one after '9'
    BMI @dec_digit
    ; letter, lower -> upper
    AND #$5F ; should remove the difference between upper and lower
    SEC
    SBC #'7' ; '7' is 10 below 'A'
    RTS
@dec_digit:
    SEC
    SBC #'0' ; '0' - '0' = 0
    RTS

prompt:
    LDA #'$'
    JSR uart_send_char
    LDA #' '
    JSR uart_send_char
    JSR uart_flush
    RTS

error_msg:
    .asciiz " !ERROR!\n"

welcome_msg:
    .asciiz "CATE-16 ROM Monitor\n"

hex_table:
    .byte "0123456789ABCDEF"