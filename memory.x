MEMORY {
    BOOT2       : ORIGIN = 0x10000000, LENGTH = 0x100
    FLASH       : ORIGIN = 0x10000100, LENGTH = 0x200000 - 0x38e68
    RESERVED    : ORIGIN = 0x101c7298, LENGTH = 0x38d68
    RAM         : ORIGIN = 0x20000000, LENGTH = 256K
    SCRATCH_X   : ORIGIN = 0x20040000, LENGTH = 4K
    SCRATCH_Y   : ORIGIN = 0x20041000, LENGTH = 4K
}

SECTIONS {
    .reserved_firmware : {
        KEEP(*(.reserved_firmware))
    } > RESERVED
}