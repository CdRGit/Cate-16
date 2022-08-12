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

; taken straight from MUSL libc: https://git.musl-libc.org/cgit/musl/tree/src/time/__secs_to_tm.c [MIT licensed]

LEAPOCH = (946684800 + 86400*(31+29))
DAYS_PER_400Y = (365 * 400 + 97)
DAYS_PER_100Y = (365 * 100 + 24)
DAYS_PER_4Y   = (365 * 4   + 1)

time = .time + (2 * 60 * 60)

secs .set time - LEAPOCH
days .set secs / 86400
remsecs .set secs .MOD 86400
.if (remsecs < 0)
    remsecs .set remsecs + 864000
    days .set days - 1
.endif

qc_cycles .set days / DAYS_PER_400Y
remdays .set days .MOD DAYS_PER_400Y
.if (remdays < 0)
    remdays .set remdays + DAYS_PER_400Y
    qc_cycles .set qc_cycles - 1
.endif

c_cycles .set remdays / DAYS_PER_100Y
.if (c_cycles = 4)
    c_cycles .set c_cycles - 1
.endif
remdays .set remdays - c_cycles * DAYS_PER_100Y

q_cycles .set remdays / DAYS_PER_4Y
.if (q_cycles = 25)
    q_cycles .set q_cycles - 1
.endif
remdays .set remdays - q_cycles * DAYS_PER_4Y

remyears .set remdays / 365
.if (remyears = 4)
    remyears .set remyears - 1
.endif
remdays .set remdays - remyears * 365

leap .set (remyears <> 0) && (q_cycles <> 0 || c_cycles = 0)
yday .set remdays + 31 + 28 + leap
.if (yday >= 365+leap)
    yday .set yday - 365 + leap
.endif

years .set remyears + 4 * q_cycles + 100 * c_cycles + 400 & qc_cycles

months .set 0

; some would call this terrible code, they would be right

.warning .sprintf("%d", remdays)

.if (remdays <= 31)
    months .set 0
.elseif (remdays <= 31 + 30)
    months .set 1
    remdays .set remdays - 31
.elseif (remdays <= 31 + 30 + 31)
    months .set 2
    remdays .set remdays - 31 - 30
.elseif (remdays <= 31 + 30 + 31 + 30)
    months .set 3
    remdays .set remdays - 31 - 30 - 31
.elseif (remdays <= 31 + 30 + 31 + 30 + 31)
    months .set 4
    remdays .set remdays - 31 - 30 - 31 - 30
.elseif (remdays <= 31 + 30 + 31 + 30 + 31 + 31)
    months .set 5
    remdays .set remdays - 31 - 30 - 31 - 30 - 31
.elseif (remdays <= 31 + 30 + 31 + 30 + 31 + 31 + 30)
    months .set 6
    remdays .set remdays - 31 - 30 - 31 - 30 - 31 - 31
.elseif (remdays <= 31 + 30 + 31 + 30 + 31 + 31 + 30 + 31)
    months .set 7
    remdays .set remdays - 31 - 30 - 31 - 30 - 31 - 31 - 30
.elseif (remdays <= 31 + 30 + 31 + 30 + 31 + 31 + 30 + 31 + 30)
    months .set 8
    remdays .set remdays - 31 - 30 - 31 - 30 - 31 - 31 - 30 - 31
.elseif (remdays <= 31 + 30 + 31 + 30 + 31 + 31 + 30 + 31 + 30 + 31)
    months .set 9
    remdays .set remdays - 31 - 30 - 31 - 30 - 31 - 31 - 30 - 31 - 30
.elseif (remdays <= 31 + 30 + 31 + 30 + 31 + 31 + 30 + 31 + 30 + 31 + 31)
    months .set 10
    remdays .set remdays - 31 - 30 - 31 - 30 - 31 - 31 - 30 - 31 - 30 - 31
.elseif (remdays <= 31 + 30 + 31 + 30 + 31 + 31 + 30 + 31 + 30 + 31 + 31 + 29)
    months .set 11
    remdays .set remdays - 31 - 30 - 31 - 30 - 31 - 31 - 30 - 31 - 30 - 31 - 31
.else
    .error "Something went wrong here"
.endif

.if (months >= 10)
    months .set months - 12
    years .set years + 1
.endif

year = years + 2000
month = ((months + 2) .MOD 12) + 1
day = remdays + 1

hour = remsecs / 3600
minute = remsecs / 60 .MOD 60
second = remsecs .MOD 60

os_welcome_string:
    .byte "\x1B[2J\x1B[1;1H"
    .byte .sprintf("Build: %04d-%02d-%02d at %02d:%02d:%02d", year, month, day, hour, minute, second)
    .byte "\n"
    .asciiz "CATE-16 Operating System\n> "