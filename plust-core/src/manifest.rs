use roxmltree::Document;
use serde::Serialize;

fn parse_iso8601_duration(s: &str) -> Option<f64> {
    let s = s.strip_prefix("PT")?;
    let mut total_seconds = 0.0;
    let mut num_str = String::new();

    for c in s.chars() {
        match c {
            'H' => {
                total_seconds += num_str.parse::<f64>().ok()? * 3600.0;
                num_str.clear();
            }
            'M' => {
                total_seconds += num_str.parse::<f64>().ok()? * 60.0;
                num_str.clear();
            }
            'S' => {
                total_seconds += num_str.parse::<f64>().ok()?;
                num_str.clear();
            }
            _ => num_str.push(c),
        }
    }
    Some(total_seconds)
}

#[derive(Serialize, Clone)]
struct Mpd {
    media_presentation_duration: String,
}

#[derive(Serialize, Clone)]
pub struct Representation {
    id: String,
    bandwidth: i32,
    codecs: String,
    height: String,
    width: String,
}

#[derive(Serialize, Clone)]
pub struct SegmentTemplate {
    pub initialization: String,
    pub start_number: i32,
    pub media: String,
    pub timescale: i32,
    pub duration: String,
}

#[derive(Serialize, Clone)]
pub struct Manifest {
    mpd: Mpd,
    pub base_url: String,
    pub representations: Vec<Representation>,
    pub segment_template: SegmentTemplate,
    pub segment_count: i32,
}

fn get_segment_template(node: roxmltree::Node) -> SegmentTemplate {
    let initialization = node.attribute("initialization").unwrap_or("").to_string();
    let start_number = node
        .attribute("startNumber")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1);

    let timescale = node
        .attribute("timescale")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1);

    let media = node.attribute("media").unwrap_or("").to_string();
    let duration = node.attribute("duration").unwrap_or("").to_string();

    return SegmentTemplate {
        initialization,
        start_number,
        media,
        timescale,
        duration,
    };
}

fn get_representations(
    doc: roxmltree::Document,
) -> Result<Vec<Representation>, Box<dyn std::error::Error>> {
    let mut representations = Vec::new();
    for node in doc
        .descendants()
        .filter(|node| node.has_tag_name("Representation"))
    {
        let rep = Representation {
            id: node.attribute("id").ok_or("Missing id")?.to_string(),
            bandwidth: node
                .attribute("bandwidth")
                .ok_or("Missing bandwidth")?
                .parse()?,
            codecs: node
                .attribute("codecs")
                .ok_or("Missing codecs")?
                .to_string(),
            width: node.attribute("width").ok_or("Missing width")?.to_string(),
            height: node
                .attribute("height")
                .ok_or("Missing height")?
                .to_string(),
        };

        representations.push(rep);
    }
    Ok(representations)
}

impl Manifest {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(base_url: &str) -> Result<Manifest, Box<dyn std::error::Error>> {
        let url = format!("{}/stream.mpd", base_url);
        let xml = reqwest::blocking::get(&url)?.text()?;
        Self::parse(base_url, &xml)
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new(base_url: &str) -> Result<Manifest, Box<dyn std::error::Error>> {
        let url = format!("{}/stream.mpd", base_url);
        let xml = reqwest::get(&url).await?.text().await?;
        Self::parse(base_url, &xml)
    }

    pub fn parse(base_url: &str, xml: &str) -> Result<Manifest, Box<dyn std::error::Error>> {
        let doc = Document::parse(&xml)?;

        let mpd_node = doc
            .descendants()
            .find(|n| n.has_tag_name("MPD"))
            .ok_or("No MPD element")
            .unwrap();

        let media_presentation_duration = mpd_node
            .attribute("mediaPresentationDuration")
            .ok_or("Missing duration")?
            .to_string();

        let mpd = Mpd {
            media_presentation_duration: media_presentation_duration,
        };

        let segment_template_node = doc
            .descendants()
            .find(|n| n.has_tag_name("SegmentTemplate"))
            .ok_or("No SegmentTemplate")
            .unwrap();

        let segment_template = get_segment_template(segment_template_node);
        let representations = get_representations(doc).unwrap();

        let media_presentation_duration = parse_iso8601_duration(&mpd.media_presentation_duration)
            .ok_or("Invalid duration format")?;
        let segment_duration_ticks = segment_template.duration.parse::<f64>().unwrap();
        let segment_duration_secs = segment_duration_ticks / segment_template.timescale as f64;
        let segment_count = (media_presentation_duration / segment_duration_secs).ceil() as i32;
        let manifest = Manifest {
            mpd,
            base_url: base_url.to_string(),
            representations,
            segment_template,
            segment_count,
        };

        Ok(manifest)
    }

    pub fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        Ok(json)
    }

    pub fn init_path(&self, rep: &Representation) -> String {
        let path = self
            .segment_template
            .initialization
            .replace("$RepresentationID$", &rep.id);
        format!("{}/{}", self.base_url, path)
    }

    pub fn segment_path(&self, representation: &Representation, number: i32) -> String {
        let path = self
            .segment_template
            .media
            .replace("$RepresentationID$", &representation.id)
            .replace("$Number$", &number.to_string());

        format!("{}/{}", self.base_url, path)
    }
}
