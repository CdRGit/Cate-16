.struct UART
    .org $7F10
    DIVISOR_LATCH:
    DATA:
        .byte
    IRQ_ENABLE:
        .byte
    IRQ_STATUS:
    FIFO_CTRL:
        .byte
    LINE_CTRL:
        .byte
    LINE_STATUS:
        .byte
    MODEM_STATUS:
        .byte
    SCRATCH:
        .byte
.endstruct