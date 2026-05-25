use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use zip::ZipArchive;

use crate::chart::{Chart, Note};

pub struct SongGroup {
    pub title: String,
    pub artist: String,
    pub source: SongSource,
    pub difficulty_count: usize,
}

pub enum SongSource {
    Osz(PathBuf),
    Dir(PathBuf),
}

pub struct DiffInfo {
    pub name: String,
    pub content: String,
    pub artist: String,
    pub title: String,
    pub source: SongSource,
    pub audio_filename: String,
}

pub struct LoadResult {
    pub chart: Chart,
    pub audio_path: PathBuf,
    pub duration: f64,
}

pub fn scan_song_groups() -> Vec<SongGroup> {
    let mut groups = Vec::new();
    let songs_dir = Path::new("songs");
    if !songs_dir.exists() {
        let _ = fs::create_dir_all(songs_dir);
        return groups;
    }
    let ok = fs::read_dir(songs_dir);
    if ok.is_err() { return groups; }
    for entry in ok.unwrap() {
        let entry = match entry { Ok(e) => e, _ => continue };
        let path = entry.path();

        if path.is_dir() {
            let osu_files: Vec<_> = match fs::read_dir(&path) {
                Ok(d) => d.filter_map(|e| e.ok()).filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("osu")).collect(),
                _ => continue,
            };
            if osu_files.is_empty() { continue; }
            if let Some((title, artist)) = probe_first_meta_dir(&osu_files) {
                groups.push(SongGroup { title, artist, source: SongSource::Dir(path), difficulty_count: osu_files.len() });
            }
        } else if path.extension().and_then(|s| s.to_str()) == Some("osz") {
            if let Some((title, artist, count)) = probe_osz(&path) {
                groups.push(SongGroup { title, artist, source: SongSource::Osz(path), difficulty_count: count });
            }
        }
    }
    groups.sort_by(|a, b| a.title.cmp(&b.title));
    groups
}

fn probe_first_meta_dir(osu_files: &[fs::DirEntry]) -> Option<(String, String)> {
    for entry in osu_files {
        let content = fs::read_to_string(entry.path()).ok()?;
        let content = strip_bom(&content);
        if let Some((t, a, _, _)) = parse_osu_meta(&content) {
            return Some((t, a));
        }
    }
    None
}

fn probe_osz(path: &Path) -> Option<(String, String, usize)> {
    let file = fs::File::open(path).ok()?;
    let mut archive = ZipArchive::new(BufReader::new(file)).ok()?;
    let mut count = 0;
    let mut meta: Option<(String, String)> = None;
    for i in 0..archive.len() {
        let mut zf = match archive.by_index(i) { Ok(z) => z, _ => continue };
        if !zf.name().ends_with(".osu") { continue; }
        count += 1;
        if meta.is_some() { continue; }
        let mut content = String::new();
        if zf.read_to_string(&mut content).is_err() { continue; }
        let content = strip_bom(&content);
        if let Some((t, a, _, _)) = parse_osu_meta(&content) {
            meta = Some((t, a));
        }
    }
    meta.map(|(t, a)| (t, a, count))
}

fn strip_bom(s: &str) -> String {
    s.trim_start_matches('\u{feff}').to_string()
}

pub fn load_difficulties(source: &SongSource) -> Vec<DiffInfo> {
    match source {
        SongSource::Dir(dir) => load_difficulties_dir(dir),
        SongSource::Osz(path) => load_difficulties_osz(path),
    }
}

fn load_difficulties_dir(dir: &Path) -> Vec<DiffInfo> {
    let read = match fs::read_dir(dir) { Ok(d) => d, _ => return vec![] };
    let mut out = Vec::new();
    for entry in read {
        let entry = match entry { Ok(e) => e, _ => continue };
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("osu") { continue; }
        let raw = match fs::read_to_string(&path) { Ok(c) => c, _ => continue };
        let content = strip_bom(&raw);
        if let Some((title, artist, _creator, version)) = parse_osu_meta(&content) {
            let audio_filename = parse_audio_filename(&content).unwrap_or_default();
            out.push(DiffInfo { name: version, content, artist, title, source: SongSource::Dir(dir.to_path_buf()), audio_filename });
        }
    }
    out
}

fn load_difficulties_osz(path: &Path) -> Vec<DiffInfo> {
    let file = match fs::File::open(path) { Ok(f) => f, _ => return vec![] };
    let mut archive = match ZipArchive::new(BufReader::new(file)) { Ok(a) => a, _ => return vec![] };
    let mut out = Vec::new();
    for i in 0..archive.len() {
        let mut zf = match archive.by_index(i) { Ok(z) => z, _ => continue };
        if !zf.name().ends_with(".osu") { continue; }
        let mut raw = String::new();
        if zf.read_to_string(&mut raw).is_err() { continue; }
        let content = strip_bom(&raw);
        if let Some((title, artist, _creator, version)) = parse_osu_meta(&content) {
            let audio_filename = parse_audio_filename(&content).unwrap_or_default();
            out.push(DiffInfo { name: version, content, artist, title, source: SongSource::Osz(path.to_path_buf()), audio_filename });
        }
    }
    out
}

fn parse_audio_filename(content: &str) -> Option<String> {
    let mut section = String::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len()-1].to_string();
            continue;
        }
        if section == "General" {
            if let Some((k, v)) = line.split_once(':') {
                if k.trim() == "AudioFilename" {
                    return Some(v.trim().to_string());
                }
            }
        }
    }
    None
}

pub fn load_chart(diff: &DiffInfo) -> Option<LoadResult> {
    let (chart, duration) = parse_osu_full(&diff.content)?;

    let audio_path = match &diff.source {
        SongSource::Dir(dir) => {
            let p = dir.join(&diff.audio_filename);
            if p.exists() { p } else {
                let read = fs::read_dir(dir).ok()?;
                let found = read.filter_map(|e| e.ok()).find(|e| {
                    let p = e.path();
                    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                    ext == "mp3" || ext == "ogg" || ext == "wav"
                });
                found?.path()
            }
        }
        SongSource::Osz(osz_path) => {
            extract_audio(osz_path, &diff.audio_filename, &diff.name)?
        }
    };

    Some(LoadResult { chart, audio_path, duration })
}

fn extract_audio(osz_path: &Path, audio_filename: &str, diff_name: &str) -> Option<PathBuf> {
    let file = fs::File::open(osz_path).ok()?;
    let mut archive = ZipArchive::new(BufReader::new(file)).ok()?;

    let mut audio_bytes: Option<Vec<u8>> = None;
    let mut audio_name: String = String::new();

    // try exact match first, then any audio file
    for i in 0..archive.len() {
        let mut zf = archive.by_index(i).ok()?;
        let name = zf.name().to_string();
        let ext = Path::new(&name).extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        if !["mp3", "ogg", "wav"].contains(&ext.as_str()) { continue; }

        if name == audio_filename || name.ends_with(&audio_filename) {
            let mut buf = Vec::new();
            zf.read_to_end(&mut buf).ok()?;
            audio_bytes = Some(buf);
            audio_name = name;
            break;
        }
        if audio_bytes.is_none() {
            let mut buf = Vec::new();
            if zf.read_to_end(&mut buf).is_ok() {
                audio_bytes = Some(buf);
                audio_name = name;
            }
        }
    }

    let bytes = audio_bytes?;
    let temp_dir = std::env::temp_dir().join("o2jam_songs");
    fs::create_dir_all(&temp_dir).ok()?;

    let mut hasher = DefaultHasher::new();
    osz_path.hash(&mut hasher);
    diff_name.hash(&mut hasher);
    let hash = hasher.finish();
    let ext = Path::new(&audio_name).extension().and_then(|s| s.to_str()).unwrap_or("mp3");
    let out = temp_dir.join(format!("{:016x}.{}", hash, ext));
    if !out.exists() {
        fs::write(&out, &bytes).ok()?;
    }
    Some(out)
}

fn parse_osu_meta(content: &str) -> Option<(String, String, String, String)> {
    let mut title = String::new();
    let mut artist = String::new();
    let mut creator = String::new();
    let mut version = String::new();
    let mut section = String::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len()-1].to_string();
            continue;
        }
        if line.is_empty() || line.starts_with("//") { continue; }
        match section.as_str() {
            "Metadata" => {
                if let Some((k, v)) = line.split_once(':') {
                    let v = v.trim();
                    match k.trim() {
                        "Title" => title = v.to_string(),
                        "Artist" => artist = v.to_string(),
                        "Creator" => creator = v.to_string(),
                        "Version" => version = v.to_string(),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    if !title.is_empty() && !artist.is_empty() {
        Some((title, artist, creator, version))
    } else {
        None
    }
}

fn parse_osu_full(content: &str) -> Option<(Chart, f64)> {
    let mut mode: Option<i32> = None;
    let mut keys: usize = 7;
    let mut bpm: f64 = 120.0;
    let mut title = String::new();
    let mut artist = String::new();
    let mut notes: Vec<Note> = Vec::new();

    let mut section = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len()-1].to_string();
            continue;
        }
        if line.is_empty() || line.starts_with("//") { continue; }

        match section.as_str() {
            "General" => {
                if let Some((k, v)) = line.split_once(':') {
                    let k = k.trim(); let v = v.trim();
                    if k == "Mode" { mode = v.parse::<i32>().ok(); }
                }
            }
            "Metadata" => {
                if let Some((k, v)) = line.split_once(':') {
                    let v = v.trim();
                    match k.trim() {
                        "Title" => title = v.to_string(),
                        "Artist" => artist = v.to_string(),
                        _ => {}
                    }
                }
            }
            "Difficulty" => {
                if let Some((k, v)) = line.split_once(':') {
                    let k = k.trim(); let v = v.trim();
                    if k == "CircleSize" { keys = v.parse::<usize>().unwrap_or(7).max(1).min(7); }
                }
            }
            "TimingPoints" => {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 8 {
                    let uninherited: i32 = parts[6].trim().parse().unwrap_or(0);
                    if uninherited == 1 {
                        let beat_length: f64 = parts[1].trim().parse().unwrap_or(500.0);
                        if beat_length > 0.0 {
                            bpm = 60000000.0 / beat_length / 1000.0;
                        }
                    }
                }
            }
            "HitObjects" => {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() < 4 { continue; }
                let x: f64 = match parts[0].trim().parse() { Ok(v) => v, _ => continue };
                let time_ms: f64 = match parts[2].trim().parse() { Ok(v) => v, _ => continue };
                let obj_type: i32 = match parts[3].trim().parse() { Ok(v) => v, _ => continue };

                let lane = ((x * keys as f64 / 512.0).floor() as usize).min(keys - 1);
                let note_time = time_ms / 1000.0;

                let mut end_time = 0.0;
                if (obj_type & 128) != 0 && parts.len() >= 6 {
                    let extras = parts[5];
                    if let Some(et_str) = extras.split(':').next() {
                        if let Ok(et_ms) = et_str.trim().parse::<f64>() {
                            end_time = et_ms / 1000.0;
                        }
                    }
                }

                notes.push(Note { time: note_time, lane, hit: false, end_time, tail_judged: false, release_time: 0.0 });
            }
            _ => {}
        }
    }

    if mode != Some(3) { return None; }

    notes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    let duration = notes.iter().map(|n| n.time).fold(0.0, f64::max) + 3.0;

    Some((Chart { title, artist, bpm, offset: 0.0, notes }, duration))
}
