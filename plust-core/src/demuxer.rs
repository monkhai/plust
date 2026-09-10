use crate::{fmp4, sample_buffer::Sample, segment};

pub struct Demuxer {
    timescale: u32,
    cumulative_dts: i64,
}

impl Demuxer {
    pub fn from_init_segment(init_segment: &[u8]) -> Option<Demuxer> {
        let mdhd = fmp4::get_atom(init_segment, &["moov", "trak", "mdia", "mdhd"])?;
        let version = mdhd[8];
        let timescale_offset = if version == 0 { 20 } else { 28 };
        let timescale = fmp4::read_uint(mdhd, timescale_offset, 4);
        Some(Demuxer {
            timescale,
            cumulative_dts: 0,
        })
    }

    pub fn demux_segment<'a>(&'a mut self, segment_data: &'a [u8]) -> Option<SampleIter<'a>> {
        let info = segment::get_segment_info(segment_data).ok()?;
        Some(SampleIter {
            segment_data,
            timescale: self.timescale,
            metadata: info.samples,
            data_cursor: info.mdat_offset,
            sample_index: 0,
            cumulative_dts: &mut self.cumulative_dts,
        })
    }
}

pub struct SampleIter<'a> {
    segment_data: &'a [u8],
    data_cursor: usize,
    sample_index: usize,
    metadata: Vec<segment::SampleMetadata>,
    cumulative_dts: &'a mut i64,
    timescale: u32,
}

impl Iterator for SampleIter<'_> {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sample_index >= self.metadata.len() {
            return None;
        }

        let metadata = &self.metadata[self.sample_index];
        let end = self.data_cursor + metadata.size as usize;
        let data = self.segment_data[self.data_cursor..end].to_vec();

        let pts = *self.cumulative_dts + metadata.cts_offset as i64;
        let dts = *self.cumulative_dts as f64 / self.timescale as f64;
        let pts_secs = pts as f64 / self.timescale as f64;
        let duration = metadata.duration as f64 / self.timescale as f64;
        let is_key = fmp4::sample_has_idr(&data);

        *self.cumulative_dts += metadata.duration as i64;
        self.data_cursor += metadata.size as usize;
        self.sample_index += 1;

        Some(Sample {
            data,
            dts,
            duration,
            pts: pts_secs,
            is_key,
            is_last: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{demuxer::Demuxer, manifest::Manifest, segment};

    #[test]
    fn demux_real_segment() {
        let base = "http://localhost:8080";
        let manifest = Manifest::new(base).unwrap();
        let init_path = manifest.init_path(&manifest.representations[0]);
        let init_segment = segment::get_segment(&init_path).unwrap();

        let mut demuxer =
            Demuxer::from_init_segment(&init_segment).expect("failed to create demuxer");

        let seg1 = segment::get_segment(&format!("{}/video/avc1/seg-1.m4s", base)).unwrap();
        let seg2 = segment::get_segment(&format!("{}/video/avc1/seg-2.m4s", base)).unwrap();

        println!("=== Segment 1 ===");
        let seg1_samples: Vec<_> = demuxer
            .demux_segment(&seg1)
            .expect("demux failed")
            .collect();
        let last = seg1_samples.last().unwrap();
        println!(
            "Seg1: {} samples, last dts={:.3}s",
            seg1_samples.len(),
            last.dts
        );

        println!("\n=== Segment 2 ===");
        let first = demuxer
            .demux_segment(&seg2)
            .expect("demux failed")
            .next()
            .unwrap();
        println!(
            "Seg2 first sample: pts={:.3}s, dts={:.3}s",
            first.pts, first.dts
        );
        assert!(
            first.dts > 10.0,
            "Seg2 should continue from seg1 (dts > 10s)"
        );
    }

    #[test]
    fn test_nal_types() {
        let base = "http://localhost:8080";
        let manifest = Manifest::new(base).unwrap();
        let init_path = manifest.init_path(&manifest.representations[0]);
        let init_segment = segment::get_segment(&init_path).unwrap();

        let mut demuxer =
            Demuxer::from_init_segment(&init_segment).expect("failed to create demuxer");

        let seg1 = segment::get_segment(&format!("{}/video/avc1/seg-1.m4s", base)).unwrap();

        println!("=== First 15 samples NAL analysis ===");
        let mut keyframe_count = 0;
        for (i, sample) in demuxer.demux_segment(&seg1).unwrap().enumerate().take(15) {
            let nal_type = sample.data[4] & 0x1F;
            let is_key = sample.is_key;
            if is_key {
                keyframe_count += 1;
            }
            println!(
                "Sample {:2}: size={:6}, nal_type={:2}, is_key={}, bytes[0..8]={:02x?}",
                i,
                sample.data.len(),
                nal_type,
                is_key,
                &sample.data[..8.min(sample.data.len())]
            );
        }
        println!("\nKeyframes in first 15 samples: {}", keyframe_count);
        assert!(
            keyframe_count <= 2,
            "Too many keyframes - NAL detection is wrong"
        );
    }
}
