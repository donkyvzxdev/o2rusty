use macroquad::prelude::*;

use crate::chart::{Chart, Note};
use crate::config::{self, GameConfig, keycode_name, hex_to_color, ScrollDirection, ScrollLayout, NoteStyle};

pub const LANE_COUNT: usize = 7;
const LEFT_COLUMNS_X: f32 = 30.0;
const TOP_Y: f32 = 0.0;
const BOT_Y: f32 = 555.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Judgment {
    Perfect,
    Great,
    Good,
    Bad,
    Miss,
}

pub struct TimingWindows {
    pub perfect: f64,
    pub great: f64,
    pub good: f64,
    pub bad: f64,
}

impl TimingWindows {
    pub fn judge(&self, delta_ms: f64) -> Option<Judgment> {
        let ms = delta_ms.abs() * 1000.0;
        if ms <= self.perfect { Some(Judgment::Perfect) }
        else if ms <= self.great { Some(Judgment::Great) }
        else if ms <= self.good { Some(Judgment::Good) }
        else if ms <= self.bad { Some(Judgment::Bad) }
        else { None }
    }
    pub fn miss_threshold(&self) -> f64 { self.bad / 1000.0 }
}

pub const DEFAULT_WINDOWS: TimingWindows = TimingWindows {
    perfect: 26.0, great: 50.0, good: 92.0, bad: 125.0,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JudgementDisplay {
    Active(Judgment, f64, f64),
    None,
}

pub struct ScoreSystem {
    pub score: u64,
    pub combo: u32,
    pub max_combo: u32,
    pub perfects: u32,
    pub greats: u32,
    pub goods: u32,
    pub bads: u32,
    pub misses: u32,
    pub total_notes: usize,
}

impl ScoreSystem {
    pub fn new(total_notes: usize) -> Self {
        ScoreSystem { score: 0, combo: 0, max_combo: 0, perfects: 0, greats: 0, goods: 0, bads: 0, misses: 0, total_notes }
    }
    pub fn register_hit(&mut self, judgment: Judgment) {
        match judgment {
            Judgment::Perfect => { self.combo += 1; self.score += 300 + (self.combo as u64).saturating_mul(10); self.perfects += 1; }
            Judgment::Great => { self.combo += 1; self.score += 200 + (self.combo as u64).saturating_mul(8); self.greats += 1; }
            Judgment::Good => { self.combo += 1; self.score += 100 + (self.combo as u64).saturating_mul(5); self.goods += 1; }
            Judgment::Bad => { self.score += 50; self.combo = 0; self.bads += 1; }
            Judgment::Miss => { self.combo = 0; self.misses += 1; }
        }
        if self.combo > self.max_combo { self.max_combo = self.combo; }
    }
    pub fn accuracy(&self) -> f64 {
        let total = self.hit_notes() as f64;
        if total == 0.0 { return 0.0; }
        let w = (self.perfects as f64 * 100.0) + (self.greats as f64 * 86.0) + (self.goods as f64 * 74.0) + (self.bads as f64 * 50.0);
        w / (total * 100.0)
    }
    pub fn hit_notes(&self) -> u32 { self.perfects + self.greats + self.goods + self.bads + self.misses }
}

fn note_progress(note_time: f64, current_time: f64, scroll_time: f64) -> f32 {
    let appear = note_time - scroll_time;
    let disappear = note_time + scroll_time * 0.1;
    let p = (current_time - appear) / (disappear - appear);
    p as f32
}

fn columns_x(layout: ScrollLayout, column_start: f32, screen_w: f32, lane_width: f32, spacing: f32) -> f32 {
    let total_w = LANE_COUNT as f32 * lane_width + (LANE_COUNT - 1) as f32 * spacing;
    let base = match layout {
        ScrollLayout::Center => (screen_w - total_w) / 2.0,
        ScrollLayout::Left => LEFT_COLUMNS_X,
    };
    base + column_start
}

fn lane_x(layout: ScrollLayout, column_start: f32, screen_w: f32, lane: usize, lane_width: f32, spacing: f32) -> f32 {
    columns_x(layout, column_start, screen_w, lane_width, spacing) + lane as f32 * (lane_width + spacing)
}

pub struct GameState {
    pub config: GameConfig,
    note_colors: [Color; LANE_COUNT],
    lane_width: f32,
    note_height: f32,
    effective_scroll_time: f64,
    pub notes: Vec<Note>,
    pub score: ScoreSystem,
    pub timing: TimingWindows,
    pub display: JudgementDisplay,
    pub finished: bool,
    pub key_states: [bool; LANE_COUNT],
    ghost_flash: [f64; LANE_COUNT],
    held_ln: [Option<usize>; LANE_COUNT],
    pub first_note_time: f64,
    last_note_end: f64,
}

impl GameState {
    pub fn new(chart: &Chart, config: GameConfig) -> Self {
        let mut notes = chart.notes.clone();
        notes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        let first_note_time = notes.first().map(|n| n.time).unwrap_or(0.0);
        let mut note_colors = [Color::new(1.0, 1.0, 1.0, 1.0); LANE_COUNT];
        for i in 0..LANE_COUNT {
            note_colors[i] = hex_to_color(&config.note_colors[i]);
        }

        let total_judgments = chart.notes.iter().map(|n| if n.end_time > 0.0 { 2 } else { 1 }).sum();
        let lane_width = config.column_width as f32;
        let note_height = config.note_height as f32;
        let last_note_end = notes.iter().map(|n| if n.end_time > 0.0 { n.end_time } else { n.time }).fold(0.0, f64::max) + DEFAULT_WINDOWS.bad / 1000.0;
        GameState {
            effective_scroll_time: config::BASE_SCROLL_TIME / config.scroll_speed,
            config, notes, note_colors,
            lane_width, note_height,
            score: ScoreSystem::new(total_judgments),
            timing: DEFAULT_WINDOWS,
            display: JudgementDisplay::None,
            finished: false,
            key_states: [false; LANE_COUNT],
            ghost_flash: [0.0; LANE_COUNT],
            held_ln: [None; LANE_COUNT],
            first_note_time,
            last_note_end,
        }
    }

    pub fn update(&mut self, t: f64) { self.check_misses(t); self.check_finished(t); self.update_display(t); }

    pub fn handle_input(&mut self, t: f64) -> bool {
        let mut skip_requested = false;
        for lane in 0..LANE_COUNT {
            let kc = self.config.key_bindings[lane];
            if is_key_pressed(kc) {
                self.key_states[lane] = true;
                let hit = self.try_hit_note(lane, t);
                if !hit && (kc == KeyCode::Space || kc == KeyCode::Backspace) {
                    skip_requested = true;
                }
            }
            if is_key_released(kc) {
                self.key_states[lane] = false;
                if let Some(held_idx) = self.held_ln[lane] {
                    self.release_ln(held_idx, t);
                }
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            skip_requested = true;
        }
        skip_requested
    }

    fn try_hit_note(&mut self, lane: usize, t: f64) -> bool {
        let threshold = self.timing.miss_threshold();
        let mut best: Option<(usize, f64)> = None;
        for (i, note) in self.notes.iter().enumerate() {
            if note.hit || note.lane != lane { continue; }
            let delta = note.time - t;
            if delta < -threshold { continue; }
            if delta > threshold { break; }
            match best {
                None => best = Some((i, delta.abs())),
                Some((_, d)) if delta.abs() < d => best = Some((i, delta.abs())),
                _ => {}
            }
        }
        if let Some((idx, delta)) = best {
            self.notes[idx].hit = true;
            if self.notes[idx].end_time > 0.0 {
                self.held_ln[lane] = Some(idx);
            }
            if let Some(j) = self.timing.judge(delta) { self.score.register_hit(j); self.display = JudgementDisplay::Active(j, t, delta * 1000.0); }
            true
        } else { self.ghost_flash[lane] = t; false }
    }

    fn release_ln(&mut self, idx: usize, t: f64) {
        let lane = self.notes[idx].lane;
        if !self.notes[idx].hit || self.notes[idx].tail_judged || self.notes[idx].end_time == 0.0 { return; }
        let delta = t - self.notes[idx].end_time;
        if let Some(j) = self.timing.judge(delta) {
            self.score.register_hit(j);
            self.display = JudgementDisplay::Active(j, t, delta * 1000.0);
        } else {
            self.score.register_hit(Judgment::Miss);
            self.display = JudgementDisplay::Active(Judgment::Miss, t, 0.0);
        }
        self.notes[idx].release_time = t;
        self.notes[idx].tail_judged = true;
        self.held_ln[lane] = None;
    }

    fn check_misses(&mut self, t: f64) {
        let th = self.timing.miss_threshold();
        for (i, note) in &mut self.notes.iter_mut().enumerate() {
            if !note.hit && t > note.time + th {
                note.hit = true;
                self.score.register_hit(Judgment::Miss);
                self.display = JudgementDisplay::Active(Judgment::Miss, t, 0.0);
            }
            // Auto-judge LN tail if past threshold and not yet judged
            if note.end_time > 0.0 && note.hit && !note.tail_judged && t > note.end_time + th {
                if let Some(held_idx) = self.held_ln[note.lane] {
                    if held_idx == i {
                        // Still held → Perfect release
                        self.score.register_hit(Judgment::Perfect);
                        self.display = JudgementDisplay::Active(Judgment::Perfect, t, 0.0);
                        self.held_ln[note.lane] = None;
                    } else {
                        // Different LN held in this lane, mark this one as missed tail
                        self.score.register_hit(Judgment::Miss);
                        self.display = JudgementDisplay::Active(Judgment::Miss, t, 0.0);
                    }
                } else {
                    // Key was released but tail not judged (shouldn't happen, but safety)
                    self.score.register_hit(Judgment::Miss);
                    self.display = JudgementDisplay::Active(Judgment::Miss, t, 0.0);
                }
                note.tail_judged = true;
            }
        }
    }

    fn check_finished(&mut self, t: f64) {
        if self.score.hit_notes() as usize >= self.notes.len() && t > self.last_note_end + 1.0 {
            self.finished = true;
        }
    }
    fn update_display(&mut self, t: f64) { if let JudgementDisplay::Active(_, time, _) = self.display { if t - time > 0.6 { self.display = JudgementDisplay::None; } } }

    pub fn can_skip(&self, t: f64) -> bool { t < self.first_note_time - 3.0 }

    pub fn skip_target(&self) -> f64 { (self.first_note_time - 3.0).max(0.0) }

    pub fn skip_to(&mut self, target: f64) {
        for note in &mut self.notes {
            if !note.hit && note.time < target {
                note.hit = true;
                self.score.register_hit(Judgment::Miss);
            }
        }
    }

    pub fn is_finished(&self) -> bool { self.finished }
    pub fn get_results(&self) -> &ScoreSystem { &self.score }

    pub fn draw(&self, t: f64) {
        self.draw_background();
        self.draw_scene(t);
        self.draw_hud(t);
        self.draw_judgment();
    }

    fn draw_background(&self) { clear_background(Color::new(0.05, 0.05, 0.08, 1.0)); }

    fn draw_scene(&self, t: f64) {
        let sw = screen_width();
        let cs = self.config.column_start as f32;
        let spacing = self.config.column_spacing as f32;
        let cx = columns_x(self.config.scroll_layout, cs, sw, self.lane_width, spacing);
        let tw = LANE_COUNT as f32 * self.lane_width + (LANE_COUNT - 1) as f32 * spacing;

        let down = self.config.scroll_direction == ScrollDirection::Down;
        let receptor_y = self.config.hit_position as f32;
        let spawn_y = if down { TOP_Y } else { BOT_Y };

        if self.config.column_bg_enabled {
            let bg_color = hex_to_color(&self.config.column_bg_color);
            if bg_color.a > 0.0 {
                for lane in 0..LANE_COUNT {
                    let x = cx + lane as f32 * (self.lane_width + spacing);
                    draw_rectangle(x, TOP_Y, self.lane_width, BOT_Y - TOP_Y + 15.0, bg_color);
                }
            }
        }

        if self.config.column_line_enabled {
            let line_color = hex_to_color(&self.config.column_line_color);
            if line_color.a > 0.0 {
                for lane in 0..=LANE_COUNT {
                    let x = cx + lane as f32 * (self.lane_width + spacing) - spacing * 0.5;
                    draw_line(x, TOP_Y, x, BOT_Y + 10.0, 1.0, line_color);
                }
            }
        }

        let st = self.effective_scroll_time;
        let miss_th = self.timing.miss_threshold();
        let shorten = self.config.ln_shortening_ms / 1000.0;
        for (i, note) in self.notes.iter().enumerate() {
            if note.hit && note.end_time == 0.0 && t > note.time + miss_th { continue; }
            if note.end_time > 0.0 {
                if note.tail_judged {
                    if note.release_time == 0.0 { continue; } // auto-perfect, hide
                    if t > note.end_time + miss_th { continue; } // early release past window
                }
            }
            let x = lane_x(self.config.scroll_layout, cs, sw, note.lane, self.lane_width, spacing) + 2.0;
            let w = self.lane_width - 4.0;
            let c = self.note_colors[note.lane];

            if note.end_time > 0.0 {
                let visual_tail = note.end_time - shorten;
                let p_head = note_progress(note.time, t, st);
                let p_tail = note_progress(visual_tail, t, st);
                let ln_appear = note.time - st;
                let ln_disappear = visual_tail + st * 0.1;
                if t < ln_appear || t > ln_disappear { continue; }
                let y1 = spawn_y + (receptor_y - spawn_y) * p_head.clamp(0.0, 1.0);
                let y2 = spawn_y + (receptor_y - spawn_y) * p_tail.clamp(0.0, 1.0);
                let head_top = y1 - self.note_height;
                let bar_top = head_top.min(y2);
                let bar_bot = y1.max(y2);

                // LN bar — always same visual, no transparency change when held
                let style = self.config.note_style;
                let head_y = y1;
                let bar_col = Color::new(c.r * 0.65, c.g * 0.65, c.b * 0.65, 1.0);

                match style {
                    NoteStyle::Rectangle => {
                        draw_rectangle(x, bar_top, w, bar_bot - bar_top, bar_col);
                    }
                    NoteStyle::Circle => {
                        let cr = (w * 0.5).min(self.note_height * 0.5);
                        let rw = (cr * 2.0).max(2.0);
                        let rx = x + (w - rw) * 0.5;
                        let gray = Color::new(0.55, 0.55, 0.58, 1.0);
                        let head_cy = head_y - self.note_height * 0.5;
                        let bar_end = if head_cy > bar_top { head_cy } else { bar_bot };
                        draw_rectangle(rx, bar_top, rw, bar_end - bar_top, gray);
                        draw_circle(rx + rw * 0.5, bar_top, rw * 0.5, gray);
                    }
                    NoteStyle::Arrow => {
                        if down {
                            draw_triangle(
                                Vec2::new(x, bar_bot), Vec2::new(x + w, bar_bot),
                                Vec2::new(x + w * 0.5, bar_top), bar_col,
                            );
                        } else {
                            draw_triangle(
                                Vec2::new(x, bar_top), Vec2::new(x + w, bar_top),
                                Vec2::new(x + w * 0.5, bar_bot), bar_col,
                            );
                        }
                    }
                }

                // Head cap (fully opaque, never changes)
                match style {
                    NoteStyle::Rectangle => {
                        draw_rectangle(x, head_y - self.note_height, w, self.note_height, c);
                    }
                    NoteStyle::Circle => {
                        let cx = x + w * 0.5;
                        let cy = head_y - self.note_height * 0.5;
                        let r = (w * 0.5).min(self.note_height * 0.5);
                        draw_circle(cx, cy, r, c);
                    }
                    NoteStyle::Arrow => {
                        let top = head_y - self.note_height;
                        if down {
                            draw_triangle(
                                Vec2::new(x, top), Vec2::new(x + w, top),
                                Vec2::new(x + w * 0.5, head_y), c,
                            );
                        } else {
                            draw_triangle(
                                Vec2::new(x, head_y), Vec2::new(x + w, head_y),
                                Vec2::new(x + w * 0.5, top), c,
                            );
                        }
                    }
                }

                // Tail cap (held indicator only for Rectangle and Arrow)
                match style {
                    NoteStyle::Rectangle => {
                        if note.hit && note.release_time == 0.0 && self.held_ln[note.lane] == Some(i) {
                            draw_rectangle(x, y2 - 2.0, w, 4.0, Color::new(0.3, 0.8, 1.0, 1.0));
                        } else {
                            draw_rectangle(x, y2 - 2.0, w, 4.0, Color::new(0.6, 0.6, 0.7, 1.0));
                        }
                    }
                    NoteStyle::Circle => {
                        // no tail cap — the pill bar end at bar_top is the finish
                    }
                    NoteStyle::Arrow => {
                        let tail_w = w * 0.25;
                        if note.hit && note.release_time == 0.0 && self.held_ln[note.lane] == Some(i) {
                            draw_rectangle(x + (w - tail_w) * 0.5, y2 - 2.0, tail_w, 4.0, Color::new(0.3, 0.8, 1.0, 1.0));
                        } else {
                            draw_rectangle(x + (w - tail_w) * 0.5, y2 - 2.0, tail_w, 4.0, Color::new(0.6, 0.6, 0.7, 1.0));
                        }
                    }
                }
            } else {
                let p = note_progress(note.time, t, st);
                if p < 0.0 || p > 1.1 { continue; }
                let pp = p.min(1.0);
                let y_bottom = spawn_y + (receptor_y - spawn_y) * pp;
                let style = self.config.note_style;
                match style {
                    NoteStyle::Rectangle => {
                        draw_rectangle(x, y_bottom - self.note_height, w, self.note_height, c);
                    }
                    NoteStyle::Circle => {
                        let cx = x + w * 0.5;
                        let cy = y_bottom - self.note_height * 0.5;
                        let r = (w * 0.5).min(self.note_height * 0.5);
                        draw_circle(cx, cy, r, c);
                    }
                    NoteStyle::Arrow => {
                        let top = y_bottom - self.note_height;
                        if down {
                            draw_triangle(
                                Vec2::new(x, top), Vec2::new(x + w, top),
                                Vec2::new(x + w * 0.5, y_bottom), c,
                            );
                        } else {
                            draw_triangle(
                                Vec2::new(x, y_bottom), Vec2::new(x + w, y_bottom),
                                Vec2::new(x + w * 0.5, top), c,
                            );
                        }
                    }
                }
                draw_line(x, y_bottom, x + w, y_bottom, 1.0, Color::new(1.0, 1.0, 1.0, 0.1));
            }
        }

        draw_line(cx, receptor_y, cx + tw, receptor_y, 2.0, Color::new(0.5, 0.6, 1.0, 0.6));
        draw_rectangle(cx, receptor_y - 3.0, tw, 6.0, Color::new(0.3, 0.4, 0.8, 0.15));
        for lane in 0..LANE_COUNT {
            let x = lane_x(self.config.scroll_layout, cs, sw, lane, self.lane_width, spacing);
            let c = if self.key_states[lane] { let mut col = self.note_colors[lane]; col.a = 0.4; col } else { Color::new(0.15, 0.15, 0.2, 0.3) };
            draw_rectangle(x, receptor_y - 2.0, self.lane_width, 4.0, c);
        }
        for lane in 0..LANE_COUNT {
            let elapsed = t - self.ghost_flash[lane];
            if elapsed > 0.0 && elapsed < 0.25 {
                let x = lane_x(self.config.scroll_layout, cs, sw, lane, self.lane_width, spacing);
                let alpha = ((1.0 - elapsed / 0.25) as f32 * 0.5).min(0.5);
                draw_rectangle(x, receptor_y - self.note_height, self.lane_width, self.note_height, Color::new(1.0, 1.0, 1.0, alpha));
            }
        }
        for lane in 0..LANE_COUNT {
            let x = lane_x(self.config.scroll_layout, cs, sw, lane, self.lane_width, spacing) + self.lane_width * 0.5;
            let kn = keycode_name(self.config.key_bindings[lane]);
            let c = if self.key_states[lane] { self.note_colors[lane] } else { Color::new(0.3, 0.3, 0.35, 1.0) };
            let ts = measure_text(kn, None, 18, 1.0);
            let ty = if down { receptor_y + 38.0 } else { receptor_y + 15.0 };
            draw_text(kn, x - ts.width * 0.5, ty, 18.0, c);
        }
    }

    fn draw_hud(&self, t: f64) {
        let score_text = format!("{:07}", self.score.score);
        let acc_text = format!("{:.2}%", self.score.accuracy() * 100.0);
        draw_text(&score_text, 20.0, 35.0, 32.0, Color::new(0.9, 0.9, 1.0, 1.0));
        draw_text("Tutorial", 20.0, 65.0, 16.0, Color::new(0.5, 0.5, 0.6, 1.0));
        let acc_size = measure_text(&acc_text, None, 20, 1.0);
        draw_text(&acc_text, screen_width() - acc_size.width - 20.0, 35.0, 20.0, Color::new(0.5, 0.7, 0.5, 1.0));
        if self.score.combo >= 10 {
            let ct = format!("{}", self.score.combo);
            let cs = measure_text(&ct, None, 52, 1.0);
            let cx = screen_width() * 0.5; let cy = screen_height() * 0.4;
            for &(dx, dy) in &[(2.0, 2.0), (-2.0, -2.0), (2.0, -2.0), (-2.0, 2.0)] {
                draw_text(&ct, cx - cs.width * 0.5 + dx, cy + dy, 52.0, Color::new(0.0, 0.0, 0.0, 0.3));
            }
            draw_text(&ct, cx - cs.width * 0.5, cy, 52.0, Color::new(1.0, 1.0, 1.0, 0.6));
            let cl = "Combo"; let ls = measure_text(cl, None, 18, 1.0);
            draw_text(cl, cx - ls.width * 0.5, cy + 30.0, 18.0, Color::new(1.0, 1.0, 1.0, 0.3));
        }
        if !self.notes.is_empty() {
            let pb = (self.score.hit_notes() as f64 / self.notes.len() as f64).min(1.0);
            let bw = 200.0; let bh = 4.0;
            let bx = screen_width() * 0.5 - bw * 0.5; let by = screen_height() - 30.0;
            draw_rectangle(bx, by, bw, bh, Color::new(0.2, 0.2, 0.25, 1.0));
            draw_rectangle(bx, by, bw * pb as f32, bh, Color::new(0.3, 0.6, 1.0, 0.8));
        }

        if self.can_skip(t) {
            let msg = "Press SPACE or BACKSPACE to Skip";
            let ms = measure_text(msg, None, 16, 1.0);
            let mx = screen_width() * 0.5;
            draw_text(msg, mx - ms.width * 0.5, screen_height() - 50.0, 16.0, Color::new(0.4, 0.4, 0.5, 0.6));
        }
    }

    fn draw_judgment(&self) {
        if let JudgementDisplay::Active(judgment, _t, offset_ms) = self.display {
            let (label, color) = match judgment {
                Judgment::Perfect => ("Perfect", Color::new(1.0, 0.8, 0.0, 1.0)),
                Judgment::Great => ("Great", Color::new(0.0, 1.0, 0.5, 1.0)),
                Judgment::Good => ("Good", Color::new(0.3, 0.6, 1.0, 1.0)),
                Judgment::Bad => ("Bad", Color::new(1.0, 0.5, 0.0, 1.0)),
                Judgment::Miss => ("Miss", Color::new(1.0, 0.2, 0.2, 1.0)),
            };
            let display = if judgment == Judgment::Miss { label.to_string() } else {
                let sign = if offset_ms >= 0.0 { "+" } else { "" };
                format!("{}  ({}{:.0}ms)", label, sign, offset_ms)
            };
            let down = self.config.scroll_direction == ScrollDirection::Down;
            let r = self.config.hit_position as f32;
            let ty = if down { r - 50.0 } else { r + 60.0 };
            let size = measure_text(&display, None, 26, 1.0);
            let tx = screen_width() * 0.5;
            draw_text(&display, tx - size.width * 0.5 + 1.0, ty + 1.0, 26.0, Color::new(0.0, 0.0, 0.0, 0.5));
            draw_text(&display, tx - size.width * 0.5, ty, 26.0, color);
        }
    }
}
