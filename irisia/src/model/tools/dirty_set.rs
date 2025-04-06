pub fn bitset_create(size: usize) -> Box<[u8]> {
    vec![0; size.div_ceil(8)].into_boxed_slice()
}

pub fn bitset_mark(data: &mut [u8], position: usize) {
    let index = position / 8;
    let shifts = position % 8;
    let byte = data.get_mut(index).expect("position out of limit");
    *byte |= 1 << shifts;
}

pub fn bitset_union(dst: &mut [u8], src: &[u8]) {
    assert_eq!(dst.len(), src.len());
    for (a, &b) in dst.iter_mut().zip(src.iter()) {
        *a |= b;
    }
}
