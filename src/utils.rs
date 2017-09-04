pub fn offset_usize(base: usize, offset: isize) -> usize {
    if offset < 0 {
        base.wrapping_sub((-offset) as usize)
    } else {
        base.wrapping_add(offset as usize)
    }
}
