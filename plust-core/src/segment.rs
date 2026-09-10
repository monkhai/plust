use crate::fmp4;

#[derive(Debug, thiserror::Error)]
pub enum SegmentError {
    #[error("Invalid segment")]
    InvalidSegment,
    #[error("No tfhd")]
    NoTfhd,
    #[error("No trun")]
    NoTrun,
}

pub struct SampleMetadata {
    pub size: u32,
    pub duration: u32,
    pub cts_offset: i32,
}

pub struct SegmentInfo {
    pub samples: Vec<SampleMetadata>,
    pub mdat_offset: usize,
}

struct SampleDefaults {
    default_size: u32,
    default_duration: u32,
}

fn get_sample_defaults(segment: &[u8]) -> Result<SampleDefaults, SegmentError> {
    let tfhd = fmp4::get_atom(segment, &["moof", "traf", "tfhd"]).ok_or(SegmentError::NoTfhd)?;

    let flags = fmp4::read_uint(tfhd, 9, 3);

    let has_base_data_offset = (flags & 0x01) != 0;
    let has_sample_description_index = (flags & 0x02) != 0;
    let has_default_sample_duration = (flags & 0x08) != 0;
    let has_default_sample_size = (flags & 0x10) != 0;

    let mut cursor = 16;

    if has_base_data_offset {
        cursor += 8
    }

    if has_sample_description_index {
        cursor += 4
    }

    let mut duration: u32 = 0;

    if has_default_sample_duration {
        duration = fmp4::read_uint(tfhd, cursor, 4);
        cursor += 4
    }

    let mut size: u32 = 0;
    if has_default_sample_size {
        size = fmp4::read_uint(tfhd, cursor, 4);
    }

    Ok(SampleDefaults {
        default_size: size,
        default_duration: duration,
    })
}

pub fn get_segment_info(segment: &[u8]) -> Result<SegmentInfo, SegmentError> {
    let defaults = get_sample_defaults(segment)?;

    let trun = fmp4::get_atom(segment, &["moof", "traf", "trun"]).ok_or(SegmentError::NoTrun)?;

    if trun.len() < 17 {
        return Err(SegmentError::InvalidSegment);
    }

    let flags = fmp4::read_uint(trun, 9, 3);
    let sample_count = fmp4::read_uint(trun, 12, 4);

    let data_offset_present = (flags & 0x01) != 0;
    let first_sample_flags_present = (flags & 0x04) != 0;

    let mut cursor = 16;

    if data_offset_present {
        cursor += 4
    }
    if first_sample_flags_present {
        cursor += 4
    }

    let sample_duration_present = (flags & 0x0100) != 0;
    let sample_size_present = (flags & 0x0200) != 0;
    let sample_flags_present = (flags & 0x400) != 0;
    let sample_cts_present = (flags & 0x800) != 0;

    let mut samples: Vec<SampleMetadata> = Vec::with_capacity(sample_count as usize);

    for _ in 0..sample_count {
        let mut sample_cursor = cursor;

        let mut duration = defaults.default_duration;
        let mut size = defaults.default_size;
        let mut cts_offset: i32 = 0;

        if sample_duration_present {
            duration = fmp4::read_uint(trun, sample_cursor, 4);
            sample_cursor += 4
        }

        if sample_size_present {
            size = fmp4::read_uint(trun, sample_cursor, 4);
            sample_cursor += 4
        }

        if sample_flags_present {
            sample_cursor += 4
        }

        if sample_cts_present {
            cts_offset = fmp4::read_uint(trun, sample_cursor, 4) as i32;
            sample_cursor += 4
        }

        samples.push(SampleMetadata {
            size,
            duration,
            cts_offset,
        });
        cursor = sample_cursor
    }

    let mdat_offset =
        fmp4::get_atom_offset(segment, &["mdat"]).ok_or(SegmentError::InvalidSegment)?;

    Ok(SegmentInfo {
        samples,
        mdat_offset: mdat_offset + 8,
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_segment(path: &str) -> Result<Vec<u8>, SegmentError> {
    let segment = reqwest::blocking::get(path)
        .map_err(|_| SegmentError::InvalidSegment)?
        .bytes()
        .map_err(|_| SegmentError::InvalidSegment)?
        .to_vec();
    Ok(segment)
}

#[cfg(target_arch = "wasm32")]
pub async fn get_segment(path: &str) -> Result<Vec<u8>, SegmentError> {
    let segment = reqwest::get(path)
        .await
        .map_err(|_| SegmentError::InvalidSegment)?
        .bytes()
        .await
        .map_err(|_| SegmentError::InvalidSegment)?
        .to_vec();
    Ok(segment)
}
