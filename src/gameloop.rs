use macroquad::color::{ SKYBLUE };
use macroquad::camera::{ Camera, set_camera, set_default_camera };
use macroquad::time::draw_fps;
use macroquad::window::{ clear_background, get_internal_gl };

use crate::GameInformation;

pub fn step(gi: &mut GameInformation) {
    let dt = macroquad::time::get_frame_time() as f32;
    gi.camera.update(&gi.settings, dt);
    set_camera(&gi.camera.camera_3d());
    clear_background(SKYBLUE);
    let player_chunk = gi.camera.get_current_chunk();
    let render_distance = gi.settings.render_distance as i32;
    let (ready_meshes, removed_chunks) = gi.chunk_generation.update(
        player_chunk,
        render_distance,
        &mut gi.chunk_manager,
    );
    let mut gl = unsafe { get_internal_gl() };
    gl.flush();

    let ctx = &mut *gl.quad_context;
    for position in removed_chunks {
        gi.mesh_cache.remove(ctx, position);
    }
    for (position, mesh) in ready_meshes {
        gi.mesh_cache.upload(ctx, position, &mesh);
    }

    let mvp = gi.camera.camera_3d().matrix().to_cols_array_2d();

    ctx.begin_default_pass(miniquad::PassAction::Clear {
        color: Some((0.5, 0.7, 1.0, 1.0)),
        depth: Some(0.0),
        stencil: None,
    });
    gi.renderer.begin(ctx);
    gi.mesh_cache
        .draw_all(ctx, &gi.renderer, mvp, player_chunk, render_distance);
    ctx.end_render_pass();

    set_default_camera();
    draw_fps();
}
