use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub time: f64,
    pub lane: usize,
    #[serde(skip)]
    pub hit: bool,
    pub end_time: f64,
    #[serde(skip)]
    pub tail_judged: bool,
    #[serde(skip)]
    pub release_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    pub title: String,
    pub artist: String,
    pub bpm: f64,
    pub offset: f64,
    pub notes: Vec<Note>,
}

impl Chart {
    pub fn generate_tutorial() -> Self {
        let bpm = 120.0;
        let mut notes = Vec::new();

        let beat = 60.0 / bpm;
        let mut t = 2.0;

        // Bars 1-2: ascending/descending single notes
        for &lane in &[0, 1, 2, 3, 4, 5, 6] {
            notes.push(Note { time: t, lane, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
            t += beat;
        }
        for &lane in &[5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6] {
            notes.push(Note { time: t, lane, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
            t += beat;
        }

        // Bars 3-4: eighth note alternations
        for &(a, b) in &[(0, 1), (2, 3), (4, 5), (6, 5), (3, 4), (1, 2), (0, 6)] {
            notes.push(Note { time: t, lane: a, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
            notes.push(Note { time: t + beat * 0.5, lane: b, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
            t += beat;
        }

        // Bars 5-6: two-note chords
        for &(a, b) in &[(0, 3), (1, 4), (2, 5), (3, 6), (4, 1), (5, 2), (6, 0), (0, 6)] {
            notes.push(Note { time: t, lane: a, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
            notes.push(Note { time: t, lane: b, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
            t += beat;
        }

        // Bars 7-8: trills
        for _ in 0..16 {
            notes.push(Note { time: t, lane: 3, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
            t += beat * 0.25;
            notes.push(Note { time: t, lane: 4, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
            t += beat * 0.25;
        }

        // Bar 9: long notes (LNs)
        for &lane in &[0, 2, 4, 6] {
            notes.push(Note { time: t, lane, hit: false, end_time: t + beat * 2.0, tail_judged: false, release_time: 0.0 });
            t += beat;
        }
        t += beat * 2.0; // skip the hold duration

        // Bars 10-11: three-note chords
        for &(a, b, c) in &[(0, 2, 4), (1, 3, 5), (0, 3, 6), (2, 4, 6), (0, 2, 5), (1, 4, 6)] {
            notes.push(Note { time: t, lane: a, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
            notes.push(Note { time: t, lane: b, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
            notes.push(Note { time: t, lane: c, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
            t += beat;
        }

        // Bars 11-12: scale runs
        for _ in 0..4 {
            for &lane in &[0, 1, 2, 3, 4, 5, 6] {
                notes.push(Note { time: t, lane, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
                t += beat * 0.25;
            }
            for &lane in &[5, 4, 3, 2, 1, 0] {
                notes.push(Note { time: t, lane, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
                t += beat * 0.25;
            }
        }

        // Bars 13-15: mixed dense pattern
        for _ in 0..12 {
            let lane = (rand::random::<usize>() % 7) as usize;
            notes.push(Note { time: t, lane, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
            t += beat * 0.25;
        }
        for _ in 0..8 {
            let a = (rand::random::<usize>() % 5) as usize;
            let b = a + 2;
            notes.push(Note { time: t, lane: a, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
            notes.push(Note { time: t, lane: b, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
            t += beat * 0.5;
        }

        // Bar 16: final chord
        for lane in 0..7 {
            notes.push(Note { time: t, lane, hit: false, end_time: 0.0, tail_judged: false, release_time: 0.0 });
        }

        notes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

        Chart {
            title: "Tutorial".into(),
            artist: "Procedural".into(),
            bpm,
            offset: 0.0,
            notes,
        }
    }

    pub fn duration(&self) -> f64 {
        let max = self.notes.iter().map(|n| n.end_time.max(n.time)).fold(0.0, f64::max);
        max + 2.0
    }

}
