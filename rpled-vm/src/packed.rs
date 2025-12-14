use bytemuck::Pod;

#[repr(C, packed)]
#[derive(Pod)]
struct PackedPair<T: Pod, U: Pod>(T, U);



impl Pod for PackedPair<u16, u16> {}