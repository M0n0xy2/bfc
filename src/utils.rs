pub fn offset_u8(base: u8, offset: i8) -> u8 {
    if offset < 0 {
        base.wrapping_sub((-offset) as u8)
    } else {
        base.wrapping_add(offset as u8)
    }
}

pub fn offset_usize(base: usize, offset: isize) -> usize {
    if offset < 0 {
        base.wrapping_sub((-offset) as usize)
    } else {
        base.wrapping_add(offset as usize)
    }
}
