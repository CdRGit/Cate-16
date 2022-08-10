.export monitor_start

.import uart_read_line
.import uart_send_char
.import uart_send_string
.import uart_flush

.zeropage
ptr_0:
    .res 3
scratch:
    .res 16

.segment "LOWRAM_0"
read_buff: ; readline buffer
    .res 257

.segment "FLASH_0"
.a8
.i16

monitor_start:
@command_loop:
    JSR prompt
    LDA #$00
    LDX #read_buff
    LDY #$256
    JSR uart_read_line
    JSR execute_command
    BRA @command_loop

execute_command:
    LDA read_buff ; load first character
    CMP #'R'
    BNE @not_read
    JMP read
@not_read:
    JMP error

read:
    LDX #$0001 ; offset of 1
    LDY #$0003 ; counter
    @ptr_loop:
        JSR read_hex_8
        STA ptr_0-1,Y
        DEY
        BNE @ptr_loop

    LDA read_buff,X
    BNE @error ; make sure we've reached the end of the buffer

    LDA #':'
    JSR uart_send_char
    LDA [ptr_0]
    JSR write_hex_8
    LDA #$0A
    JSR uart_send_char
    JSR uart_flush

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
    LDX #error_msg
    JSR uart_send_string
    JSR uart_flush
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
    ; X = offset
    ; increments X by 1 before returning
    ; value in A
    LDA read_buff,X ; char in X
    ; assuming valid hex digit
    INX
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

hex_table:
    .byte "0123456789ABCDEF"