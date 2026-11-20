use crate::{
    GLOBAL_CONSTANTS::{ CHUNK_SIZE, CHUNK_VOLUME, FACES, STRIDE },
    renderer::packed_mesh::{ PackedMesh, PackedVertex },
};
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

    fn index_to_pos(i: usize) -> (usize, usize, usize) {
        let x = i % STRIDE[0];
        let z = (i / STRIDE[0]) % STRIDE[1];
        let y = i / STRIDE[2];

        (x, y, z)
    }
    fn get_block(&self, x: usize, y: usize, z: usize) -> u8 {
        self.blocks[Self::index(x, y, z)]
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
    fn face_is_empty(
        blocks: &[u8; CHUNK_VOLUME],
        neighbor: Option<&[u8; CHUNK_VOLUME]>,
        at_boundary: bool,
        self_index: usize,
        neighbor_index: usize
    ) -> bool {
        if at_boundary {
            match neighbor {
                Some(nb) => nb[neighbor_index] == 0,
                None => true,
            }
        } else {
            blocks[self_index] == 0
        }
    }

    pub fn generate_chunk_mesh(&self) -> PackedMesh {
        self.generate_chunk_mesh_with_neighbors([None; 6])
    }

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

                    if
                        Self::face_is_empty(
                            &self.blocks,
                            neighbors[0],
                            y == last,
                            Self::index(x, y + 1, z),
                            Self::index(x, 0, z)
                        )
                    {
                        Self::add_face(&mut vertices, &mut indices, x, y, z, 0, id);
                    }

                    if
                        Self::face_is_empty(
                            &self.blocks,
                            neighbors[1],
                            y == 0,
                            Self::index(x, y.saturating_sub(1), z),
                            Self::index(x, last, z)
                        )
                    {
                        Self::add_face(&mut vertices, &mut indices, x, y, z, 1, id);
                    }

                    if
                        Self::face_is_empty(
                            &self.blocks,
                            neighbors[2],
                            x == last,
                            Self::index(x + 1, y, z),
                            Self::index(0, y, z)
                        )
                    {
                        Self::add_face(&mut vertices, &mut indices, x, y, z, 2, id);
                    }

                    if
                        Self::face_is_empty(
                            &self.blocks,
                            neighbors[3],
                            x == 0,
                            Self::index(x.saturating_sub(1), y, z),
                            Self::index(last, y, z)
                        )
                    {
                        Self::add_face(&mut vertices, &mut indices, x, y, z, 3, id);
                    }

                    if
                        Self::face_is_empty(
                            &self.blocks,
                            neighbors[4],
                            z == last,
                            Self::index(x, y, z + 1),
                            Self::index(x, y, 0)
                        )
                    {
                        Self::add_face(&mut vertices, &mut indices, x, y, z, 4, id);
                    }

                    if
                        Self::face_is_empty(
                            &self.blocks,
                            neighbors[5],
                            z == 0,
                            Self::index(x, y, z.saturating_sub(1)),
                            Self::index(x, y, last)
                        )
                    {
                        Self::add_face(&mut vertices, &mut indices, x, y, z, 5, id);
                    }
                }
            }
        }

        PackedMesh { vertices, indices }
    }
}
