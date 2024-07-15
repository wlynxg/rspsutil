pub fn big_endian_u16(b: &[u8]) -> u16 {
    if b.len() < 2 {
        return 0;
    }

    b[1] as u16 | (b[0] as u16) << 8
}


pub fn big_endian_u32(b: &[u8]) -> u32 {
    if b.len() < 4 {
        return 0;
    }

    b[3] as u32 | (b[2] as u32) << 8 | (b[1] as u32) << 16 | (b[0] as u32) << 24
}

pub fn big_endian_u64(b: &[u8]) -> u64 {
    if b.len() < 8 {
        return 0;
    }

    b[7] as u64 | (b[6] as u64) << 8 | (b[5] as u64) << 16 | (b[4] as u64) << 24 |
        (b[3] as u64) << 32 | (b[2] as u64) << 40 | (b[1] as u64) << 48 | (b[0] as u64) << 56
}

pub fn little_endian_u16(b: &[u8]) -> u16 {
    if b.len() < 2 {
        return 0;
    }
    b[0] as u16 | (b[1] as u16) << 8
}

pub fn little_endian_u32(b: &[u8]) -> u32 {
    if b.len() < 4 {
        return 0;
    }
    b[0] as u32 | (b[1] as u32) << 8 | (b[2] as u32) << 16 | (b[3] as u32) << 24
}

pub fn little_endian_u64(b: &[u8]) -> u64 {
    if b.len() < 8 {
        return 0;
    }
    b[0] as u64 | (b[1] as u64) << 8 | (b[2] as u64) << 16 | (b[3] as u64) << 24 |
        (b[4] as u64) << 32 | (b[5] as u64) << 40 | (b[6] as u64) << 48 | (b[7] as u64) << 56
}

