use std::collections::HashMap;
use miniquad::*;

use super::packed_mesh::{ PackedMesh, GpuPackedMesh };
use crate::GLOBAL_CONSTANTS::CHUNK_VOLUME;

pub type ChunkPos = (i32, i32, i32);

pub struct ChunkGpuCache {
    chunks: HashMap<ChunkPos, GpuPackedMesh>,
    blocks: HashMap<ChunkPos, [u8; CHUNK_VOLUME]>,
}

impl ChunkGpuCache {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            blocks: HashMap::new(),
        }
    }

    pub fn upload(
        &mut self,
        ctx: &mut dyn RenderingBackend,
        position: ChunkPos,
        blocks: [u8; CHUNK_VOLUME],
        mesh: &PackedMesh
    ) {
        // Destroy old GPU mesh if any (keep block data intact for neighbor lookups)
        if let Some(mesh) = self.chunks.remove(&position) {
            mesh.destroy(ctx);
        }

        // Store/update block data
        self.blocks.insert(position, blocks);

        // Upload new GPU mesh
        if let Some(gpu_mesh) = mesh.upload(ctx) {
            self.chunks.insert(position, gpu_mesh);
        }
    }

    /// Stores raw block data for a chunk without GPU upload.
    /// Used to populate neighbor data before mesh generation.
    pub fn store_blocks(&mut self, position: ChunkPos, blocks: [u8; CHUNK_VOLUME]) {
        self.blocks.insert(position, blocks);
    }

    /// Returns the raw block data for a chunk, if present.
    pub fn get_block_data(&self, position: ChunkPos) -> Option<&[u8; CHUNK_VOLUME]> {
        self.blocks.get(&position)
    }

    /// Resolves all 6 neighboring chunk block references in one call.
    ///
    /// Order matches FACES: [+Y, -Y, +X, -X, +Z, -Z].
    /// Exactly 6 HashMap lookups per chunk (not per voxel).
    pub fn get_neighbors(&self, position: ChunkPos) -> [Option<&[u8; CHUNK_VOLUME]>; 6] {
        [
            self.blocks.get(&(position.0, position.1 + 1, position.2)),     // +Y
            self.blocks.get(&(position.0, position.1 - 1, position.2)),     // -Y
            self.blocks.get(&(position.0 + 1, position.1, position.2)),     // +X
            self.blocks.get(&(position.0 - 1, position.1, position.2)),     // -X
            self.blocks.get(&(position.0, position.1, position.2 + 1)),     // +Z
            self.blocks.get(&(position.0, position.1, position.2 - 1)),     // -Z
        ]
    }

    pub fn draw_all(
        &self,
        ctx: &mut dyn RenderingBackend,
        renderer: &super::packed_renderer::PackedRenderer,
        mvp: [[f32; 4]; 4]
    ) {
        renderer.begin(ctx);

        for (pos, mesh) in self.chunks.iter() {
            let chunk_pos = [pos.0 as f32, pos.1 as f32, pos.2 as f32, 0.0];
            renderer.draw(ctx, mesh, mvp, chunk_pos);
        }
    }

    pub fn remove(&mut self, ctx: &mut dyn RenderingBackend, position: ChunkPos) {
        if let Some(mesh) = self.chunks.remove(&position) {
            mesh.destroy(ctx);
        }
        self.blocks.remove(&position);
    }

    pub fn clear(&mut self, ctx: &mut dyn RenderingBackend) {
        for (_, mesh) in self.chunks.drain() {
            mesh.destroy(ctx);
        }
        self.blocks.clear();
    }
}
