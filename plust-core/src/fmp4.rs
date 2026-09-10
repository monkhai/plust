use std::collections::HashMap;
use std::ops::Range;
use std::sync::LazyLock;

pub struct CodecConfig {
    pub nal_length_size: u8,
    pub width: u32,
    pub height: u32,
    pub sps: Vec<u8>,
    pub pps: Vec<u8>,
    pub avcc_data: Vec<u8>, // The raw avcC payload for WebCodecs
}

pub static BOX_CHILD_OFFSETS: LazyLock<HashMap<&'static str, usize>> = LazyLock::new(|| {
    HashMap::from([
        ("moov", 8),
        ("trak", 8),
        ("mdia", 8),
        ("minf", 8),
        ("stbl", 8),
        ("dinf", 8),
        ("moof", 8),
        ("traf", 8),
        ("mvex", 8),
        ("stsd", 16),
        ("dref", 16),
        ("avc1", 86),
        ("hvc1", 86),
        ("mp4a", 36),
    ])
});

pub fn get_header_data(data: &[u8], offset: usize) -> Option<(u32, String)> {
    if offset + 8 > data.len() {
        return None;
    }

    let size = u32::from_be_bytes(data[offset..offset + 4].try_into().ok()?);
    let box_type = String::from_utf8_lossy(&data[offset + 4..offset + 8]).to_string();

    Some((size, box_type))
}

pub fn read_uint(data: &[u8], offset: usize, bytes: usize) -> u32 {
    let mut value: u32 = 0;
    for i in 0..bytes {
        let shift = (bytes - 1 - i) * 8;
        value |= u32::from(data[offset + i]) << shift;
    }
    value
}

pub fn get_atom<'a>(data: &'a [u8], path: &[&str]) -> Option<&'a [u8]> {
    get_atom_range(data, path).map(|range| &data[range])
}

pub fn get_atom_range(data: &[u8], path: &[&str]) -> Option<Range<usize>> {
    if path.is_empty() {
        return None;
    }

    let mut offset = 0;
    let mut path_index = 0;

    while offset + 8 <= data.len() {
        let (size, box_type) = get_header_data(data, offset)?;
        if size == 0 {
            break;
        }

        if box_type == path[path_index] {
            if path_index == path.len() - 1 {
                return Some(offset..offset + size as usize);
            }
            let child_offset = BOX_CHILD_OFFSETS
                .get(box_type.as_str())
                .copied()
                .unwrap_or(8);

            offset += child_offset;
            path_index += 1;
        } else {
            offset += size as usize;
        }
    }

    None
}

pub fn get_atom_offset(data: &[u8], path: &[&str]) -> Option<usize> {
    get_atom_range(data, path).map(|range| range.start)
}

pub fn get_codec_config(data: &[u8]) -> Option<CodecConfig> {
    let Some((width, height)) = get_dims(&data) else {
        return None;
    };
    let path = [
        "moov", "trak", "mdia", "minf", "stbl", "stsd", "avc1", "avcC",
    ];
    let Some(avc_c) = get_atom(&data, &path) else {
        return None;
    };
    let Some((sps, pps)) = get_parameter_sets(&avc_c) else {
        return None;
    };
    let nal_length_size = get_nal_length_size(&avc_c);

    // avcC atom includes 8-byte box header (size + 'avcC'), skip it to get payload
    let avcc_data = if avc_c.len() > 8 {
        avc_c[8..].to_vec()
    } else {
        vec![]
    };

    Some(CodecConfig {
        sps,
        pps,
        width,
        height,
        nal_length_size,
        avcc_data,
    })
}

// is the sample a keyframe?
pub fn sample_has_idr(data: &[u8]) -> bool {
    let mut offset = 0;
    // cycle through all the NAL units
    // and read the header types until
    // we find an idr (means keyframe) or return
    while offset + 4 < data.len() {
        let nal_len = read_uint(data, offset, 4) as usize;
        let nal_type = data[offset + 4] & 0x1F;
        if nal_type == 5 {
            return true;
        }
        offset += 4 + nal_len;
    }
    false
}

fn get_parameter_sets(avc_c: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let sps = get_sps(&avc_c);
    let pps = get_pps(&avc_c, sps.len());

    Some((sps, pps))
}

fn get_nal_length_size(data: &[u8]) -> u8 {
    let length_size_minus_one = data[12] & 0x03; // Mask to get lower 2 bits
    let nal_length_size = length_size_minus_one + 1; // Add 1 to get actual size
    nal_length_size
}

fn get_sps(data: &[u8]) -> Vec<u8> {
    let sps_len_offset = 14;
    let sps_len_size = 2;
    let sps_len = read_uint(data, sps_len_offset, sps_len_size) as usize; // offset 14, cast to usize
    let sps_data_offset: usize = sps_len_offset + sps_len_size;
    data[sps_data_offset..sps_data_offset + sps_len].to_vec()
}

fn get_pps(data: &[u8], sps_len: usize) -> Vec<u8> {
    let pps_len_offset = sps_len + 17;
    let pps_len_size = 2;
    let pps_len = read_uint(data, pps_len_offset, pps_len_size) as usize;
    let pps_data_offset: usize = pps_len_offset + pps_len_size;

    data[pps_data_offset..pps_data_offset + pps_len].to_vec()
}

fn get_dims(data: &[u8]) -> Option<(u32, u32)> {
    let path = ["moov", "trak", "mdia", "minf", "stbl", "stsd", "avc1"];
    let Some(avc_a) = get_atom(&data, &path) else {
        return None;
    };
    let width = read_uint(avc_a, 32, 2);
    let height = read_uint(avc_a, 34, 2);
    return Some((width, height));
}

#[cfg(test)]
mod tests {
    use crate::{
        fmp4::{get_atom, get_dims, get_pps, get_sps},
        manifest::Manifest,
        segment,
    };

    fn get_init_data() -> Vec<u8> {
        let url = "http://localhost:8080";
        let Ok(man) = Manifest::new(url) else {
            panic!("Failed to load manifest");
        };

        let init_path = man.init_path(&man.representations[0]);
        let Ok(data) = segment::get_segment(&init_path) else {
            panic!("Failed to load init segment");
        };
        return data;
    }

    fn get_avc_c() -> Vec<u8> {
        let data = get_init_data();
        let path = [
            "moov", "trak", "mdia", "minf", "stbl", "stsd", "avc1", "avcC",
        ];
        let Some(avc_c) = get_atom(&data, &path) else {
            panic!("failed to get avcC");
        };
        avc_c.to_vec()
    }

    #[test]
    fn test_get_dims() {
        let data = get_init_data();
        let Some((width, height)) = get_dims(&data) else {
            panic!("failed to get dims");
        };

        println!("width={}, height={}", width, height);
    }

    #[test]
    fn test_get_sps() {
        let data = get_avc_c();
        let sps = get_sps(&data);

        println!("size of {}", sps.len())
    }

    #[test]
    fn test_get_pps() {
        let data = get_avc_c();
        let sps = get_sps(&data);
        let pps = get_pps(&data, sps.len());

        println!("size of {}", pps.len())
    }
}
