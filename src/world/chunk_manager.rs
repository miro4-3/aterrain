use std::collections::HashMap;

use crate::{
    GLOBAL_CONSTANTS::CHUNK_VOLUME,
    renderer::chunk_cache::ChunkPos,
    renderer::packed_mesh::PackedMesh,
    world::chunk::Chunk,
};

pub struct ChunkManager {
    chunks: HashMap<ChunkPos, [u8; CHUNK_VOLUME]>,
}

pub struct ChunkMeshInput {
    pub blocks: [u8; CHUNK_VOLUME],
    pub neighbors: [Option<[u8; CHUNK_VOLUME]>; 6],
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn insert(&mut self, position: ChunkPos, blocks: [u8; CHUNK_VOLUME]) {
        self.chunks.insert(position, blocks);
    }

    pub fn get(&self, position: ChunkPos) -> Option<&[u8; CHUNK_VOLUME]> {
        self.chunks.get(&position)
    }

    pub fn contains(&self, position: ChunkPos) -> bool {
        self.chunks.contains_key(&position)
    }

    pub fn retain(&mut self, mut keep: impl FnMut(ChunkPos) -> bool) -> Vec<ChunkPos> {
        let mut removed = Vec::new();
        self.chunks.retain(|&position, _| {
            let should_keep = keep(position);
            if !should_keep {
                removed.push(position);
            }
            should_keep
        });
        removed
    }

    pub fn mesh_input_at(
        &self,
        position: ChunkPos,
    ) -> Option<ChunkMeshInput> {
        let blocks = *self.chunks.get(&position)?;
        let neighbors = Self::neighbor_positions(position)
            .map(|neighbor| self.chunks.get(&neighbor).copied());

        Some(ChunkMeshInput { blocks, neighbors })
    }

    pub fn neighbor_positions(position: ChunkPos) -> [ChunkPos; 6] {
        [
            (position.0, position.1 + 1, position.2),
            (position.0, position.1 - 1, position.2),
            (position.0 + 1, position.1, position.2),
            (position.0 - 1, position.1, position.2),
            (position.0, position.1, position.2 + 1),
            (position.0, position.1, position.2 - 1),
        ]
    }

    pub fn get_neighbors(&self, position: ChunkPos) -> [Option<&[u8; CHUNK_VOLUME]>; 6] {
        Self::neighbor_positions(position).map(|neighbor| self.chunks.get(&neighbor))
    }

    pub fn generate_mesh_at(&self, position: ChunkPos) -> Option<PackedMesh> {
        let blocks = self.chunks.get(&position)?;
        let neighbors = self.get_neighbors(position);
        let chunk = Chunk {
            blocks: *blocks,
            lod: 0,
            is_dirty: true,
        };
        Some(chunk.generate_chunk_mesh_with_neighbors(neighbors))
    }
}
