use bevy::prelude::*;

#[derive(Resource)]
pub struct StreamingConfig {
    pub view_radius: u32,
    pub unload_distance: u32,
    pub request_cooldown: f32,
    pub last_request_time: f32
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            view_radius: 1,
            unload_distance: 2,
            request_cooldown: 0.5,
            last_request_time: 999.,
        }
    }
}

impl StreamingConfig {
    pub fn is_request_valid(&self, time: Time) -> bool {
        // info!("Time: {}, last request: {}, cooldown: {}", time.elapsed_secs(), self.last_request_time, self.request_cooldown);
        time.elapsed_secs() - self.last_request_time < self.request_cooldown
    }

    pub fn update_request_time(&mut self, time: f32) {
        self.last_request_time = time;
    }
}

