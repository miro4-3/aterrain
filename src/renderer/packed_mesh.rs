use bytemuck::{ Pod, Zeroable };
use miniquad::*;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PackedVertex(pub u32);

// CPU-side mesh
pub struct PackedMesh {
    pub vertices: Vec<PackedVertex>,
    pub indices: Vec<u16>,
}

impl PackedMesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn upload(
        &self,
        ctx: &mut dyn RenderingBackend
    ) -> Option<GpuPackedMesh> {
        if self.vertices.is_empty() || self.indices.is_empty() {
            return None;
        }

        Some(GpuPackedMesh::new(ctx, &self.vertices, &self.indices))
    }
}

// GPU-side mesh
pub struct GpuPackedMesh {
    pub bindings: Bindings,
    pub index_count: i32,
}

impl GpuPackedMesh {
    pub fn new(
        ctx: &mut dyn RenderingBackend,
        vertices: &[PackedVertex],
        indices: &[u16],
    ) -> Self {
        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(vertices)
        );

        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(indices)
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![],
        };

        Self {
            bindings,
            index_count: indices.len() as i32,
        }
    }

    pub fn destroy(self, ctx: &mut dyn RenderingBackend) {
        for buffer in self.bindings.vertex_buffers {
            ctx.delete_buffer(buffer);
        }

        ctx.delete_buffer(self.bindings.index_buffer);
    }
}
