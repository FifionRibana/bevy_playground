use bevy::prelude::*;
use hexx::*;

use crate::camera::MainCamera;
use crate::hex::HexConfig;
use crate::shared::types::ChunkId;

use super::cache::WorldCache;
use super::components::StreamingConfig;

pub fn request_chunks(
    camera: Query<&Transform, With<MainCamera>>,
    mut cache: ResMut<WorldCache>,
    mut streaming_config: ResMut<StreamingConfig>,
    hex_config: Res<HexConfig>,
    time: Res<Time>,
) {
    let Ok(transform) = camera.single() else {
        return;
    };

    if !streaming_config.is_request_valid(*time) {
        return;
    }

    let position = transform.translation.truncate();

    let center_chunk = ChunkId::from_position(position, &hex_config);
    let mut request_count: u32 = 0;

    for chunk_id in center_chunk.range(streaming_config.view_radius) {
        if !cache.is_loaded(&chunk_id) {
            // info!("Adding chunk to render: {:?}", chunk_id);
            // cache.mark_requested(chunk_id);
            cache.insert_chunk(chunk_id);
            request_count += 1;
        }
    }

    if request_count > 0 {
        info!("{} chunks requested", request_count);
    }
    streaming_config.update_request_time(time.elapsed_secs());
}

pub fn unload_distant_chunks(
    camera: Query<&Transform, With<MainCamera>>,
    mut cache: ResMut<WorldCache>,
    streaming_config: ResMut<StreamingConfig>,
    hex_config: Res<HexConfig>,
) {
    let Ok(transform) = camera.single() else {
        return;
    };

    let position = transform.translation.truncate();
    let center_chunk = ChunkId::from_position(position, &hex_config);
    // info!("Center chunk to unload: {:?}", center_chunk.coord);

    cache.unload_distant(center_chunk, streaming_config.unload_distance);
}
