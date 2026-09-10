pub struct Clock {
    is_playing: bool,
    wall_time: f64,
    media_time: f64,
    playback_rate: f64,
}

impl Clock {
    pub fn new() -> Clock {
        let clock = Clock {
            is_playing: false,
            wall_time: 0.0,
            media_time: 0.0,
            playback_rate: 1.0,
        };

        clock
    }

    pub fn play(&mut self, now: f64) {
        self.wall_time = now;
        self.is_playing = true;
    }

    pub fn pause(&mut self, now: f64) {
        self.media_time = self.get_media_time(now);
        self.wall_time = now;
        self.is_playing = false;
    }

    pub fn seek(&mut self, media: f64, now: f64) {
        self.media_time = media;
        self.wall_time = now;
    }

    pub fn set_playback_rate(&mut self, rate: f64, now: f64) {
        self.media_time = self.get_media_time(now);
        self.playback_rate = rate;
        self.wall_time = now;
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn get_media_time(&self, now: f64) -> f64 {
        if !self.is_playing {
            return self.media_time;
        }
        self.media_time + (now - self.wall_time) * self.playback_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_paused_at_zero() {
        let clock = Clock::new();
        assert!(!clock.is_playing());
        assert_eq!(clock.get_media_time(0.0), 0.0);
    }

    #[test]
    fn play_advances_time() {
        let mut clock = Clock::new();
        clock.play(0.0);
        assert!(clock.is_playing());
        assert_eq!(clock.get_media_time(5.0), 5.0);
        assert_eq!(clock.get_media_time(10.0), 10.0);
    }

    #[test]
    fn pause_freezes_time() {
        let mut clock = Clock::new();
        clock.play(0.0);
        clock.pause(5.0);
        assert!(!clock.is_playing());
        assert_eq!(clock.get_media_time(5.0), 5.0);
        assert_eq!(clock.get_media_time(100.0), 5.0);
    }

    #[test]
    fn play_pause_play_resumes() {
        let mut clock = Clock::new();
        clock.play(0.0);
        clock.pause(10.0);
        assert_eq!(clock.get_media_time(10.0), 10.0);
        clock.play(20.0);
        assert_eq!(clock.get_media_time(25.0), 15.0);
    }

    #[test]
    fn seek_while_paused() {
        let mut clock = Clock::new();
        clock.seek(30.0, 0.0);
        assert_eq!(clock.get_media_time(0.0), 30.0);
        assert_eq!(clock.get_media_time(100.0), 30.0);
    }

    #[test]
    fn seek_while_playing() {
        let mut clock = Clock::new();
        clock.play(0.0);
        clock.seek(30.0, 5.0);
        assert!(clock.is_playing());
        assert_eq!(clock.get_media_time(5.0), 30.0);
        assert_eq!(clock.get_media_time(10.0), 35.0);
    }

    #[test]
    fn playback_rate_2x() {
        let mut clock = Clock::new();
        clock.set_playback_rate(2.0, 0.0);
        clock.play(0.0);
        assert_eq!(clock.get_media_time(5.0), 10.0);
    }

    #[test]
    fn change_playback_rate_while_playing() {
        let mut clock = Clock::new();
        clock.play(0.0);
        assert_eq!(clock.get_media_time(10.0), 10.0);
        clock.set_playback_rate(2.0, 10.0);
        assert_eq!(clock.get_media_time(10.0), 10.0);
        assert_eq!(clock.get_media_time(15.0), 20.0);
    }
}
