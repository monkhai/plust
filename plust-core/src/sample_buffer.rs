use std::collections::VecDeque;

pub struct Sample {
    pub data: Vec<u8>,
    pub pts: f64,
    pub dts: f64,
    pub duration: f64,
    pub is_key: bool,
    pub is_last: bool,
}

pub struct SampleBuffer {
    samples: VecDeque<Sample>,
}

impl SampleBuffer {
    pub fn new() -> SampleBuffer {
        SampleBuffer {
            samples: VecDeque::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn push(&mut self, sample: Sample) {
        self.samples.push_back(sample);
    }

    pub fn peek_next(&self) -> Option<&Sample> {
        self.samples.front()
    }

    pub fn pop_next(&mut self) -> Option<Sample> {
        self.samples.pop_front()
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }

    pub fn seek(&mut self, pts: f64) -> bool {
        for (i, sample) in self.samples.iter().enumerate().rev() {
            if sample.is_key && sample.pts <= pts {
                self.samples.drain(0..i);
                return true;
            }
        }

        self.samples.clear();
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sample(dts: f64, pts: f64, is_key: bool) -> Sample {
        Sample {
            pts,
            dts,
            duration: 0.033,
            data: vec![dts as u8],
            is_key,
            is_last: false,
        }
    }

    // Helper: make a non-keyframe sample (dts == pts for simplicity)
    fn make_frame(dts: f64) -> Sample {
        make_sample(dts, dts, false)
    }

    // Helper: make a keyframe
    fn make_keyframe(dts: f64) -> Sample {
        make_sample(dts, dts, true)
    }

    #[test]
    fn new_buffer_is_empty() {
        let buf = SampleBuffer::new();
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
    }

    #[test]
    fn push_increases_len() {
        let mut buf = SampleBuffer::new();
        buf.push(make_frame(0.0));
        assert_eq!(buf.len(), 1);
        buf.push(make_frame(1.0));
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn peek_returns_front_without_removing() {
        let mut buf = SampleBuffer::new();
        buf.push(make_frame(0.0));
        buf.push(make_frame(1.0));

        assert_eq!(buf.peek_next().unwrap().dts, 0.0);
        assert_eq!(buf.len(), 2); // Still 2
        assert_eq!(buf.peek_next().unwrap().dts, 0.0); // Same sample
    }

    #[test]
    fn pop_removes_and_returns_front() {
        let mut buf = SampleBuffer::new();
        buf.push(make_frame(0.0));
        buf.push(make_frame(1.0));
        buf.push(make_frame(2.0));

        assert_eq!(buf.pop_next().unwrap().dts, 0.0);
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.pop_next().unwrap().dts, 1.0);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.pop_next().unwrap().dts, 2.0);
        assert_eq!(buf.len(), 0);
        assert!(buf.pop_next().is_none());
    }

    #[test]
    fn empty_buffer_returns_none() {
        let mut buf = SampleBuffer::new();
        assert!(buf.peek_next().is_none());
        assert!(buf.pop_next().is_none());
    }

    #[test]
    fn clear_empties_buffer() {
        let mut buf = SampleBuffer::new();
        buf.push(make_frame(0.0));
        buf.push(make_frame(1.0));
        assert_eq!(buf.len(), 2);

        buf.clear();
        assert_eq!(buf.len(), 0);
        assert!(buf.peek_next().is_none());
    }

    #[test]
    fn seek_finds_keyframe_before_pts() {
        let mut buf = SampleBuffer::new();
        // Simulate a GOP: I B B P B B P
        buf.push(make_keyframe(0.0)); // I-frame at 0
        buf.push(make_frame(1.0));
        buf.push(make_frame(2.0));
        buf.push(make_keyframe(3.0)); // I-frame at 3
        buf.push(make_frame(4.0));
        buf.push(make_frame(5.0));

        // Seek to pts=4.5 should find keyframe at 3.0
        let found = buf.seek(4.5);
        assert!(found);
        assert_eq!(buf.peek_next().unwrap().dts, 3.0);
        assert!(buf.peek_next().unwrap().is_key);
    }

    #[test]
    fn seek_to_first_keyframe() {
        let mut buf = SampleBuffer::new();
        buf.push(make_keyframe(0.0));
        buf.push(make_frame(1.0));
        buf.push(make_frame(2.0));

        // Seek to pts=1.5 should find keyframe at 0.0
        let found = buf.seek(1.5);
        assert!(found);
        assert_eq!(buf.peek_next().unwrap().dts, 0.0);
        assert_eq!(buf.len(), 3); // Nothing dropped
    }

    #[test]
    fn seek_before_any_keyframe_clears_buffer() {
        let mut buf = SampleBuffer::new();
        buf.push(make_keyframe(5.0)); // Keyframe at 5.0
        buf.push(make_frame(6.0));

        // Seek to pts=2.0, no keyframe before that
        let found = buf.seek(2.0);
        assert!(!found);
        assert!(buf.is_empty());
    }

    #[test]
    fn seek_with_no_keyframes_clears_buffer() {
        let mut buf = SampleBuffer::new();
        buf.push(make_frame(0.0));
        buf.push(make_frame(1.0));
        buf.push(make_frame(2.0));

        let found = buf.seek(1.5);
        assert!(!found);
        assert!(buf.is_empty());
    }

    #[test]
    fn seek_exactly_on_keyframe() {
        let mut buf = SampleBuffer::new();
        buf.push(make_keyframe(0.0));
        buf.push(make_frame(1.0));
        buf.push(make_keyframe(2.0));
        buf.push(make_frame(3.0));

        // Seek to exactly pts=2.0
        let found = buf.seek(2.0);
        assert!(found);
        assert_eq!(buf.peek_next().unwrap().dts, 2.0);
        assert_eq!(buf.len(), 2); // Dropped first 2 samples
    }
}
