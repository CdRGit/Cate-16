# CATE-16 16 bit computer

## Specs
- 508 KiB Low RAM
- 512 KiB Flash ROM
- 2 MiB High RAM

# Memory map
256 * 64 KiB banks (65C816)

## Banks 00-0F
[508 KiB total] 31.75 KiB (32 KiB - 256 B) RAM (unique per bank)

256 B (mirrored) IO

[512 KiB total] 32 KiB flash 'ROM' (unique per bank)

## Banks 10-1F
Larger memory mapped IO: **TODO**

## Banks 20-3F
2 MiB RAM

## Banks 40-FF
Not Allocated Yet