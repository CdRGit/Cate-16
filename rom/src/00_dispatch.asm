.segment "FLASH_0"
.import d_reset
.export reset:far
reset:
    JSR d_reset
    RTL

.segment "FLASH_1"
.import d_uart_setup
.export uart_setup:far
uart_setup:
    JSR d_uart_setup
    RTL

.import d_uart_flush
.export uart_flush:far
uart_flush:
    JSR d_uart_flush
    RTL

.import d_uart_send_string
.export uart_send_string:far
uart_send_string:
    JSR d_uart_send_string
    RTL

.import d_uart_send_char
.export uart_send_char:far
uart_send_char:
    JSR d_uart_send_char
    RTL

.import d_uart_read_line
.export uart_read_line:far
uart_read_line:
    JSR d_uart_read_line
    RTL

.import d_uart_read_char
.export uart_read_char:far
uart_read_char:
    JSR d_uart_read_char
    RTL