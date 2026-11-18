use macroquad::math::Mat4;
use miniquad::*;

use super::packed_mesh::GpuPackedMesh;

pub struct PackedRenderer {
    pub pipeline: Pipeline,
}

impl PackedRenderer {
    pub fn new(ctx: &mut dyn RenderingBackend, vertex_shader: &str, fragment_shader: &str) -> Self {
        let shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: vertex_shader,
                    fragment: fragment_shader,
                },
                ShaderMeta {
                    images: vec![],
                    uniforms: UniformBlockLayout {
                        uniforms: vec![
                            UniformDesc::new("u_MVP", UniformType::Mat4),
                            UniformDesc::new("u_ChunkPos", UniformType::Float4)
                        ],
                    },
                }
            )
            .unwrap();

        let mut attribute = VertexAttribute::new("a_PackedVertex", VertexFormat::Int1);

        attribute.gl_pass_as_float = false;

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[attribute],
            shader,
            PipelineParams {
                cull_face: CullFace::Back,
                depth_test: Comparison::Greater,
                depth_write: true,
                ..PipelineParams::default()
            }
        );

        Self { pipeline }
    }

    pub fn begin(&self, ctx: &mut dyn RenderingBackend) {
        ctx.apply_pipeline(&self.pipeline);
    }

    pub fn draw(
        &self,
        ctx: &mut dyn RenderingBackend,
        mesh: &GpuPackedMesh,
        mvp: [[f32; 4]; 4],
        chunk_pos: [f32; 4]
    ) {
        ctx.apply_bindings(&mesh.bindings);
        ctx.apply_uniforms(
            UniformsSource::table(
                &(Uniforms {
                    u_MVP: mvp,
                    u_ChunkPos: chunk_pos,
                })
            )
        );
        ctx.draw(0, mesh.index_count, 1);
    }

    pub fn set_mvp(&self, ctx: &mut dyn RenderingBackend, mvp: &Mat4) {
        ctx.apply_uniforms(
            UniformsSource::table(
                &(Uniforms { u_MVP: mvp.to_cols_array_2d(), u_ChunkPos: [0.0; 4] })
            )
        );
    }
}
#[repr(C)]
struct Uniforms {
    u_MVP: [[f32; 4]; 4],
    u_ChunkPos: [f32; 4],
}
