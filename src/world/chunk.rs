use crate::{
    GLOBAL_CONSTANTS::{ CHUNK_SIZE, CHUNK_VOLUME, FACES, STRIDE },
    renderer::packed_mesh::{ PackedMesh, PackedVertex },
};
// SIZE: 16 x 16 x 16
#[derive(Clone, Copy)]
pub struct Chunk {
    pub blocks: [u8; CHUNK_VOLUME],
    pub lod: u8,
    pub is_dirty: bool,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: [0; CHUNK_VOLUME],
            lod: 0,
            is_dirty: true,
        }
    }

    pub fn index(x: usize, y: usize, z: usize) -> usize {
        x * STRIDE[0] + z * STRIDE[1] + y * STRIDE[2]
    }

    fn get_block(&self, x: usize, y: usize, z: usize) -> u8 {
        self.blocks[Self::index(x, y, z)]
    }

    fn is_empty_at(&self, x: isize, y: isize, z: isize) -> bool {
        if
            x < 0 ||
            y < 0 ||
            z < 0 ||
            x >= (CHUNK_SIZE as isize) ||
            y >= (CHUNK_SIZE as isize) ||
            z >= (CHUNK_SIZE as isize)
        {
            return true;
        }

        self.blocks[Self::index(x as usize, y as usize, z as usize)] == 0
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_id: u8) {
        self.blocks[Self::index(x, y, z)] = block_id;
    }

    fn pack_vertex(x: u32, y: u32, z: u32, face: u32, id: u32) -> PackedVertex {
        PackedVertex(
            (x & 0x1f) | ((y & 0x1f) << 5) | ((z & 0x1f) << 10) | ((face & 0x07) << 15) | (id << 18)
        )
    }

    fn add_face(
        vertices: &mut Vec<PackedVertex>,
        indices: &mut Vec<u16>,
        x: usize,
        y: usize,
        z: usize,
        face_id: u8,
        block_id: u8
    ) {
        let base = vertices.len() as u16;

        for corner in FACES[face_id as usize].chunks_exact(3) {
            vertices.push(
                Self::pack_vertex(
                    (x as u32) + (corner[0] as u32),
                    (y as u32) + (corner[1] as u32),
                    (z as u32) + (corner[2] as u32),
                    face_id as u32,
                    block_id as u32
                )
            );
        }

        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    pub fn generate_chunk_mesh(&self) -> PackedMesh {
        self.generate_chunk_mesh_with_neighbors([None; 6])
    }

    /// Generate mesh with cross-chunk face culling.
    ///
    /// `neighbors` maps to FACES order:
    ///   0 = +Y, 1 = -Y, 2 = +X, 3 = -X, 4 = +Z, 5 = -Z
    ///
    /// Neighbor references are resolved once before the meshing loop.
    /// Boundary voxels index directly into the neighbor's block array.
    /// Missing neighbors are treated as air (face is generated).
    pub fn generate_chunk_mesh_with_neighbors(
        &self,
        neighbors: [Option<&[u8; CHUNK_VOLUME]>; 6]
    ) -> PackedMesh {
        let mut vertices: Vec<PackedVertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        let last = CHUNK_SIZE - 1;

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let id = self.get_block(x, y, z);
                    if id == 0 {
                        continue;
                    }

                    // +Y face
                    let empty = if y == last {
                        match neighbors[0] {
                            Some(blocks) => blocks[Self::index(x, 0, z)] == 0,
                            None => true,
                        }
                    } else {
                        self.blocks[Self::index(x, y + 1, z)] == 0
                    };
                    if empty {
                        Self::add_face(&mut vertices, &mut indices, x, y, z, 0, id);
                    }

                    // -Y face
                    let empty = if y == 0 {
                        match neighbors[1] {
                            Some(blocks) => blocks[Self::index(x, last, z)] == 0,
                            None => true,
                        }
                    } else {
                        self.blocks[Self::index(x, y - 1, z)] == 0
                    };
                    if empty {
                        Self::add_face(&mut vertices, &mut indices, x, y, z, 1, id);
                    }

                    // +X face
                    let empty = if x == last {
                        match neighbors[2] {
                            Some(blocks) => blocks[Self::index(0, y, z)] == 0,
                            None => true,
                        }
                    } else {
                        self.blocks[Self::index(x + 1, y, z)] == 0
                    };
                    if empty {
                        Self::add_face(&mut vertices, &mut indices, x, y, z, 2, id);
                    }

                    // -X face
                    let empty = if x == 0 {
                        match neighbors[3] {
                            Some(blocks) => blocks[Self::index(last, y, z)] == 0,
                            None => true,
                        }
                    } else {
                        self.blocks[Self::index(x - 1, y, z)] == 0
                    };
                    if empty {
                        Self::add_face(&mut vertices, &mut indices, x, y, z, 3, id);
                    }

                    // +Z face
                    let empty = if z == last {
                        match neighbors[4] {
                            Some(blocks) => blocks[Self::index(x, y, 0)] == 0,
                            None => true,
                        }
                    } else {
                        self.blocks[Self::index(x, y, z + 1)] == 0
                    };
                    if empty {
                        Self::add_face(&mut vertices, &mut indices, x, y, z, 4, id);
                    }

                    // -Z face
                    let empty = if z == 0 {
                        match neighbors[5] {
                            Some(blocks) => blocks[Self::index(x, y, last)] == 0,
                            None => true,
                        }
                    } else {
                        self.blocks[Self::index(x, y, z - 1)] == 0
                    };
                    if empty {
                        Self::add_face(&mut vertices, &mut indices, x, y, z, 5, id);
                    }
                }
            }
        }

        PackedMesh { vertices, indices }
    }
}
