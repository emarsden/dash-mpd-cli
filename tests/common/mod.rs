/// Shared code for our test harness.


use fs_err as fs;
use std::env;
use std::path::Path;
use std::process::Command;
use std::io::Cursor;
use anyhow::{Context, Result};


// We tolerate significant differences in final output file size, because as encoder performance
// changes in newer versions of ffmpeg, the resulting file size when reencoding may change
// significantly.
pub fn check_file_size_approx(p: &Path, expected: u64) {
    let meta = fs::metadata(p).unwrap();
    let ratio = meta.len() as f64 / expected as f64;
    assert!(0.9 < ratio && ratio < 1.1, "File sizes: expected {expected}, got {}", meta.len());
}


pub fn ffmpeg_approval(name: &Path) -> bool {
    let ffmpeg = Command::new("ffmpeg")
        .args(["-v", "error",
               "-i", &name.to_string_lossy(),
               "-f", "null", "-"])
        .output()
        .expect("spawning ffmpeg");
    let msg = String::from_utf8_lossy(&ffmpeg.stderr);
    println!("FFMPEG stderr> {msg}");
    msg.len() == 0
}


pub fn generate_minimal_mp4() -> Vec<u8> {
    let config = mp4::Mp4Config {
        major_brand: str::parse("isom").unwrap(),
        minor_version: 512,
        compatible_brands: vec![
            str::parse("isom").unwrap(),
            str::parse("iso2").unwrap(),
            str::parse("avc1").unwrap(),
            str::parse("mp41").unwrap(),
        ],
        timescale: 1000,
    };
    let data = Cursor::new(Vec::<u8>::new());
    let mut writer = mp4::Mp4Writer::write_start(data, &config).unwrap();
    let media_conf = mp4::MediaConfig::AvcConfig(mp4::AvcConfig {
        width: 10,
        height: 10,
        // from https://github.com/ISSOtm/gb-packing-visualizer/blob/1954066537b373f2ddcd5768131bdb5595734a85/src/render.rs#L260
        seq_param_set: vec![
            0, // ???
            0, // avc_profile_indication
            0, // profile_compatibility
            0, // avc_level_indication
        ],
        pic_param_set: vec![],
    });
    let track_conf = mp4::TrackConfig {
        track_type: mp4::TrackType::Video,
        timescale: 1000,
        language: "und".to_string(),
        media_conf,
    };
    writer.add_track(&track_conf).unwrap();
    let sample = mp4::Mp4Sample {
        start_time: 0,
        duration: 2,
        rendering_offset: 0,
        is_sync: true,
        bytes: mp4::Bytes::from(vec![0x0u8; 751]),
    };
    writer.write_sample(1, &sample).unwrap();
    writer.write_end().unwrap();
    writer.into_writer().into_inner()
}


// Useful ffmpeg recipes: https://github.com/videojs/http-streaming/blob/main/docs/creating-content.md
// ffmpeg -y -f lavfi -i testsrc=size=10x10:rate=1 -vf hue=s=0 -t 1 -metadata title=foobles1 tiny.mp4
pub fn generate_minimal_mp4_ffmpeg(metadata: &str) -> Vec<u8> {
    let tmp = env::temp_dir().join("segment.mp4");
    let ffmpeg = Command::new("ffmpeg")
        .args(["-f", "lavfi",
               "-y",  // overwrite output file if it exists
               "-i", "testsrc=size=10x10:rate=1",
               "-vf", "hue=s=0",
               "-t", "1",
               "-metadata", metadata,
               tmp.to_str().unwrap()])
        .output()
        .expect("spawning ffmpeg");
    assert!(ffmpeg.status.success());
    fs::read(tmp).unwrap()
}


// ffprobe -loglevel error -show_entries format_tags -of json tiny.mp4
pub fn ffprobe_metadata_title(mp4: &Path) -> Result<u8> {
    let ffprobe = Command::new("ffprobe")
        .args(["-loglevel", "error",
               "-show_entries", "format_tags",
               "-of", "json",
               mp4.to_str().unwrap()])
        .output()
        .expect("spawning ffmpeg");
    assert!(ffprobe.status.success());
    let parsed = json::parse(&String::from_utf8_lossy(&ffprobe.stdout)).unwrap();
    let title = parsed["format"]["tags"]["title"].as_str().unwrap();
    title.parse().context("parsing title metadata")
}


