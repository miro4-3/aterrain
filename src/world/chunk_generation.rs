use std::{
    cmp::Ordering,
    collections::{HashSet, VecDeque},
    sync::{Arc, mpsc},
    thread,
};

use noise::Perlin;

use crate::{
    GLOBAL_CONSTANTS::{CHUNK_SIZE, CHUNK_VOLUME, HEIGHT_SCALE},
    renderer::chunk_cache::{ChunkPos, within_chunk_distance},
    renderer::packed_mesh::PackedMesh,
    world::{
        chunk::Chunk,
        chunk_manager::ChunkManager,
        multi_threading::ThreadPool,
        terrain_generation::generate_chunk_data,
    },
};

const CLEANUP_MARGIN: i32 = 2;

enum ChunkWorkResult {
    Generated {
        position: ChunkPos,
        blocks: [u8; CHUNK_VOLUME],
    },
    Meshed {
        position: ChunkPos,
        mesh: PackedMesh,
        neighbor_presence: [bool; 6],
    },
}

pub struct ChunkGeneration {
    workers: ThreadPool,
    noise: Arc<Perlin>,
    result_sender: mpsc::Sender<ChunkWorkResult>,
    result_receiver: mpsc::Receiver<ChunkWorkResult>,
    pending_chunks: VecDeque<ChunkPos>,
    generating_chunks: HashSet<ChunkPos>,
    mesh_candidates: HashSet<ChunkPos>,
    meshed_chunks: HashSet<ChunkPos>,
    meshing_chunks: HashSet<ChunkPos>,
    desired_chunks: HashSet<ChunkPos>,
    interest: Option<(i32, i32, i32)>,
    in_flight: usize,
    max_in_flight: usize,
}

impl ChunkGeneration {
    pub fn new(seed: u32) -> Self {
        let worker_count = thread
            ::available_parallelism()
            .map(|count| count.get().saturating_sub(1).max(1))
            .unwrap_or(1);
        let max_in_flight = worker_count.saturating_mul(2).max(1);
        let (result_sender, result_receiver) = mpsc::channel();

        Self {
            workers: ThreadPool::new(worker_count),
            noise: Arc::new(Perlin::new(seed)),
            result_sender,
            result_receiver,
            pending_chunks: VecDeque::new(),
            generating_chunks: HashSet::new(),
            mesh_candidates: HashSet::new(),
            meshed_chunks: HashSet::new(),
            meshing_chunks: HashSet::new(),
            desired_chunks: HashSet::new(),
            interest: None,
            in_flight: 0,
            max_in_flight,
        }
    }

    pub fn update(
        &mut self,
        center: ChunkPos,
        render_distance: i32,
        chunks: &mut ChunkManager,
    ) -> (Vec<(ChunkPos, PackedMesh)>, Vec<ChunkPos>) {
        let render_distance = render_distance.max(0);
        let removed_chunks = self.update_interest(center, render_distance, chunks);
        let mut ready_meshes = Vec::new();

        while let Ok(result) = self.result_receiver.try_recv() {
            self.in_flight -= 1;

            match result {
                ChunkWorkResult::Generated { position, blocks } => {
                    self.generating_chunks.remove(&position);
                    if self.desired_chunks.contains(&position) {
                        chunks.insert(position, blocks);
                        self.add_mesh_candidates(position);
                    }
                }
                ChunkWorkResult::Meshed {
                    position,
                    mesh,
                    neighbor_presence,
                } => {
                    self.meshing_chunks.remove(&position);
                    if self.desired_chunks.contains(&position)
                        && chunks.contains(position)
                        && self.neighbors_are_ready(position, chunks)
                        && neighbor_presence == Self::neighbor_presence(position, chunks)
                    {
                        self.meshed_chunks.insert(position);
                        ready_meshes.push((position, mesh));
                    } else {
                        self.meshed_chunks.remove(&position);
                        if self.is_visible(position) && chunks.contains(position) {
                            self.mesh_candidates.insert(position);
                        }
                    }
                }
            }
        }

        self.schedule_meshes(chunks);
        self.schedule_chunks();

        (ready_meshes, removed_chunks)
    }

    fn update_interest(
        &mut self,
        center: ChunkPos,
        render_distance: i32,
        chunks: &mut ChunkManager,
    ) -> Vec<ChunkPos> {
        if self.interest == Some((center.0, center.2, render_distance)) {
            return Vec::new();
        }

        self.interest = Some((center.0, center.2, render_distance));
        let cleanup_distance = render_distance.saturating_add(CLEANUP_MARGIN);
        self.desired_chunks = Self::build_desired_chunks(center, cleanup_distance);
        let removed = chunks.retain(|position| self.desired_chunks.contains(&position));

        self.mesh_candidates
            .retain(|position| self.desired_chunks.contains(position));
        self.meshed_chunks
            .retain(|position| self.desired_chunks.contains(position));

        let mut cache_removals: HashSet<_> = removed.iter().copied().collect();
        for position in removed {
            for neighbor in ChunkManager::neighbor_positions(position) {
                if self.desired_chunks.contains(&neighbor) && chunks.contains(neighbor) {
                    self.meshed_chunks.remove(&neighbor);
                    self.mesh_candidates.remove(&neighbor);
                    cache_removals.insert(neighbor);
                }
            }
        }

        let mut missing: Vec<_> = self
            .desired_chunks
            .iter()
            .copied()
            .filter(|position| {
                !chunks.contains(*position) && !self.generating_chunks.contains(position)
            })
            .collect();
        missing.sort_by(|a, b| {
            Self::distance_squared(*a, center)
                .partial_cmp(&Self::distance_squared(*b, center))
                .unwrap_or(Ordering::Equal)
        });
        self.pending_chunks = missing.into();

        for &position in &self.desired_chunks {
            if self.is_visible(position)
                && chunks.contains(position)
                && !self.meshed_chunks.contains(&position)
                && !self.meshing_chunks.contains(&position)
            {
                self.mesh_candidates.insert(position);
            }
        }

        cache_removals.into_iter().collect()
    }

    fn build_desired_chunks(center: ChunkPos, radius: i32) -> HashSet<ChunkPos> {
        let mut desired = HashSet::new();
        let min_x = center.0.saturating_sub(radius);
        let max_x = center.0.saturating_add(radius);
        let min_z = center.2.saturating_sub(radius);
        let max_z = center.2.saturating_add(radius);
        let chunk_height = (HEIGHT_SCALE / CHUNK_SIZE) as i32 + 1;

        for x in min_x..=max_x {
            for z in min_z..=max_z {
                for y in 0..chunk_height {
                    let position = (x, y, z);
                    if within_chunk_distance(position, center, radius) {
                        desired.insert(position);
                    }
                }
            }
        }

        desired
    }

    fn schedule_chunks(&mut self) {
        while self.in_flight < self.max_in_flight {
            let Some(position) = self.pending_chunks.pop_front() else {
                break;
            };
            if !self.desired_chunks.contains(&position) {
                continue;
            }

            let noise = Arc::clone(&self.noise);
            let result_sender = self.result_sender.clone();

            self.workers.execute(move || {
                let chunk = generate_chunk_data(position.0, position.1, position.2, &noise);
                let _ = result_sender.send(ChunkWorkResult::Generated {
                    position,
                    blocks: chunk.blocks,
                });
            });

            self.generating_chunks.insert(position);
            self.in_flight += 1;
        }
    }

    fn schedule_meshes(&mut self, chunks: &ChunkManager) {
        let candidates = std::mem::take(&mut self.mesh_candidates);

        for position in candidates {
            if !self.is_visible(position)
                || !chunks.contains(position)
                || self.meshed_chunks.contains(&position)
                || self.meshing_chunks.contains(&position)
            {
                continue;
            }

            if self.in_flight >= self.max_in_flight {
                self.mesh_candidates.insert(position);
                continue;
            }

            if !self.neighbors_are_ready(position, chunks) {
                self.mesh_candidates.insert(position);
                continue;
            }

            let Some(mesh_input) = chunks.mesh_input_at(position) else {
                continue;
            };
            let neighbor_presence = Self::neighbor_presence(position, chunks);
            self.meshing_chunks.insert(position);
            let result_sender = self.result_sender.clone();

            self.workers.execute(move || {
                let chunk = Chunk {
                    blocks: mesh_input.blocks,
                    lod: 0,
                    is_dirty: true,
                };
                let neighbors = std::array::from_fn(|index| {
                    mesh_input.neighbors[index].as_ref()
                });
                let mesh = chunk.generate_chunk_mesh_with_neighbors(neighbors);
                let _ = result_sender.send(ChunkWorkResult::Meshed {
                    position,
                    mesh,
                    neighbor_presence,
                });
            });

            self.in_flight += 1;
        }
    }

    fn neighbors_are_ready(&self, position: ChunkPos, chunks: &ChunkManager) -> bool {
        ChunkManager::neighbor_positions(position)
            .into_iter()
            .all(|neighbor| !self.desired_chunks.contains(&neighbor) || chunks.contains(neighbor))
    }

    fn neighbor_presence(position: ChunkPos, chunks: &ChunkManager) -> [bool; 6] {
        ChunkManager::neighbor_positions(position).map(|neighbor| chunks.contains(neighbor))
    }

    fn add_mesh_candidates(&mut self, position: ChunkPos) {
        if self.desired_chunks.contains(&position) {
            self.mesh_candidates.insert(position);
        }

        for neighbor in ChunkManager::neighbor_positions(position) {
            if self.desired_chunks.contains(&neighbor) {
                self.mesh_candidates.insert(neighbor);
            }
        }
    }

    fn is_visible(&self, position: ChunkPos) -> bool {
        let Some((center_x, center_z, render_distance)) = self.interest else {
            return false;
        };
        within_chunk_distance(position, (center_x, 0, center_z), render_distance)
    }

    fn distance_squared(position: ChunkPos, center: ChunkPos) -> f64 {
        let dx = position.0 as f64 - center.0 as f64;
        let dz = position.2 as f64 - center.2 as f64;
        dx * dx + dz * dz
    }
}
