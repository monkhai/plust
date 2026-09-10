use std::{
    sync::{Arc, Mutex},
    thread::{spawn, JoinHandle},
};

use crate::{
    clock::Clock,
    demuxer::Demuxer,
    manifest::Manifest,
    sample_buffer::{Sample, SampleBuffer},
    segment,
};

pub struct Player {
    clock: Clock,
    buffer: Arc<Mutex<SampleBuffer>>,
    loader_handle: Option<JoinHandle<()>>,
    manifest: Arc<Manifest>,
    init_segment: Arc<Vec<u8>>,
}

impl Drop for Player {
    fn drop(&mut self) {
        // Join the loader thread to ensure it completes and resources are freed
        if let Some(handle) = self.loader_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Player {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(base_url: &str) -> Option<Player> {
        let clock = Clock::new();
        let buffer = Arc::new(Mutex::new(SampleBuffer::new()));
        let manifest = Arc::new(Manifest::new(base_url).ok()?);
        let init_path = manifest.init_path(&manifest.representations[0]);
        let init_segment = Arc::new(segment::get_segment(&init_path).ok()?);
        Some(Player {
            buffer,
            clock,
            manifest,
            init_segment,
            loader_handle: None,
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new(base_url: &str) -> Option<Player> {
        let clock = Clock::new();
        let buffer = Arc::new(Mutex::new(SampleBuffer::new()));
        let manifest = Arc::new(Manifest::new(base_url).await.ok()?);
        let init_path = manifest.init_path(&manifest.representations[0]);
        let init_segment = Arc::new(segment::get_segment(&init_path).await.ok()?);
        Some(Player {
            buffer,
            clock,
            manifest,
            init_segment,
            loader_handle: None,
        })
    }
    ///////////////////////////////////////////////////////////////////////////
    /// PUBLIC
    ///////////////////////////////////////////////////////////////////////////

    pub fn get_media_time(&self, now: f64) -> f64 {
        self.clock.get_media_time(now)
    }

    pub fn play(&mut self, now: f64) {
        self.clock.play(now)
    }

    pub fn pause(&mut self, now: f64) {
        self.clock.pause(now)
    }

    pub fn seek(&mut self, media_time: f64, now: f64) {
        self.clock.seek(media_time, now)
    }

    pub fn set_playback_rate(&mut self, rate: f64, now: f64) {
        self.clock.set_playback_rate(rate, now)
    }

    pub fn is_playing(&self) -> bool {
        self.clock.is_playing()
    }

    pub fn pop_next_sample(&mut self) -> Option<Sample> {
        self.buffer.lock().unwrap().pop_next()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn start_loading(&mut self) {
        let buffer = self.buffer.clone();
        let init_segment = self.init_segment.clone();
        let representation = self.manifest.representations[0].clone();
        let manifest = self.manifest.clone();
        let segment_count = manifest.segment_count;
        let start_number = manifest.segment_template.start_number;

        self.loader_handle = Some(spawn(move || {
            let Some(mut demuxer) = Demuxer::from_init_segment(&init_segment) else {
                return;
            };

            let last_segment_i = start_number + segment_count - 1;

            for i in start_number..(start_number + segment_count) {
                let segment_path = manifest.segment_path(&representation, i);
                let Ok(seg) = segment::get_segment(&segment_path) else {
                    continue;
                };
                let Some(seg) = demuxer.demux_segment(&seg) else {
                    continue;
                };

                let mut iter = seg.peekable();

                while let Some(mut sample) = iter.next() {
                    if i == last_segment_i && iter.peek().is_none() {
                        sample.is_last = true
                    }
                    buffer.lock().unwrap().push(sample);
                }
            }
        }));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn start_loading(&mut self) {
        let buffer = self.buffer.clone();
        let init_segment = self.init_segment.clone();
        let representation = self.manifest.representations[0].clone();
        let manifest = self.manifest.clone();
        let segment_count = manifest.segment_count;
        let start_number = manifest.segment_template.start_number;

        wasm_bindgen_futures::spawn_local(async move {
            let Some(mut demuxer) = Demuxer::from_init_segment(&init_segment) else {
                return;
            };

            let last_segment_i = start_number + segment_count - 1;

            for i in start_number..(start_number + segment_count) {
                let segment_path = manifest.segment_path(&representation, i);
                let Ok(seg) = segment::get_segment(&segment_path).await else {
                    continue;
                };
                let Some(seg) = demuxer.demux_segment(&seg) else {
                    continue;
                };

                let mut iter = seg.peekable();

                while let Some(mut sample) = iter.next() {
                    if i == last_segment_i && iter.peek().is_none() {
                        sample.is_last = true
                    }
                    buffer.lock().unwrap().push(sample);
                }
            }
        });
    }
}

///////////////////////////////////////////////////////////////////////////
/// TESTS
///////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn player_loads_and_provides_samples() {
        let mut player = Player::new("http://localhost:8080").expect("Failed to create player");

        player.start_loading();

        // Give the loader thread time to fetch
        thread::sleep(Duration::from_secs(2));

        // Should have samples now
        let sample = player.pop_next_sample();
        assert!(sample.is_some(), "Expected at least one sample");

        let s = sample.unwrap();
        println!("Got sample: pts={}, size={}", s.pts, s.data.len());
    }
}
