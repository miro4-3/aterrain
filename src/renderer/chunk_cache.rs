use std::collections::HashMap;
use miniquad::*;

use super::packed_mesh::{ PackedMesh, GpuPackedMesh };

pub type ChunkPos = (i32, i32, i32);

pub struct ChunkGpuCache {
    chunks: HashMap<ChunkPos, GpuPackedMesh>,
}

impl ChunkGpuCache {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn upload(
        &mut self,
        ctx: &mut dyn RenderingBackend,
        position: ChunkPos,
        mesh: &PackedMesh
    ) {
        if let Some(mesh) = self.chunks.remove(&position) {
            mesh.destroy(ctx);
        }

        if let Some(gpu_mesh) = mesh.upload(ctx) {
            self.chunks.insert(position, gpu_mesh);
        }
    }

    pub fn draw_all(
        &self,
        ctx: &mut dyn RenderingBackend,
        renderer: &super::packed_renderer::PackedRenderer,
        mvp: [[f32; 4]; 4],
        center: ChunkPos,
        render_distance: i32
    ) {
        renderer.begin(ctx);

        for (pos, mesh) in self.chunks.iter() {
            if !within_chunk_distance(*pos, center, render_distance) {
                continue;
            }
            let chunk_pos = [pos.0 as f32, pos.1 as f32, pos.2 as f32, 0.0];
            renderer.draw(ctx, mesh, mvp, chunk_pos);
        }
    }

    pub fn remove(&mut self, ctx: &mut dyn RenderingBackend, position: ChunkPos) {
        if let Some(mesh) = self.chunks.remove(&position) {
            mesh.destroy(ctx);
        }
    }

    pub fn clear(&mut self, ctx: &mut dyn RenderingBackend) {
        for (_, mesh) in self.chunks.drain() {
            mesh.destroy(ctx);
        }
    }
}

pub fn within_chunk_distance(position: ChunkPos, center: ChunkPos, distance: i32) -> bool {
    let dx = (position.0 as f64) - (center.0 as f64);
    let dz = (position.2 as f64) - (center.2 as f64);
    let distance = distance.max(0) as f64;

    // Render distance is horizontal; changing altitude must not hide or unload terrain.
    dx.abs() <= distance && dz.abs() <= distance
}
