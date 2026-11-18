use macroquad::conf::Conf;
#[allow(unused_mut, unused_imports, unused)]
#[allow(non_snake_case)]
use macroquad::input::{ set_cursor_grab, show_mouse };
use crate::{
    GLOBAL_CONSTANTS::CHUNK_SIZE,
    renderer::chunk_cache::ChunkGpuCache,
    world::chunk::Chunk,
};

mod gameloop;
mod user;
mod world;
mod renderer;
mod GLOBAL_CONSTANTS;

fn config_window() -> Conf {
    let miniquad_conf = miniquad::conf::Conf {
        window_title: "g".into(),
        window_resizable: true,
        ..Default::default()
    };
    Conf {
        miniquad_conf: miniquad_conf,
        default_filter_mode: miniquad::FilterMode::Nearest,
        ..Default::default()
    }
}
#[macroquad::main(config_window)]
async fn main() {
    use macroquad::prelude::*;
    use macroquad::window::get_internal_gl;

    use renderer::{ chunk_cache::ChunkGpuCache, packed_renderer::PackedRenderer };
    // then we load the renderer
    let (renderer, mut cache) = {
        let gl = unsafe { get_internal_gl() };
        let ctx = &mut *gl.quad_context;

        let renderer = PackedRenderer::new(
            ctx,
            include_str!("../assets/shaders/default.vert"),
            include_str!("../assets/shaders/default.frag")
        );
        let mut cache = ChunkGpuCache::new();

        for x in 0..16 {
            for z in 0..16 {
                for y in 0..16 {
                    let chunk = generate_test_world();
                    cache.store_blocks((x, y, z), chunk.blocks);
                }
            }
        }

        for x in 0..16 {
            for z in 0..16 {
                for y in 0..16 {
                    let pos = (x, y, z);
                    let neighbors = cache.get_neighbors(pos);
                    let blocks = cache.get_block_data(pos).copied().expect("failed to cpy");
                    let chunk = Chunk { blocks, lod: 0, is_dirty: true };
                    let mesh = chunk.generate_chunk_mesh_with_neighbors(neighbors);
                    cache.upload(ctx, pos, blocks, &mesh);
                }
            }
        }

        (renderer, cache)
    };
    set_cursor_grab(true);
    show_mouse(false);

    let camera = user::camera::Camera::default();

    let mut gi = GameInformation {
        camera: camera,
        settings: user::setting::CameraSetting::default(),
        renderer: renderer,
        mesh_cache: cache,
    };
    loop {
        gameloop::step(&mut gi);
        next_frame().await;
    }
}

pub struct GameInformation {
    pub camera: user::camera::Camera,
    pub settings: user::setting::CameraSetting,
    pub renderer: renderer::packed_renderer::PackedRenderer,
    pub mesh_cache: ChunkGpuCache,
}

fn generate_test_world() -> Chunk {
    let mut chunk = Chunk::new();
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                if x % 4 == 0 && y % 4 == 0 && z % 4 == 0 {
                    chunk.set_block(x, y, z, 1);
                }
            }
        }
    }
    chunk
}
