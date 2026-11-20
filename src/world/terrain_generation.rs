use noise::{ NoiseFn, Perlin };

use crate::{ GLOBAL_CONSTANTS::{ CHUNK_SIZE, HEIGHT_SCALE }, world::chunk::Chunk };

fn chunk_world_bounds(chunk: i32) -> Option<(i32, i32)> {
    let chunk_size = i32::try_from(CHUNK_SIZE).ok()?;
    let first_block = chunk.checked_mul(chunk_size)?;
    let last_block = first_block.checked_add(chunk_size.checked_sub(1)?)?;
    Some((first_block, last_block))
}

// Frequency: smaller = smoother
fn get_noise_phase(cx: i32, cz: i32, perlin: &Perlin, frequency: f64) -> f64 {
    perlin.get([(cx as f64) * frequency, (cz as f64) * frequency]).abs()
}

//generate columns like minecraft but it cubic chunk so ye i aint doing that
pub fn generate_chunk_data(cx: i32, cy: i32, cz: i32, perlin: &Perlin) -> Chunk {
    let mut chunk = Chunk::new();
    let Some((chunk_min_x, _)) = chunk_world_bounds(cx) else {
        return chunk;
    };
    let Some((chunk_min_y, chunk_max_y)) = chunk_world_bounds(cy) else {
        return chunk;
    };
    let Some((chunk_min_z, _)) = chunk_world_bounds(cz) else {
        return chunk;
    };

    if chunk_max_y < 0 || chunk_min_y as i64 > HEIGHT_SCALE as i64 {
        return chunk;
    }

    let chunk_size = CHUNK_SIZE as i64;
    let chunk_min_y = chunk_min_y as i64;

    for x in 0..CHUNK_SIZE {
        let world_x = chunk_min_x + x as i32;
        for z in 0..CHUNK_SIZE {
            let world_z = chunk_min_z + z as i32;
            let height = (get_noise_phase(world_x, world_z, perlin, 0.001) *
                (HEIGHT_SCALE as f64)) as i64;
            let max_y = (height - chunk_min_y + 1).clamp(0, chunk_size) as usize;
            for y in 0..max_y {
                chunk.set_block(x, y, z, 1);
            }
        }
    }
    chunk
}
