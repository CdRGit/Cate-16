# CATE-16 16 bit computer

## Specs
- 508 KiB Low RAM
- 256 B of MMIO
- 512 KiB Flash ROM
- 2 MiB High RAM

# ROM Monitor
Currently the CATE-16 runs a simple ROM monitor with two commands:
- `Rxxxxxx` which reads consecutive bytes from 24 bit address `xxxxxx` (enter after the command, any other key to keep reading)
- `H` which halts the computer immediately

# Memory map
256 * 64 KiB banks (65C816)

## Banks 00-0F
- [508 KiB total] 31.75 KiB (32 KiB - 256 B) RAM (unique per bank)
- 256 B (mirrored) IO
- [512 KiB total] 32 KiB flash 'ROM' (unique per bank)

## Banks 10-1F
Larger memory mapped IO: **TODO**

## Banks 20-3F
2 MiB RAM

## Banks 40-FF
Not Allocated Yet

# Emulator
The CATE-16 emulator is written in Rust