use macroquad::{ conf::Conf, window::next_frame };
#[allow(unused_mut, unused_imports, unused)]
#[allow(nonstandard_style)]
use macroquad::input::{ set_cursor_grab, show_mouse };
use macroquad::window::get_internal_gl;
use miniquad::RenderingBackend;
use crate::{
    renderer::{ chunk_cache::ChunkGpuCache, packed_renderer::PackedRenderer },
    world::{
        chunk_generation::ChunkGeneration,
        chunk_manager::ChunkManager,
    },
};

mod gameloop;
mod user;
mod world;
mod renderer;
mod GLOBAL_CONSTANTS;

fn init_game() -> (PackedRenderer, ChunkGeneration) {
    let gl = unsafe { get_internal_gl() };
    let ctx = &mut *gl.quad_context;

    let renderer = load_renderer(ctx);
    let chunk_generation = ChunkGeneration::new(0);

    (renderer, chunk_generation)
}

pub struct GameInformation {
    pub camera: user::camera::Camera,
    pub settings: user::setting::CameraSetting,
    pub renderer: PackedRenderer,
    pub mesh_cache: ChunkGpuCache,
    pub chunk_manager: ChunkManager,
    pub chunk_generation: ChunkGeneration,
}

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
fn load_renderer(ctx: &mut dyn RenderingBackend) -> PackedRenderer {
    PackedRenderer::new(
        ctx,
        include_str!("../assets/shaders/default.vert"),
        include_str!("../assets/shaders/default.frag")
    )
}

#[macroquad::main(config_window)]
async fn main() {
    let (renderer, chunk_generation) = init_game();
    set_cursor_grab(true);
    show_mouse(false);

    let camera = user::camera::Camera::default();

    let mut gi = GameInformation {
        camera,
        settings: user::setting::CameraSetting::default(),
        renderer,
        mesh_cache: ChunkGpuCache::new(),
        chunk_manager: ChunkManager::new(),
        chunk_generation,
    };
    loop {
        gameloop::step(&mut gi);
        next_frame().await;
    }
}
