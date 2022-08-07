pub struct Bus {
    low_ram:   [u8; 512 * 1024],
    flash_rom: [u8; 512 * 1024],
    high_ram:  [u8;2048 * 1024],
}