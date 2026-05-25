use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

use rodio::{Decoder, OutputStream, Sink};

use crate::chart::Chart;

pub struct AudioEngine {
    _stream: OutputStream,
    sink: Sink,
    duration: f64,
    start_instant: Instant,
    pause_duration: f64,
    pause_start: Option<Instant>,
}

impl AudioEngine {
    pub fn new(chart: &Chart) -> Result<Self, Box<dyn std::error::Error>> {
        let (_stream, handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&handle)?;

        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("o2jam_audio.wav");
        generate_wav(&temp_path, chart.bpm, chart.duration())?;

        let mut engine = AudioEngine {
            _stream,
            sink,
            duration: chart.duration(),
            start_instant: Instant::now(),
            pause_duration: 0.0,
            pause_start: None,
        };
        engine.load_file(&temp_path)?;
        Ok(engine)
    }

    pub fn from_file(path: &Path, duration: f64) -> Result<Self, Box<dyn std::error::Error>> {
        let (_stream, handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&handle)?;

        let mut engine = AudioEngine {
            _stream,
            sink,
            duration,
            start_instant: Instant::now(),
            pause_duration: 0.0,
            pause_start: None,
        };
        engine.load_file(path)?;
        Ok(engine)
    }

    fn load_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let source = Decoder::new(BufReader::new(file))?;
        self.sink.append(source);
        Ok(())
    }

    pub fn pause(&mut self) {
        self.sink.pause();
        self.pause_start = Some(Instant::now());
    }

    pub fn resume(&mut self) {
        if let Some(start) = self.pause_start.take() {
            self.pause_duration += start.elapsed().as_secs_f64();
        }
        self.sink.play();
    }

    pub fn set_volume(&self, vol: f32) {
        self.sink.set_volume(vol);
    }

    pub fn seek(&mut self, time_secs: f64) {
        let dur = std::time::Duration::from_secs_f64(time_secs);
        if let Err(e) = self.sink.try_seek(dur) {
            eprintln!("Seek error: {}", e);
        }
        self.start_instant = Instant::now() - dur;
    }

    pub fn current_time(&self) -> f64 {
        let elapsed = self.start_instant.elapsed().as_secs_f64();
        let paused_so_far = self.pause_duration
            + self.pause_start.map_or(0.0, |s| s.elapsed().as_secs_f64());
        let adjusted = elapsed - paused_so_far;
        if adjusted > self.duration { self.duration } else { adjusted.max(0.0) }
    }
}

fn generate_wav(path: &Path, bpm: f64, duration: f64) -> Result<(), Box<dyn std::error::Error>> {
    let sample_rate = 44100;
    let num_samples = (duration * sample_rate as f64) as u64;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)?;
    let beat_dur = 60.0 / bpm;

    let mut last_pct = 0;

    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;

        let mut sample = 0.0f64;

        let beat_pos = t / beat_dur;
        let nearest_beat = (beat_pos.round() as i64).max(0) as f64 * beat_dur;
        let dist_to_beat = (t - nearest_beat).abs();

        if dist_to_beat < 0.06 {
            let env = (-dist_to_beat * 60.0).exp();
            let kick = (env * 0.40 * (std::f64::consts::PI * 80.0 * dist_to_beat).sin()).sin();
            sample += kick * env;
        }

        let beat_num = (beat_pos.floor() as i64) % 4;
        if (beat_num == 1 || beat_num == 3) && dist_to_beat < 0.04 {
            let env = (-dist_to_beat * 80.0).exp();
            let noise = rand::random::<f64>() * 2.0 - 1.0;
            sample += noise * env * 0.15;
            let snare_tone = (std::f64::consts::PI * 180.0 * dist_to_beat * 2.0).sin();
            sample += snare_tone * env * 0.15;
        }

        let eighth_pos = t / (beat_dur / 2.0);
        let nearest_eighth = (eighth_pos.round() as i64).max(0) as f64 * (beat_dur / 2.0);
        let dist_to_hat = (t - nearest_eighth).abs();
        if dist_to_hat < 0.015 {
            let env = (-dist_to_hat * 200.0).exp();
            let noise = rand::random::<f64>() * 2.0 - 1.0;
            sample += noise * env * 0.08;
        }

        let bar = (beat_pos / 4.0).floor() as i64;
        let bass_freq = match bar % 4 {
            0 => 130.81,
            1 => 98.0,
            2 => 110.0,
            3 => 103.83,
            _ => 130.81,
        };
        let bass = (std::f64::consts::PI * 2.0 * bass_freq * t).sin();
        let bass_env = (-((t % beat_dur) - beat_dur * 0.5).abs() * 5.0).exp();
        sample += bass * bass_env * 0.06;

        let pad_freqs = match bar % 4 {
            0 => [261.63, 329.63, 392.0],
            1 => [196.0, 246.94, 311.13],
            2 => [220.0, 277.18, 349.23],
            3 => [207.65, 261.63, 329.63],
            _ => [261.63, 329.63, 392.0],
        };
        for &f in &pad_freqs {
            let pad = (std::f64::consts::PI * 2.0 * f * t).sin();
            sample += pad * 0.015;
        }

        sample = sample.clamp(-1.0, 1.0);

        let int_sample = (sample * 0.8 * 32767.0) as i16;
        writer.write_sample(int_sample)?;

        let pct = i * 100 / num_samples;
        if pct != last_pct && pct % 10 == 0 {
            last_pct = pct;
        }
    }

    writer.finalize()?;
    Ok(())
}
