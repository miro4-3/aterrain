pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
pub const STRIDE: [usize; 3] = [1, CHUNK_SIZE, CHUNK_SIZE * CHUNK_SIZE];
pub const FACES: [[u8; 12]; 6] = [
    // +Y
    [0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0],
    // -Y
    [0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1],
    // +X
    [1, 0, 0, 1, 1, 0, 1, 1, 1, 1, 0, 1],
    // -X
    [0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0],
    // +Z
    [0, 0, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1],
    // -Z
    [0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0],
];

pub const HEIGHT_SCALE: usize = 100;