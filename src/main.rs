mod audio;
mod chart;
mod config;
mod game;
mod loader;

use std::path::PathBuf;

use macroquad::prelude::*;

use audio::AudioEngine;
use chart::Chart;
use config::GameConfig;
use config::hex_to_color;
use game::{GameState, LANE_COUNT};

const BG: Color = Color::new(0.05, 0.05, 0.08, 1.0);
const GOLD: Color = Color::new(1.0, 0.85, 0.2, 1.0);
const CYAN: Color = Color::new(0.2, 0.7, 1.0, 1.0);
const WHITE_SOFT: Color = Color::new(0.7, 0.7, 0.8, 1.0);
const DIM: Color = Color::new(0.4, 0.4, 0.5, 1.0);

const PALETTE_COLORS: &[&str] = &[
    "#FF4444", "#FF8800", "#FFCC00", "#44CC44", "#00CCCC",
    "#4488FF", "#8844FF", "#CC44CC", "#FF6688", "#FFAA44",
    "#CCFF44", "#66FF66", "#44FFCC", "#6688FF", "#AA66FF",
    "#FF66AA", "#FFFFFF", "#AAAAAA", "#666666", "#222222",
    "#9C59B5", "#3399DB", "#2ECC70", "#F2C40F", "#E67D21",
    "#E84D3D", "#EDEDF2",
];
const PALETTE_COLS: usize = 5;
const PALETTE_SQ: f32 = 22.0;
const PALETTE_GAP: f32 = 3.0;

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum SettingsPrev {
    MainMenu,
    SongSelect,
}

enum AudioSource {
    Procedural,
    File { path: PathBuf, duration: f64 },
}

struct GameResources {
    chart: Chart,
    audio_source: AudioSource,
}

enum Screen {
    MainMenu { selected: usize },
    SongSelect { selected: usize, songs: Vec<loader::SongGroup>, scroll_y: f32 },
    DiffSelect { diffs: Vec<loader::DiffInfo>, selected: usize, scroll_y: f32 },
    Settings { selected_item: usize, rebinding_lane: Option<usize>, picking_color: Option<usize>, scroll_y: f32, topic: usize, prev: SettingsPrev },
    Playing { state: GameState, audio: AudioEngine, resources: GameResources, paused: bool, resume_timer: f64 },
    Results { score: u64, max_combo: u32, perfects: u32, greats: u32, goods: u32, bads: u32, misses: u32, accuracy: f64, total: usize, grade: &'static str },
    Exit,
}

#[macroquad::main("o2jam-rust")]
async fn main() {
    request_new_screen_size(960.0, 700.0);

    let mut config = config::load_config();
    let mut screen = Screen::MainMenu { selected: 0 };
    let mut vol_alpha = 0.0f32;

    loop {
        clear_background(BG);

        let audio_time = match &screen {
            Screen::Playing { audio, .. } => Some(audio.current_time()),
            _ => None,
        };

        match &screen {
            Screen::MainMenu { selected } => draw_main_menu(*selected),
            Screen::SongSelect { selected, songs, scroll_y } => draw_song_select(*selected, songs, *scroll_y),
            Screen::DiffSelect { diffs, selected, scroll_y, .. } => draw_diff_select(*selected, diffs, *scroll_y),
            Screen::Settings { selected_item, rebinding_lane, picking_color, scroll_y, topic, .. } => draw_settings(&config, *selected_item, *rebinding_lane, *picking_color, *scroll_y, *topic),
            Screen::Playing { state, paused, resume_timer, .. } => {
                state.draw(audio_time.unwrap());
                if *paused { draw_pause_menu(*resume_timer); }
            }
            Screen::Results { score, max_combo, perfects, greats, goods, bads, misses, accuracy, total, grade } => {
                draw_results(*score, *max_combo, *perfects, *greats, *goods, *bads, *misses, *accuracy, *total, grade)
            }
            Screen::Exit => {}
        }

        // Volume overlay
        let is_menu = matches!(&screen, Screen::MainMenu { .. } | Screen::SongSelect { .. } | Screen::DiffSelect { .. } | Screen::Settings { .. } | Screen::Playing { .. });

        let vol_cx = screen_width() - 50.0;
        let vol_cy = screen_height() - 80.0;
        let vol_radius = 22.0;

        if is_menu {
            let (_wx, wy) = mouse_wheel();
            if wy != 0.0 {
                if !matches!(&screen, Screen::Settings { .. }) {
                    let step = -wy.signum() as f64 * 0.05;
                    config.volume = (config.volume + step).clamp(0.0, 1.0);
                    config.volume = (config.volume * 20.0).round() / 20.0;
                }
                vol_alpha = 1.0;
            }
        }

        let (mx, my) = mouse_position();
        let vol_hover = (mx - vol_cx).powi(2) + (my - vol_cy).powi(2) <= (vol_radius + 5.0f32).powi(2);

        if vol_hover && vol_alpha > 0.0 {
            vol_alpha = 1.0;
        } else if vol_alpha > 0.0 {
            vol_alpha = (vol_alpha - get_frame_time() / 2.0).max(0.0);
        }

        if vol_alpha > 0.0 {
            let c = Color::new(0.1, 0.1, 0.15, vol_alpha * 0.85);
            draw_circle(vol_cx, vol_cy, vol_radius, c);
            draw_circle_lines(vol_cx, vol_cy, vol_radius, 2.0, Color::new(0.3, 0.6, 1.0, vol_alpha));

            let pct = format!("{}%", (config.volume * 100.0).round() as u32);
            let ts = measure_text(&pct, None, 16, 1.0);
            draw_text(&pct, vol_cx - ts.width * 0.5, vol_cy + 5.0, 16.0, Color::new(1.0, 1.0, 1.0, vol_alpha));

            let vs = measure_text("Volume", None, 12, 1.0);
            draw_text("Volume", vol_cx - vs.width * 0.5, vol_cy - vol_radius - 8.0, 12.0, Color::new(0.5, 0.5, 0.6, vol_alpha));

            let gs = measure_text("Geral", None, 12, 1.0);
            draw_text("Geral", vol_cx - gs.width * 0.5, vol_cy + vol_radius + 16.0, 12.0, Color::new(0.5, 0.5, 0.6, vol_alpha));
        }

        let transition = match &mut screen {
            Screen::MainMenu { selected } => handle_main_menu(selected),
            Screen::SongSelect { selected, songs, scroll_y } => handle_song_select(selected, songs, scroll_y, &config),
            Screen::DiffSelect { diffs, selected, scroll_y, .. } => handle_diff_select(selected, diffs, scroll_y, &config),
            Screen::Settings { selected_item, rebinding_lane, picking_color, scroll_y, topic, prev } => handle_settings(selected_item, rebinding_lane, picking_color, scroll_y, topic, prev, &mut config),
            Screen::Playing { state, audio, resources, paused, resume_timer } => {
                if *paused {
                    handle_paused(state, audio, resources, paused, resume_timer, &config)
                } else {
                    let result = handle_playing(state, audio);
                    if result.is_none() && is_key_pressed(KeyCode::Escape) {
                        audio.pause();
                        *paused = true;
                    }
                    result
                }
            }
            Screen::Results { .. } => handle_results(),
            Screen::Exit => None,
        };

        if let Some(new_screen) = transition {
            match new_screen {
                Screen::Exit => { config::save_config(&config); break; }
                s => screen = s,
            }
        }

        next_frame().await;
    }
}

fn mouse_in_rect(x: f32, y: f32, w: f32, h: f32) -> bool {
    let (mx, my) = mouse_position();
    mx >= x && mx <= x + w && my >= y && my <= y + h
}

fn draw_menu_item(text: &str, x: f32, y: f32, selected: bool, hovered: bool, font_size: f32) {
    let color = if selected || hovered { CYAN } else { WHITE_SOFT };
    let prefix = if selected || hovered { "▶ " } else { "  " };
    draw_text(&format!("{}{}", prefix, text), x, y, font_size, color);
}

fn draw_title_screen() {
    let cx = screen_width() / 2.0;
    let title = "O2JAM-RUST";
    let sub = "7-Key Rhythm Game";

    let ts = measure_text(title, None, 60, 1.0);
    draw_text(title, cx - ts.width / 2.0, 170.0, 60.0, GOLD);

    let ss = measure_text(sub, None, 18, 1.0);
    draw_text(sub, cx - ss.width / 2.0, 200.0, 18.0, DIM);
}

fn draw_main_menu(selected: usize) {
    draw_title_screen();

    let items = ["Play", "Settings", "Exit"];
    let cx = screen_width() / 2.0;
    let start_y = 290.0;
    for (i, item) in items.iter().enumerate() {
        let y = start_y + i as f32 * 52.0;
        let size = measure_text(item, None, 26, 1.0);
        let text_x = cx - size.width / 2.0 - 30.0;
        let hovered = mouse_in_rect(text_x, y - 22.0, size.width + 60.0, 28.0);
        let is_selected = i == selected;
        if hovered {
            draw_rectangle(text_x - 5.0, y - 22.0, size.width + 70.0, 28.0, Color::new(0.2, 0.7, 1.0, 0.08));
        }
        draw_menu_item(item, text_x, y, is_selected, hovered, 26.0);
    }
}

fn draw_song_select(selected: usize, songs: &[loader::SongGroup], scroll_y: f32) {
    let cx = screen_width() / 2.0;
    let sy = |cy: f32| cy - scroll_y;
    let start_y = 230.0;
    let sh = screen_height();

    // Clamp drawing to visible area
    let in_view = |y: f32| y > 0.0 && y < sh + 30.0;

    let tut_y = sy(start_y);
    let tut_hovered = mouse_in_rect(cx - 80.0, tut_y - 18.0, 160.0, 26.0) && tut_y > 0.0 && tut_y < sh;
    if tut_hovered {
        draw_rectangle(cx - 85.0, tut_y - 18.0, 170.0, 26.0, Color::new(0.2, 0.7, 1.0, 0.08));
    }
    if in_view(tut_y) { draw_menu_item("Tutorial", cx - 50.0, tut_y, selected == 0, tut_hovered, 22.0); }

    let header_y = sy(start_y + 50.0);
    if in_view(header_y) {
        let htext = "Imported Songs";
        let hs = measure_text(htext, None, 16, 1.0);
        draw_text(htext, cx - hs.width / 2.0, header_y, 16.0, DIM);
    }

    for (i, song) in songs.iter().enumerate() {
        let y = sy(start_y + 70.0 + i as f32 * 36.0);
        if !in_view(y) { continue; }
        let label = format!("{} - {}  ({} diffs)", song.artist, song.title, song.difficulty_count);
        let sel_i = i + 1;
        let hovered = mouse_in_rect(cx - 240.0, y - 16.0, 480.0, 24.0) && y > 0.0 && y < sh;
        if hovered {
            draw_rectangle(cx - 245.0, y - 16.0, 490.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08));
        }
        draw_menu_item(&label, cx - 220.0, y, selected == sel_i, hovered, 17.0);
    }

    let list_end = start_y + 70.0 + songs.len() as f32 * 36.0;
    let back_y = sy(list_end + 30.0);
    let back_idx = songs.len() + 1;
    let back_hovered = mouse_in_rect(cx - 80.0, back_y - 18.0, 160.0, 24.0) && back_y > 0.0 && back_y < sh;
    if back_hovered {
        draw_rectangle(cx - 85.0, back_y - 18.0, 170.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08));
    }
    if in_view(back_y) { draw_menu_item("Back", cx - 50.0, back_y, selected == back_idx, back_hovered, 22.0); }

    let hint = "Enter/Space to select, Esc to go back";
    let hs = measure_text(hint, None, 14, 1.0);
    draw_text(hint, cx - hs.width / 2.0, screen_height() - 40.0, 14.0, DIM);
}

fn draw_diff_select(selected: usize, diffs: &[loader::DiffInfo], scroll_y: f32) {
    let cx = screen_width() / 2.0;
    let sy = |cy: f32| cy - scroll_y;
    let sh = screen_height();
    let in_view = |y: f32| y > 0.0 && y < sh + 30.0;

    if let Some(first) = diffs.first() {
        let header = format!("{} - {}", first.artist, first.title);
        let hs = measure_text(&header, None, 28, 1.0);
        draw_text(&header, cx - hs.width / 2.0, sy(160.0), 28.0, GOLD);
    }

    let sub = "Select Difficulty";
    let ss = measure_text(sub, None, 18, 1.0);
    draw_text(sub, cx - ss.width / 2.0, sy(190.0), 18.0, DIM);

    let start_y = 230.0;
    for (i, diff) in diffs.iter().enumerate() {
        let y = sy(start_y + i as f32 * 40.0);
        if !in_view(y) { continue; }
        let hovered = mouse_in_rect(cx - 120.0, y - 18.0, 240.0, 26.0) && y > 0.0 && y < sh;
        if hovered {
            draw_rectangle(cx - 125.0, y - 18.0, 250.0, 26.0, Color::new(0.2, 0.7, 1.0, 0.08));
        }
        draw_menu_item(&diff.name, cx - 100.0, y, selected == i, hovered, 22.0);
    }

    let list_end = start_y + diffs.len() as f32 * 40.0;
    let back_y = sy(list_end + 40.0);
    let back_idx = diffs.len();
    let back_hovered = mouse_in_rect(cx - 80.0, back_y - 18.0, 160.0, 24.0) && back_y > 0.0 && back_y < sh;
    if back_hovered {
        draw_rectangle(cx - 85.0, back_y - 18.0, 170.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08));
    }
    if in_view(back_y) { draw_menu_item("Back", cx - 50.0, back_y, selected == back_idx, back_hovered, 22.0); }
}

fn draw_settings(config: &GameConfig, selected_item: usize, rebinding_lane: Option<usize>, picking_color: Option<usize>, scroll_y: f32, topic: usize) {
    let cx = screen_width() / 2.0;
    let sy = |cy: f32| cy - scroll_y;

    // ── Topic tabs ──
    let tab_names = ["Controls", "Gameplay", "Visuals", "Audio"];
    let tab_w = 130.0;
    let tab_h = 30.0;
    let tab_y = 95.0;
    let total_w = tab_names.len() as f32 * tab_w;
    let start_tab_x = (screen_width() - total_w) / 2.0;

    for (i, name) in tab_names.iter().enumerate() {
        let tx = start_tab_x + i as f32 * tab_w;
        let is_active = topic == i;
        let hovered = mouse_in_rect(tx, tab_y - tab_h, tab_w, tab_h);
        let bg = if is_active { CYAN } else if hovered { Color::new(0.2, 0.7, 1.0, 0.15) } else { Color::new(0.12, 0.12, 0.18, 0.6) };
        draw_rectangle(tx, tab_y - tab_h, tab_w, tab_h, bg);
        if is_active { draw_rectangle_lines(tx, tab_y - tab_h, tab_w, tab_h, 1.5, CYAN); }
        let text_color = if is_active { Color::new(0.0, 0.0, 0.0, 1.0) } else if hovered { CYAN } else { WHITE_SOFT };
        let ts = measure_text(name, None, 17, 1.0);
        draw_text(name, tx + (tab_w - ts.width) / 2.0, tab_y - 7.0, 17.0, text_color);
    }

    let gap = 30.0;
    let start_y = 155.0;
    let back_idx = LANE_COUNT + 20;

    match topic {
        0 => {
            // ── Controls ──
            for lane in 0..LANE_COUNT {
                let cy = start_y + lane as f32 * 36.0;
                let y = sy(cy);
                let is_sel = selected_item == lane;
                let is_rebinding = rebinding_lane == Some(lane);
                let label = format!("Lane {}: ", lane + 1);
                let key_name = if is_rebinding { "?" } else { config::keycode_name(config.key_bindings[lane]) };
                let txt = format!("{}{}", label, key_name);
                let is_h = mouse_in_rect(220.0, y - 22.0, 200.0, 24.0);
                if is_h || is_sel { draw_rectangle(218.0, y - 22.0, 204.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
                let col = if is_rebinding { CYAN } else if is_h || is_sel { CYAN } else { WHITE_SOFT };
                let fs = if is_h || is_sel { 24.0 } else { 20.0 };
                draw_text(&txt, 220.0, y, fs, col);
                if is_rebinding {
                    draw_text("Press a key...", 220.0, y + 22.0, 18.0, DIM);
                }
                // Color swatch
                let sc = hex_to_color(&config.note_colors[lane]);
                draw_rectangle(450.0, y - 16.0, 16.0, 16.0, sc);
                if picking_color == Some(lane) {
                    let rows = (PALETTE_COLORS.len() + PALETTE_COLS - 1) / PALETTE_COLS;
                    let pal_h = rows as f32 * (PALETTE_SQ + PALETTE_GAP) + 10.0;
                    let pal_x = 470.0; let pal_y = y - 10.0;
                    let pal_w = PALETTE_COLS as f32 * (PALETTE_SQ + PALETTE_GAP) + 10.0;
                    draw_rectangle(pal_x - 2.0, pal_y - 2.0, pal_w + 4.0, pal_h + 4.0, Color::new(0.05, 0.05, 0.08, 1.0));
                    draw_rectangle_lines(pal_x - 2.0, pal_y - 2.0, pal_w + 4.0, pal_h + 4.0, 1.5, CYAN);
                    for (idx, &hex) in PALETTE_COLORS.iter().enumerate() {
                        let col = idx % PALETTE_COLS; let row = idx / PALETTE_COLS;
                        let pcx = pal_x + 5.0 + col as f32 * (PALETTE_SQ + PALETTE_GAP);
                        let pcy = pal_y + 5.0 + row as f32 * (PALETTE_SQ + PALETTE_GAP);
                        draw_rectangle(pcx, pcy, PALETTE_SQ, PALETTE_SQ, hex_to_color(hex));
                        if mouse_in_rect(pcx, pcy, PALETTE_SQ, PALETTE_SQ) { draw_rectangle_lines(pcx - 1.0, pcy - 1.0, PALETTE_SQ + 2.0, PALETTE_SQ + 2.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.8)); }
                    }
                }
            }
            let back_cy = start_y + LANE_COUNT as f32 * 36.0 + gap;
            let back_y = sy(back_cy);
            let back_hovered = mouse_in_rect(cx - 80.0, back_y - 18.0, 160.0, 24.0);
            if back_hovered { draw_rectangle(cx - 85.0, back_y - 18.0, 170.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_menu_item("Back", cx - 50.0, back_y, selected_item == back_idx, back_hovered, 22.0);
        }
        1 => {
            // ── Gameplay ──
            let ss_cy = start_y; let co_cy = ss_cy + gap; let hp_cy = co_cy + gap;
            let sd_cy = hp_cy + gap; let sl_cy = sd_cy + gap;
            let ss_y = sy(ss_cy); let co_y = sy(co_cy); let hp_y = sy(hp_cy);
            let sd_y = sy(sd_cy); let sl_y = sy(sl_cy);
            let arrow = |y: f32, dec_x: f32, inc_x: f32, dec_ox: f32, inc_ox: f32| {
                let ay = y - 8.0;
                let dh = mouse_in_rect(dec_x, ay - 10.0, 28.0, 20.0);
                let ih = mouse_in_rect(inc_x, ay - 10.0, 28.0, 20.0);
                draw_text("<", dec_ox, y, if dh { 22.0 } else { 18.0 }, if dh { CYAN } else { DIM });
                draw_text(">", inc_ox, y, if ih { 22.0 } else { 18.0 }, if ih { CYAN } else { DIM });
            };

            let ss_text = format!("Scroll Speed: {:.1}x", config.scroll_speed);
            let ss_h = mouse_in_rect(220.0, ss_y - 22.0, 300.0, 24.0);
            if ss_h { draw_rectangle(218.0, ss_y - 22.0, 304.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_text(&ss_text, 220.0, ss_y, if selected_item == LANE_COUNT || ss_h { 24.0 } else { 20.0 }, if selected_item == LANE_COUNT || ss_h { CYAN } else { WHITE_SOFT });
            arrow(ss_y, 440.0, 475.0, 448.0, 483.0);

            let co_text = format!("Column Offset: {:.0}px", config.column_start);
            let co_h = mouse_in_rect(220.0, co_y - 22.0, 350.0, 24.0);
            if co_h { draw_rectangle(218.0, co_y - 22.0, 354.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_text(&co_text, 220.0, co_y, if selected_item == LANE_COUNT+1 || co_h { 24.0 } else { 20.0 }, if selected_item == LANE_COUNT+1 || co_h { CYAN } else { WHITE_SOFT });
            arrow(co_y, 475.0, 510.0, 483.0, 518.0);

            let hp_text = format!("Hit Position: {:.0}", config.hit_position);
            let hp_h = mouse_in_rect(220.0, hp_y - 22.0, 350.0, 24.0);
            if hp_h { draw_rectangle(218.0, hp_y - 22.0, 354.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_text(&hp_text, 220.0, hp_y, if selected_item == LANE_COUNT+2 || hp_h { 24.0 } else { 20.0 }, if selected_item == LANE_COUNT+2 || hp_h { CYAN } else { WHITE_SOFT });
            arrow(hp_y, 500.0, 535.0, 508.0, 543.0);

            let sd_text = format!("Scroll Dir: {}", config.scroll_direction.name());
            let sd_h = mouse_in_rect(220.0, sd_y - 22.0, 300.0, 24.0);
            if sd_h { draw_rectangle(218.0, sd_y - 22.0, 304.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_text(&sd_text, 220.0, sd_y, if selected_item == LANE_COUNT+3 || sd_h { 24.0 } else { 20.0 }, if selected_item == LANE_COUNT+3 || sd_h { CYAN } else { WHITE_SOFT });

            let sl_text = format!("Scroll Layout: {}", config.scroll_layout.name());
            let sl_h = mouse_in_rect(220.0, sl_y - 22.0, 300.0, 24.0);
            if sl_h { draw_rectangle(218.0, sl_y - 22.0, 304.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_text(&sl_text, 220.0, sl_y, if selected_item == LANE_COUNT+4 || sl_h { 24.0 } else { 20.0 }, if selected_item == LANE_COUNT+4 || sl_h { CYAN } else { WHITE_SOFT });

            let back_cy = sl_cy + gap + 10.0;
            let back_y = sy(back_cy);
            let back_hovered = mouse_in_rect(cx - 80.0, back_y - 18.0, 160.0, 24.0);
            if back_hovered { draw_rectangle(cx - 85.0, back_y - 18.0, 170.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_menu_item("Back", cx - 50.0, back_y, selected_item == back_idx, back_hovered, 22.0);
        }
        2 => {
            // ── Visuals ──
            let nh_cy = start_y; let cw_cy = nh_cy + gap; let ls_cy = cw_cy + gap; let ns_cy = ls_cy + gap;
            let cs_cy = ns_cy + gap; let cl_cy = cs_cy + gap; let cb_cy = cl_cy + gap;
            let cle_cy = cb_cy + gap; let cbe_cy = cle_cy + gap;
            let nh_y = sy(nh_cy); let cw_y = sy(cw_cy); let ls_y = sy(ls_cy); let ns_y = sy(ns_cy);
            let cs_y = sy(cs_cy); let cl_y = sy(cl_cy); let cb_y = sy(cb_cy);
            let cle_y = sy(cle_cy); let cbe_y = sy(cbe_cy);
            let arrow = |y: f32| {
                let dh = mouse_in_rect(500.0, y - 18.0, 28.0, 20.0);
                let ih = mouse_in_rect(535.0, y - 18.0, 28.0, 20.0);
                draw_text("<", 508.0, y, if dh { 22.0 } else { 18.0 }, if dh { CYAN } else { DIM });
                draw_text(">", 543.0, y, if ih { 22.0 } else { 18.0 }, if ih { CYAN } else { DIM });
            };

            let nh_text = format!("Note Height: {:.0}", config.note_height);
            let nh_h = mouse_in_rect(220.0, nh_y - 22.0, 350.0, 24.0);
            if nh_h { draw_rectangle(218.0, nh_y - 22.0, 354.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_text(&nh_text, 220.0, nh_y, if selected_item == LANE_COUNT+5 || nh_h { 24.0 } else { 20.0 }, if selected_item == LANE_COUNT+5 || nh_h { CYAN } else { WHITE_SOFT });
            arrow(nh_y);

            let cw_text = format!("Column Width: {:.0}", config.column_width);
            let cw_h = mouse_in_rect(220.0, cw_y - 22.0, 350.0, 24.0);
            if cw_h { draw_rectangle(218.0, cw_y - 22.0, 354.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_text(&cw_text, 220.0, cw_y, if selected_item == LANE_COUNT+6 || cw_h { 24.0 } else { 20.0 }, if selected_item == LANE_COUNT+6 || cw_h { CYAN } else { WHITE_SOFT });
            arrow(cw_y);

            let ls_text = format!("LN Shortening: {:.0} ms", config.ln_shortening_ms);
            let ls_h = mouse_in_rect(220.0, ls_y - 22.0, 350.0, 24.0);
            if ls_h { draw_rectangle(218.0, ls_y - 22.0, 354.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_text(&ls_text, 220.0, ls_y, if selected_item == LANE_COUNT+7 || ls_h { 24.0 } else { 20.0 }, if selected_item == LANE_COUNT+7 || ls_h { CYAN } else { WHITE_SOFT });
            let ls_ay = ls_y - 8.0;
            let ls_dh = mouse_in_rect(500.0, ls_ay - 10.0, 28.0, 20.0);
            let ls_ih = mouse_in_rect(535.0, ls_ay - 10.0, 28.0, 20.0);
            draw_text("<", 508.0, ls_y, if ls_dh { 22.0 } else { 18.0 }, if ls_dh { CYAN } else { DIM });
            draw_text(">", 543.0, ls_y, if ls_ih { 22.0 } else { 18.0 }, if ls_ih { CYAN } else { DIM });

            let ns_text = format!("Note Style: {}", config.note_style.name());
            let ns_h = mouse_in_rect(220.0, ns_y - 22.0, 350.0, 24.0);
            if ns_h { draw_rectangle(218.0, ns_y - 22.0, 354.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_text(&ns_text, 220.0, ns_y, if selected_item == LANE_COUNT+8 || ns_h { 24.0 } else { 20.0 }, if selected_item == LANE_COUNT+8 || ns_h { CYAN } else { WHITE_SOFT });

            // Column Spacing
            let cs_text = format!("Column Spacing: {:.0}px", config.column_spacing);
            let cs_h = mouse_in_rect(220.0, cs_y - 22.0, 350.0, 24.0);
            if cs_h { draw_rectangle(218.0, cs_y - 22.0, 354.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_text(&cs_text, 220.0, cs_y, if selected_item == LANE_COUNT+10 || cs_h { 24.0 } else { 20.0 }, if selected_item == LANE_COUNT+10 || cs_h { CYAN } else { WHITE_SOFT });
            let cs_ay = cs_y - 8.0;
            let cs_dh = mouse_in_rect(500.0, cs_ay - 10.0, 28.0, 20.0);
            let cs_ih = mouse_in_rect(535.0, cs_ay - 10.0, 28.0, 20.0);
            draw_text("<", 508.0, cs_y, if cs_dh { 22.0 } else { 18.0 }, if cs_dh { CYAN } else { DIM });
            draw_text(">", 543.0, cs_y, if cs_ih { 22.0 } else { 18.0 }, if cs_ih { CYAN } else { DIM });

            // Column Line Color swatch
            let cl_text = "Column Line Color:";
            let cl_h = mouse_in_rect(220.0, cl_y - 22.0, 350.0, 24.0);
            if cl_h { draw_rectangle(218.0, cl_y - 22.0, 354.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_text(cl_text, 220.0, cl_y, if selected_item == LANE_COUNT+11 || cl_h { 24.0 } else { 20.0 }, if selected_item == LANE_COUNT+11 || cl_h { CYAN } else { WHITE_SOFT });
            let cl_swatch = hex_to_color(&config.column_line_color);
            draw_rectangle(450.0, cl_y - 16.0, 16.0, 16.0, cl_swatch);
            if picking_color == Some(100) {
                let rows = (PALETTE_COLORS.len() + PALETTE_COLS - 1) / PALETTE_COLS;
                let pal_h = rows as f32 * (PALETTE_SQ + PALETTE_GAP) + 10.0;
                let pal_x = 470.0; let pal_y2 = cl_y - 10.0;
                let pal_w = PALETTE_COLS as f32 * (PALETTE_SQ + PALETTE_GAP) + 10.0;
                draw_rectangle(pal_x - 2.0, pal_y2 - 2.0, pal_w + 4.0, pal_h + 4.0, Color::new(0.05, 0.05, 0.08, 1.0));
                draw_rectangle_lines(pal_x - 2.0, pal_y2 - 2.0, pal_w + 4.0, pal_h + 4.0, 1.5, CYAN);
                for (idx, &hex) in PALETTE_COLORS.iter().enumerate() {
                    let col = idx % PALETTE_COLS; let row = idx / PALETTE_COLS;
                    let pcx = pal_x + 5.0 + col as f32 * (PALETTE_SQ + PALETTE_GAP);
                    let pcy = pal_y2 + 5.0 + row as f32 * (PALETTE_SQ + PALETTE_GAP);
                    draw_rectangle(pcx, pcy, PALETTE_SQ, PALETTE_SQ, hex_to_color(hex));
                    if mouse_in_rect(pcx, pcy, PALETTE_SQ, PALETTE_SQ) { draw_rectangle_lines(pcx - 1.0, pcy - 1.0, PALETTE_SQ + 2.0, PALETTE_SQ + 2.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.8)); }
                }
            }

            // Column BG Color swatch
            let cb_text = "Column BG Color:";
            let cb_h = mouse_in_rect(220.0, cb_y - 22.0, 350.0, 24.0);
            if cb_h { draw_rectangle(218.0, cb_y - 22.0, 354.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_text(cb_text, 220.0, cb_y, if selected_item == LANE_COUNT+12 || cb_h { 24.0 } else { 20.0 }, if selected_item == LANE_COUNT+12 || cb_h { CYAN } else { WHITE_SOFT });
            let cb_swatch = hex_to_color(&config.column_bg_color);
            draw_rectangle(450.0, cb_y - 16.0, 16.0, 16.0, cb_swatch);
            if picking_color == Some(101) {
                let rows = (PALETTE_COLORS.len() + PALETTE_COLS - 1) / PALETTE_COLS;
                let pal_h = rows as f32 * (PALETTE_SQ + PALETTE_GAP) + 10.0;
                let pal_x = 470.0; let pal_y2 = cb_y - 10.0;
                let pal_w = PALETTE_COLS as f32 * (PALETTE_SQ + PALETTE_GAP) + 10.0;
                draw_rectangle(pal_x - 2.0, pal_y2 - 2.0, pal_w + 4.0, pal_h + 4.0, Color::new(0.05, 0.05, 0.08, 1.0));
                draw_rectangle_lines(pal_x - 2.0, pal_y2 - 2.0, pal_w + 4.0, pal_h + 4.0, 1.5, CYAN);
                for (idx, &hex) in PALETTE_COLORS.iter().enumerate() {
                    let col = idx % PALETTE_COLS; let row = idx / PALETTE_COLS;
                    let pcx = pal_x + 5.0 + col as f32 * (PALETTE_SQ + PALETTE_GAP);
                    let pcy = pal_y2 + 5.0 + row as f32 * (PALETTE_SQ + PALETTE_GAP);
                    draw_rectangle(pcx, pcy, PALETTE_SQ, PALETTE_SQ, hex_to_color(hex));
                    if mouse_in_rect(pcx, pcy, PALETTE_SQ, PALETTE_SQ) { draw_rectangle_lines(pcx - 1.0, pcy - 1.0, PALETTE_SQ + 2.0, PALETTE_SQ + 2.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.8)); }
                }
            }

            // Column Line toggle
            let cle_text = format!("Column Line: {}", if config.column_line_enabled { "ON" } else { "OFF" });
            let cle_h = mouse_in_rect(220.0, cle_y - 22.0, 350.0, 24.0);
            if cle_h { draw_rectangle(218.0, cle_y - 22.0, 354.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_text(&cle_text, 220.0, cle_y, if selected_item == LANE_COUNT+13 || cle_h { 24.0 } else { 20.0 }, if selected_item == LANE_COUNT+13 || cle_h { CYAN } else { WHITE_SOFT });

            // Column BG toggle
            let cbe_text = format!("Column BG: {}", if config.column_bg_enabled { "ON" } else { "OFF" });
            let cbe_h = mouse_in_rect(220.0, cbe_y - 22.0, 350.0, 24.0);
            if cbe_h { draw_rectangle(218.0, cbe_y - 22.0, 354.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_text(&cbe_text, 220.0, cbe_y, if selected_item == LANE_COUNT+14 || cbe_h { 24.0 } else { 20.0 }, if selected_item == LANE_COUNT+14 || cbe_h { CYAN } else { WHITE_SOFT });

            let back_cy = cbe_cy + gap + 10.0;
            let back_y = sy(back_cy);
            let back_hovered = mouse_in_rect(cx - 80.0, back_y - 18.0, 160.0, 24.0);
            if back_hovered { draw_rectangle(cx - 85.0, back_y - 18.0, 170.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_menu_item("Back", cx - 50.0, back_y, selected_item == back_idx, back_hovered, 22.0);
        }
        3 => {
            // ── Audio ──
            let ao_cy = start_y;
            let ao_y = sy(ao_cy);
            let ao_text = format!("Audio Offset: {:.0} ms", config.audio_offset_ms);
            let ao_h = mouse_in_rect(220.0, ao_y - 22.0, 350.0, 24.0);
            if ao_h { draw_rectangle(218.0, ao_y - 22.0, 354.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_text(&ao_text, 220.0, ao_y, if selected_item == LANE_COUNT+9 || ao_h { 24.0 } else { 20.0 }, if selected_item == LANE_COUNT+9 || ao_h { CYAN } else { WHITE_SOFT });
            let dh = mouse_in_rect(500.0, ao_y - 18.0, 28.0, 20.0);
            let ih = mouse_in_rect(535.0, ao_y - 18.0, 28.0, 20.0);
            draw_text("<", 508.0, ao_y, if dh { 22.0 } else { 18.0 }, if dh { CYAN } else { DIM });
            draw_text(">", 543.0, ao_y, if ih { 22.0 } else { 18.0 }, if ih { CYAN } else { DIM });

            let back_cy = ao_cy + gap + 10.0;
            let back_y = sy(back_cy);
            let back_hovered = mouse_in_rect(cx - 80.0, back_y - 18.0, 160.0, 24.0);
            if back_hovered { draw_rectangle(cx - 85.0, back_y - 18.0, 170.0, 24.0, Color::new(0.2, 0.7, 1.0, 0.08)); }
            draw_menu_item("Back", cx - 50.0, back_y, selected_item == back_idx, back_hovered, 22.0);
        }
        _ => {}
    }
}

fn handle_settings(selected_item: &mut usize, rebinding_lane: &mut Option<usize>, picking_color: &mut Option<usize>, scroll_y: &mut f32, topic: &mut usize, prev: &mut SettingsPrev, config: &mut GameConfig) -> Option<Screen> {
    let cx = screen_width() / 2.0;
    let gap = 30.0;
    let start_y = 155.0;
    let back_idx = LANE_COUNT + 20;

    // ── Topic tab clicks ──
    let tab_names = ["Controls", "Gameplay", "Visuals", "Audio"];
    let tab_w = 130.0; let tab_h = 30.0; let tab_y = 95.0;
    let total_w = tab_names.len() as f32 * tab_w;
    let start_tab_x = (screen_width() - total_w) / 2.0;

    if is_mouse_button_pressed(MouseButton::Left) {
        for (i, _) in tab_names.iter().enumerate() {
            let tx = start_tab_x + i as f32 * tab_w;
            if mouse_in_rect(tx, tab_y - tab_h, tab_w, tab_h) && *topic != i {
                *topic = i;
                *selected_item = match i {
                    0 => 0,
                    1 => LANE_COUNT,
                    2 => LANE_COUNT + 5,
                    3 => LANE_COUNT + 9,
                    _ => 0,
                };
                *scroll_y = 0.0;
            }
        }
    }

    // ── Mouse wheel scroll ──
    let wheel = mouse_wheel();
    if wheel.1 != 0.0 {
        *scroll_y = (*scroll_y - wheel.1 * 30.0).max(0.0);
    }

    // ── Color picker active ──
    if let Some(pick_target) = *picking_color {
        let ly = if pick_target < LANE_COUNT {
            start_y + pick_target as f32 * 36.0 - *scroll_y
        } else {
            let item_cy = match pick_target {
                100 => start_y + gap * 5.0,
                101 => start_y + gap * 6.0,
                _ => start_y,
            };
            item_cy - *scroll_y
        };
        let pal_x = 470.0; let pal_y = ly - 10.0;
        if is_key_pressed(KeyCode::Escape) { *picking_color = None; return None; }
        let lclick = is_mouse_button_pressed(MouseButton::Left);
        if lclick {
            let mut clicked = false;
            for (idx, &hex) in PALETTE_COLORS.iter().enumerate() {
                let col = idx % PALETTE_COLS; let row = idx / PALETTE_COLS;
                let pcx = pal_x + 5.0 + col as f32 * (PALETTE_SQ + PALETTE_GAP);
                let pcy = pal_y + 5.0 + row as f32 * (PALETTE_SQ + PALETTE_GAP);
                if mouse_in_rect(pcx, pcy, PALETTE_SQ, PALETTE_SQ) {
                    if pick_target < LANE_COUNT {
                        config.note_colors[pick_target] = hex.to_string();
                    } else if pick_target == 100 {
                        config.column_line_color = hex.to_string();
                    } else if pick_target == 101 {
                        config.column_bg_color = hex.to_string();
                    }
                    *picking_color = None; clicked = true; break;
                }
            }
            if !clicked { *picking_color = None; }
        }
        return None;
    }

    // ── Rebinding active ──
    if let Some(lane) = *rebinding_lane {
        if is_key_pressed(KeyCode::Escape) { *rebinding_lane = None; return None; }
        for &(kc, _) in config::ALL_KEYS {
            if is_key_pressed(kc) && kc != KeyCode::Escape {
                for (i, k) in config.key_bindings.iter().enumerate() {
                    if i != lane && *k == kc { config.key_bindings[i] = config.key_bindings[lane]; break; }
                }
                config.key_bindings[lane] = kc; *rebinding_lane = None; return None;
            }
        }
        return None;
    }

    // ── Per-topic content ──
    // Compute content Y chain for current topic
    let (topic_items, content_y_for_idx): (Vec<usize>, Box<dyn Fn(usize) -> f32>) = match *topic {
        0 => {
            let items: Vec<usize> = (0..LANE_COUNT).chain(std::iter::once(back_idx)).collect();
            let cy_fn = move |idx: usize| if idx < LANE_COUNT { start_y + idx as f32 * 36.0 } else { start_y + LANE_COUNT as f32 * 36.0 + gap };
            (items, Box::new(cy_fn))
        }
        1 => {
            let items = vec![LANE_COUNT, LANE_COUNT+1, LANE_COUNT+2, LANE_COUNT+3, LANE_COUNT+4, back_idx];
            let cy_fn = move |idx: usize| match idx {
                x if x == LANE_COUNT => start_y,
                x if x == LANE_COUNT+1 => start_y + gap,
                x if x == LANE_COUNT+2 => start_y + gap*2.0,
                x if x == LANE_COUNT+3 => start_y + gap*3.0,
                x if x == LANE_COUNT+4 => start_y + gap*4.0,
                _ => start_y + gap*5.0 + 10.0,
            };
            (items, Box::new(cy_fn))
        }
        2 => {
            let items = vec![LANE_COUNT+5, LANE_COUNT+6, LANE_COUNT+7, LANE_COUNT+8, LANE_COUNT+10, LANE_COUNT+11, LANE_COUNT+12, LANE_COUNT+13, LANE_COUNT+14, back_idx];
            let cy_fn = move |idx: usize| match idx {
                x if x == LANE_COUNT+5 => start_y,
                x if x == LANE_COUNT+6 => start_y + gap,
                x if x == LANE_COUNT+7 => start_y + gap*2.0,
                x if x == LANE_COUNT+8 => start_y + gap*3.0,
                x if x == LANE_COUNT+10 => start_y + gap*4.0,
                x if x == LANE_COUNT+11 => start_y + gap*5.0,
                x if x == LANE_COUNT+12 => start_y + gap*6.0,
                x if x == LANE_COUNT+13 => start_y + gap*7.0,
                x if x == LANE_COUNT+14 => start_y + gap*8.0,
                _ => start_y + gap*9.0 + 10.0,
            };
            (items, Box::new(cy_fn))
        }
        3 => {
            let items = vec![LANE_COUNT+9, back_idx];
            let cy_fn = move |idx: usize| match idx {
                x if x == LANE_COUNT+9 => start_y,
                _ => start_y + gap + 10.0,
            };
            (items, Box::new(cy_fn))
        }
        _ => (vec![], Box::new(|_| 0.0)),
    };

    // Clamp scroll
    let last_cy = content_y_for_idx(*topic_items.last().unwrap_or(&back_idx));
    let max_scroll = (last_cy + 18.0 - screen_height() + 10.0).max(0.0);
    if *scroll_y > max_scroll { *scroll_y = max_scroll; }
    if *scroll_y < 0.0 { *scroll_y = 0.0; }

    // ── Hover detection ──
    let mut hovered_item: Option<usize> = None;

    if *topic == 0 {
        for lane in 0..LANE_COUNT {
            if mouse_in_rect(220.0, start_y + lane as f32 * 36.0 - *scroll_y - 22.0, 200.0, 24.0) {
                hovered_item = Some(lane);
            }
        }
    }

    // Check hover for each item in current topic (except lanes which are handled above)
    for &item in &topic_items {
        if item < LANE_COUNT { continue; } // already handled
        let cy = content_y_for_idx(item) - *scroll_y;
        let rect_w = if item == back_idx { 160.0 } else if item <= LANE_COUNT+1 { 350.0 } else { 300.0 };
        let mx = if item == back_idx { cx - 80.0 } else { 220.0 };
        if mouse_in_rect(mx, cy - 22.0, rect_w, 24.0) {
            hovered_item = Some(item);
        }
    }

    if let Some(h) = hovered_item { *selected_item = h; }

    // ── Keyboard navigation within topic ──
    let pos = topic_items.iter().position(|&x| x == *selected_item).unwrap_or(0);
    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
        let new_pos = pos.saturating_sub(1);
        *selected_item = topic_items[new_pos];
        let sel_cy = content_y_for_idx(*selected_item);
        if sel_cy - *scroll_y < 0.0 { *scroll_y = (*scroll_y - 30.0).max(0.0); }
    }
    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
        let new_pos = (pos + 1).min(topic_items.len() - 1);
        *selected_item = topic_items[new_pos];
        let sel_cy = content_y_for_idx(*selected_item);
        if sel_cy - *scroll_y > screen_height() { *scroll_y = (*scroll_y + 30.0).min(max_scroll); }
    }

    // ── Per-item value adjustments ──
    let lclick = is_mouse_button_pressed(MouseButton::Left);
    let rclick = is_mouse_button_pressed(MouseButton::Right);

    let check_arrows = |item: usize, dec_x: f32, inc_x: f32| -> (bool, bool, bool, bool, bool) {
        let is_sel = *selected_item == item || hovered_item == Some(item);
        if !is_sel { return (false, false, false, false, false); }
        let cy = content_y_for_idx(item) - *scroll_y;
        let ay = cy - 8.0;
        let inc_h = mouse_in_rect(inc_x, ay - 10.0, 28.0, 20.0);
        let dec_h = mouse_in_rect(dec_x, ay - 10.0, 28.0, 20.0);
        (is_sel, lclick && inc_h, rclick && inc_h, lclick && dec_h, rclick && dec_h)
    };

    // Scroll Speed
    if *topic == 1 {
        let (sel, inc_l, inc_r, dec_l, dec_r) = check_arrows(LANE_COUNT, 440.0, 475.0);
        if sel {
            if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Equal) || inc_l { config.scroll_speed = (config.scroll_speed + 1.0).min(50.0); }
            if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Minus) || dec_l { config.scroll_speed = (config.scroll_speed - 1.0).max(0.5); }
            if inc_r { config.scroll_speed = (config.scroll_speed + 0.1).min(50.0); }
            if dec_r { config.scroll_speed = (config.scroll_speed - 0.1).max(0.5); }
            config.scroll_speed = (config.scroll_speed * 10.0).round() / 10.0;
        }
    }

    // Column Offset
    if *topic == 1 {
        let (sel, inc_l, inc_r, dec_l, dec_r) = check_arrows(LANE_COUNT+1, 475.0, 510.0);
        if sel {
            if (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Equal)) || inc_l { config.column_start = (config.column_start + 10.0).min(500.0); }
            if (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Minus)) || dec_l { config.column_start = (config.column_start - 10.0).max(-500.0); }
            if inc_r { config.column_start = (config.column_start + 1.0).min(500.0); }
            if dec_r { config.column_start = (config.column_start - 1.0).max(-500.0); }
        }
    }

    // Hit Position
    if *topic == 1 {
        let (sel, inc_l, inc_r, dec_l, dec_r) = check_arrows(LANE_COUNT+2, 500.0, 535.0);
        if sel {
            if (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Equal)) || inc_l { config.hit_position = (config.hit_position + 10.0).min(700.0); }
            if (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Minus)) || dec_l { config.hit_position = (config.hit_position - 10.0).max(50.0); }
            if inc_r { config.hit_position = (config.hit_position + 1.0).min(700.0); }
            if dec_r { config.hit_position = (config.hit_position - 1.0).max(50.0); }
        }
    }

    // Scroll Dir toggle
    if *topic == 1 && *selected_item == LANE_COUNT+3 || hovered_item == Some(LANE_COUNT+3) {
        if (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space)) && *selected_item == LANE_COUNT+3
            || (lclick && hovered_item == Some(LANE_COUNT+3))
        { config.scroll_direction = config.scroll_direction.next(); }
    }

    // Scroll Layout toggle
    if *topic == 1 && *selected_item == LANE_COUNT+4 || hovered_item == Some(LANE_COUNT+4) {
        if (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space)) && *selected_item == LANE_COUNT+4
            || (lclick && hovered_item == Some(LANE_COUNT+4))
        { config.scroll_layout = config.scroll_layout.next(); }
    }

    // Note Height
    if *topic == 2 {
        let (sel, inc_l, inc_r, dec_l, dec_r) = check_arrows(LANE_COUNT+5, 500.0, 535.0);
        if sel {
            if (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Equal)) || inc_l { config.note_height = (config.note_height + 5.0).min(80.0); }
            if (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Minus)) || dec_l { config.note_height = (config.note_height - 5.0).max(5.0); }
            if inc_r { config.note_height = (config.note_height + 1.0).min(80.0); }
            if dec_r { config.note_height = (config.note_height - 1.0).max(5.0); }
        }
    }

    // Column Width
    if *topic == 2 {
        let (sel, inc_l, inc_r, dec_l, dec_r) = check_arrows(LANE_COUNT+6, 500.0, 535.0);
        if sel {
            if (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Equal)) || inc_l { config.column_width = (config.column_width + 5.0).min(300.0); }
            if (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Minus)) || dec_l { config.column_width = (config.column_width - 5.0).max(10.0); }
            if inc_r { config.column_width = (config.column_width + 1.0).min(300.0); }
            if dec_r { config.column_width = (config.column_width - 1.0).max(10.0); }
        }
    }

    // LN Shortening
    if *topic == 2 {
        let (sel, inc_l, inc_r, dec_l, dec_r) = check_arrows(LANE_COUNT+7, 500.0, 535.0);
        if sel {
            if (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Equal)) || inc_l { config.ln_shortening_ms = (config.ln_shortening_ms + 5.0).min(500.0); }
            if (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Minus)) || dec_l { config.ln_shortening_ms = (config.ln_shortening_ms - 5.0).max(0.0); }
            if inc_r { config.ln_shortening_ms = (config.ln_shortening_ms + 1.0).min(500.0); }
            if dec_r { config.ln_shortening_ms = (config.ln_shortening_ms - 1.0).max(0.0); }
        }
    }

    // Note Style toggle
    if *topic == 2 && (*selected_item == LANE_COUNT+8 || hovered_item == Some(LANE_COUNT+8)) {
        if (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space)) && *selected_item == LANE_COUNT+8
            || (lclick && hovered_item == Some(LANE_COUNT+8))
        { config.note_style = config.note_style.next(); }
    }

    // Column Spacing
    if *topic == 2 {
        let (sel, inc_l, inc_r, dec_l, dec_r) = check_arrows(LANE_COUNT+10, 500.0, 535.0);
        if sel {
            if (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Equal)) || inc_l { config.column_spacing = (config.column_spacing + 5.0).min(100.0); }
            if (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Minus)) || dec_l { config.column_spacing = (config.column_spacing - 5.0).max(-100.0); }
            if inc_r { config.column_spacing = (config.column_spacing + 1.0).min(100.0); }
            if dec_r { config.column_spacing = (config.column_spacing - 1.0).max(-100.0); }
        }
    }

    // Column Line toggle
    if *topic == 2 && (*selected_item == LANE_COUNT+13 || hovered_item == Some(LANE_COUNT+13)) {
        if (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space)) && *selected_item == LANE_COUNT+13
            || (lclick && hovered_item == Some(LANE_COUNT+13))
        { config.column_line_enabled = !config.column_line_enabled; }
    }

    // Column BG toggle
    if *topic == 2 && (*selected_item == LANE_COUNT+14 || hovered_item == Some(LANE_COUNT+14)) {
        if (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space)) && *selected_item == LANE_COUNT+14
            || (lclick && hovered_item == Some(LANE_COUNT+14))
        { config.column_bg_enabled = !config.column_bg_enabled; }
    }

    // Audio Offset
    if *topic == 3 {
        let (sel, inc_l, inc_r, dec_l, dec_r) = check_arrows(LANE_COUNT+9, 500.0, 535.0);
        if sel {
            if (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Equal)) || inc_l { config.audio_offset_ms = (config.audio_offset_ms + 5.0).min(500.0); }
            if (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Minus)) || dec_l { config.audio_offset_ms = (config.audio_offset_ms - 5.0).max(-500.0); }
            if inc_r { config.audio_offset_ms = (config.audio_offset_ms + 1.0).min(500.0); }
            if dec_r { config.audio_offset_ms = (config.audio_offset_ms - 1.0).max(-500.0); }
        }
    }

    // ── Color swatch clicks for column settings (topic 2) ──
    if *topic == 2 && picking_color.is_none() && rebinding_lane.is_none() {
        let lclick = is_mouse_button_pressed(MouseButton::Left);
        if lclick {
            let cl_y = start_y + gap * 5.0 - *scroll_y;
            if mouse_in_rect(450.0, cl_y - 16.0, 16.0, 16.0) { *picking_color = Some(100); }
            let cb_y = start_y + gap * 6.0 - *scroll_y;
            if mouse_in_rect(450.0, cb_y - 16.0, 16.0, 16.0) { *picking_color = Some(101); }
        }
    }

    // ── Lane rebinding (topic 0 only) ──
    if *topic == 0 {
        for lane in 0..LANE_COUNT {
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                if *selected_item == lane { *rebinding_lane = Some(lane); }
            }
            let lane_y = start_y + lane as f32 * 36.0 - *scroll_y;
            if lclick && rebinding_lane.is_none() && picking_color.is_none()
                && mouse_in_rect(220.0, lane_y - 22.0, 200.0, 24.0)
            {
                *rebinding_lane = Some(lane);
            }
        }
        // Color swatch click
        let lclick = is_mouse_button_pressed(MouseButton::Left);
        if lclick && picking_color.is_none() && rebinding_lane.is_none() {
            for lane in 0..LANE_COUNT {
                let y = start_y + lane as f32 * 36.0 - *scroll_y;
                if mouse_in_rect(450.0, y - 16.0, 16.0, 16.0) { *picking_color = Some(lane); }
            }
        }
    }

    // ── Back button ──
    if *selected_item == back_idx || hovered_item == Some(back_idx) {
        let lclick = is_mouse_button_pressed(MouseButton::Left);
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || (lclick && hovered_item == Some(back_idx)) {
            config::save_config(config);
            return match *prev {
                SettingsPrev::MainMenu => Some(Screen::MainMenu { selected: 0 }),
                SettingsPrev::SongSelect => Some(Screen::SongSelect { selected: 0, songs: loader::scan_song_groups(), scroll_y: 0.0 }),
            };
        }
    }

    None
}

fn draw_results(score: u64, max_combo: u32, perfects: u32, greats: u32, goods: u32, bads: u32, misses: u32, accuracy: f64, total: usize, grade: &str) {
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.85));

    let cx = screen_width() / 2.0;
    let cy = screen_height() / 2.0;

    let title = "Results";
    let ts = measure_text(title, None, 48, 1.0);
    draw_text(title, cx - ts.width / 2.0, cy - 180.0, 48.0, GOLD);

    let lines = [
        format!("Score:     {}", score),
        format!("Max Combo: {}", max_combo),
        format!("Accuracy:  {:.2}%", accuracy * 100.0),
        String::new(),
        format!("Perfect:   {}  Great: {}", perfects, greats),
        format!("Good:      {}  Bad:   {}", goods, bads),
        format!("Miss:      {}", misses),
        String::new(),
        format!("Total:     {}", total),
    ];

    for (i, line) in lines.iter().enumerate() {
        let y = cy - 100.0 + i as f32 * 36.0;
        if line.is_empty() { continue; }
        let ls = measure_text(line, None, 24, 1.0);
        let color = if line.starts_with("Score") { Color::new(0.3, 0.8, 1.0, 1.0) } else { WHITE_SOFT };
        draw_text(line, cx - ls.width / 2.0, y, 24.0, color);
    }

    let gs = measure_text(grade, None, 64, 1.0);
    let gcolor = match grade {
        "SS" => Color::new(1.0, 0.9, 0.0, 1.0),
        "S" => Color::new(1.0, 0.7, 0.0, 1.0),
        "A" => Color::new(0.0, 1.0, 0.5, 1.0),
        "B" => Color::new(0.3, 0.6, 1.0, 1.0),
        "C" => Color::new(1.0, 0.5, 0.0, 1.0),
        _ => Color::new(1.0, 0.2, 0.2, 1.0),
    };
    draw_text(grade, cx - gs.width / 2.0, cy + 160.0, 64.0, gcolor);

    // Button
    let btn_text = "Back to Song Select";
    let btn_w = 250.0;
    let btn_h = 36.0;
    let btn_x = cx - btn_w / 2.0;
    let btn_y = screen_height() - 60.0;
    let btn_hovered = mouse_in_rect(btn_x, btn_y - btn_h / 2.0, btn_w, btn_h);
    draw_rectangle(btn_x, btn_y - btn_h / 2.0, btn_w, btn_h,
        if btn_hovered { Color::new(0.2, 0.7, 1.0, 0.15) } else { Color::new(0.12, 0.12, 0.18, 0.6) });
    draw_rectangle_lines(btn_x, btn_y - btn_h / 2.0, btn_w, btn_h, 1.5,
        if btn_hovered { CYAN } else { Color::new(0.3, 0.3, 0.35, 1.0) });
    let bs = measure_text(btn_text, None, 20, 1.0);
    draw_text(btn_text, cx - bs.width / 2.0, btn_y + 6.0, 20.0,
        if btn_hovered { CYAN } else { WHITE_SOFT });
}

fn handle_results() -> Option<Screen> {
    let cx = screen_width() / 2.0;
    let btn_w = 250.0;
    let btn_h = 36.0;
    let btn_x = cx - btn_w / 2.0;
    let btn_y = screen_height() - 60.0;
    if is_mouse_button_pressed(MouseButton::Left)
        && mouse_in_rect(btn_x, btn_y - btn_h / 2.0, btn_w, btn_h)
    {
        Some(Screen::SongSelect { selected: 0, songs: loader::scan_song_groups(), scroll_y: 0.0 })
    } else {
        None
    }
}

fn draw_pause_menu(resume_timer: f64) {
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.6));

    if resume_timer > 0.0 {
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;
        let secs = (resume_timer.ceil() as u32).min(3);
        let text = if secs > 0 { format!("{}", secs) } else { "GO!".to_string() };
        let ts = measure_text(&text, None, 72, 1.0);
        draw_text(&text, cx - ts.width / 2.0, cy + 24.0, 72.0, CYAN);
        return;
    }

    let cx = screen_width() / 2.0;
    let cy = screen_height() / 2.0;
    let items = ["Continue", "Reset", "Quit"];
    let start_y = cy - 60.0;

    let title = "PAUSED";
    let ts = measure_text(title, None, 40, 1.0);
    draw_text(title, cx - ts.width / 2.0, start_y - 50.0, 40.0, GOLD);

    for (i, item) in items.iter().enumerate() {
        let y = start_y + i as f32 * 52.0;
        let hovered = mouse_in_rect(cx - 100.0, y - 16.0, 200.0, 32.0);
        draw_rectangle(cx - 100.0, y - 16.0, 200.0, 32.0,
            if hovered { Color::new(0.2, 0.7, 1.0, 0.15) } else { Color::new(0.12, 0.12, 0.18, 0.6) });
        draw_rectangle_lines(cx - 100.0, y - 16.0, 200.0, 32.0, 1.0, Color::new(0.3, 0.3, 0.35, 1.0));
        let is = measure_text(item, None, 22, 1.0);
        draw_text(item, cx - is.width / 2.0, y + 8.0, 22.0, if hovered { CYAN } else { WHITE_SOFT });
    }
}

fn handle_paused(state: &mut GameState, audio: &mut AudioEngine, resources: &mut GameResources, paused: &mut bool, resume_timer: &mut f64, config: &GameConfig) -> Option<Screen> {
    if *resume_timer > 0.0 {
        *resume_timer -= get_frame_time() as f64;
        if *resume_timer <= 0.0 {
            *paused = false;
            audio.resume();
        }
        return None;
    }

    if is_key_pressed(KeyCode::Escape) {
        *resume_timer = 3.0;
        return None;
    }

    if !is_mouse_button_pressed(MouseButton::Left) {
        return None;
    }

    let cx = screen_width() / 2.0;
    let cy = screen_height() / 2.0;
    let start_y = cy - 60.0;
    let items = ["Continue", "Reset", "Quit"];

    for (i, item) in items.iter().enumerate() {
        let y = start_y + i as f32 * 52.0;
        if mouse_in_rect(cx - 100.0, y - 16.0, 200.0, 32.0) {
            match *item {
                "Continue" => {
                    *resume_timer = 3.0;
                    return None;
                }
                "Reset" => {
                    let chart = &resources.chart;
                    let new_audio = match &resources.audio_source {
                        AudioSource::Procedural => AudioEngine::new(chart).ok()?,
                        AudioSource::File { path, duration } => AudioEngine::from_file(path, *duration).ok()?,
                    };
                    new_audio.set_volume(config.volume as f32);
                    let new_state = GameState::new(chart, config.clone());
                    *state = new_state;
                    *audio = new_audio;
                    *paused = false;
                    *resume_timer = 0.0;
                    return None;
                }
                "Quit" => {
                    audio.resume();
                    return Some(Screen::SongSelect { selected: 0, songs: loader::scan_song_groups(), scroll_y: 0.0 });
                }
                _ => {}
            }
        }
    }
    None
}

fn handle_main_menu(selected: &mut usize) -> Option<Screen> {
    let cx = screen_width() / 2.0;
    let start_y = 290.0;
    let items = ["Play", "Settings", "Exit"];

    let mut hovered_item: Option<usize> = None;
    for i in 0..=2 {
        let y = start_y + i as f32 * 52.0;
        let size = measure_text(items[i], None, 26, 1.0);
        let text_x = cx - size.width / 2.0 - 30.0;
        if mouse_in_rect(text_x, y - 22.0, size.width + 60.0, 28.0) {
            hovered_item = Some(i);
        }
    }

    if let Some(h) = hovered_item {
        *selected = h;
    }

    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
        *selected = selected.saturating_sub(1);
    }
    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
        *selected = selected.saturating_add(1).min(2);
    }

    let clicked = is_mouse_button_pressed(MouseButton::Left);
    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || (clicked && hovered_item.is_some()) {
        match *selected {
            0 => return Some(Screen::SongSelect { selected: 0, songs: loader::scan_song_groups(), scroll_y: 0.0 }),
            1 => return Some(Screen::Settings { selected_item: LANE_COUNT, rebinding_lane: None, picking_color: None, scroll_y: 0.0, topic: 0, prev: SettingsPrev::MainMenu }),
            2 => return Some(Screen::Exit),
            _ => {}
        }
    }
    None
}

fn handle_song_select(selected: &mut usize, songs: &[loader::SongGroup], scroll_y: &mut f32, config: &GameConfig) -> Option<Screen> {
    let cx = screen_width() / 2.0;
    let start_y = 230.0;
    let max_idx = songs.len() + 1;
    let sh = screen_height();

    // Mouse wheel scroll
    let (_wx, wy) = mouse_wheel();
    if wy != 0.0 {
        let total = start_y + 70.0 + songs.len() as f32 * 36.0 + 30.0 + 30.0;
        let max_scroll = (total - sh + 40.0).max(0.0);
        *scroll_y = (*scroll_y - wy * 30.0).clamp(0.0, max_scroll);
    }

    let mut hovered_item: Option<usize> = None;

    let tut_y = start_y - *scroll_y;
    if mouse_in_rect(cx - 240.0, tut_y - 16.0, 480.0, 24.0) && tut_y > -30.0 && tut_y < sh + 30.0 {
        hovered_item = Some(0);
    }

    for i in 0..songs.len() {
        let y = start_y + 70.0 + i as f32 * 36.0 - *scroll_y;
        if y < -30.0 || y > sh + 30.0 { continue; }
        if mouse_in_rect(cx - 240.0, y - 16.0, 480.0, 24.0) {
            hovered_item = Some(i + 1);
        }
    }

    let list_end = start_y + 70.0 + songs.len() as f32 * 36.0;
    let back_y = list_end + 30.0 - *scroll_y;
    if mouse_in_rect(cx - 80.0, back_y - 18.0, 160.0, 24.0) && back_y > -30.0 && back_y < sh + 30.0 {
        hovered_item = Some(max_idx);
    }

    if let Some(h) = hovered_item {
        *selected = h;
    }

    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
        *selected = selected.saturating_sub(1);
    }
    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
        *selected = selected.saturating_add(1).min(max_idx);
    }
    if is_key_pressed(KeyCode::Escape) {
        return Some(Screen::MainMenu { selected: 0 });
    }

    let clicked = is_mouse_button_pressed(MouseButton::Left);
    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || (clicked && hovered_item.is_some()) {
        if *selected == max_idx {
            return Some(Screen::MainMenu { selected: 0 });
        }
        if *selected == 0 {
            let chart = Chart::generate_tutorial();
            let resources = GameResources {
                chart: chart.clone(),
                audio_source: AudioSource::Procedural,
            };
            let audio = match AudioEngine::new(&chart) {
                Ok(a) => a,
                Err(_) => return None,
            };
            audio.set_volume(config.volume as f32);
            let gs = GameState::new(&chart, config.clone());
            return Some(Screen::Playing { state: gs, audio, resources, paused: false, resume_timer: 0.0 });
        }
        let song = &songs[*selected - 1];
        let diffs = loader::load_difficulties(&song.source);
        if diffs.is_empty() { return None; }
        return Some(Screen::DiffSelect { diffs, selected: 0, scroll_y: 0.0 });
    }

    None
}

fn handle_diff_select(selected: &mut usize, diffs: &[loader::DiffInfo], scroll_y: &mut f32, config: &GameConfig) -> Option<Screen> {
    let cx = screen_width() / 2.0;
    let start_y = 230.0;
    let max_idx = diffs.len();
    let sh = screen_height();

    // Mouse wheel scroll
    let (_wx, wy) = mouse_wheel();
    if wy != 0.0 {
        let total = start_y + diffs.len() as f32 * 40.0 + 40.0 + 30.0;
        let max_scroll = (total - sh + 40.0).max(0.0);
        *scroll_y = (*scroll_y - wy * 30.0).clamp(0.0, max_scroll);
    }

    let mut hovered_item: Option<usize> = None;

    for i in 0..diffs.len() {
        let y = start_y + i as f32 * 40.0 - *scroll_y;
        if y < -30.0 || y > sh + 30.0 { continue; }
        if mouse_in_rect(cx - 120.0, y - 18.0, 240.0, 26.0) {
            hovered_item = Some(i);
        }
    }

    let list_end = start_y + diffs.len() as f32 * 40.0;
    let back_y = list_end + 40.0 - *scroll_y;
    if mouse_in_rect(cx - 80.0, back_y - 18.0, 160.0, 24.0) && back_y > -30.0 && back_y < sh + 30.0 {
        hovered_item = Some(max_idx);
    }

    if let Some(h) = hovered_item {
        *selected = h;
    }

    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
        *selected = selected.saturating_sub(1);
    }
    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
        *selected = selected.saturating_add(1).min(max_idx);
    }
    if is_key_pressed(KeyCode::Escape) {
        return Some(Screen::SongSelect { selected: 0, songs: loader::scan_song_groups(), scroll_y: 0.0 });
    }

    let clicked = is_mouse_button_pressed(MouseButton::Left);
    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || (clicked && hovered_item.is_some()) {
        if *selected == max_idx {
            return Some(Screen::SongSelect { selected: 0, songs: loader::scan_song_groups(), scroll_y: 0.0 });
        }
        let diff = &diffs[*selected];
        let result = match loader::load_chart(diff) {
            Some(r) => r,
            None => { eprintln!("Failed to load chart"); return None; }
        };
        let chart = result.chart;
        let audio_path = result.audio_path;
        let duration = result.duration;
        let resources = GameResources {
            chart: chart.clone(),
            audio_source: AudioSource::File { path: audio_path.clone(), duration },
        };
        let audio = match AudioEngine::from_file(&audio_path, duration) {
            Ok(a) => a,
            Err(e) => { eprintln!("Audio error: {}", e); return None; }
        };
        audio.set_volume(config.volume as f32);
        let gs = GameState::new(&chart, config.clone());
        return Some(Screen::Playing { state: gs, audio, resources, paused: false, resume_timer: 0.0 });
    }

    None
}

fn handle_playing(state: &mut GameState, audio: &mut AudioEngine) -> Option<Screen> {
    let (_wx, wy) = mouse_wheel();
    if wy != 0.0 {
        let step = -wy.signum() as f64 * 0.05;
        state.config.volume = (state.config.volume + step).clamp(0.0, 1.0);
        state.config.volume = (state.config.volume * 20.0).round() / 20.0;
        audio.set_volume(state.config.volume as f32);
    }

    let t_raw = audio.current_time();
    let offset_secs = state.config.audio_offset_ms / 1000.0;
    let t_input = (t_raw + offset_secs).max(0.0);
    let skip = state.handle_input(t_input);
    if skip && state.can_skip(t_input) {
        let target = state.skip_target();
        state.skip_to(target);
        audio.seek(target);
    }

    let t_raw2 = audio.current_time();
    let t = (t_raw2 + offset_secs).max(0.0);
    state.update(t);

    if state.is_finished() {
        let s = state.get_results();
        let acc = s.accuracy();
        let grade = match acc {
            _ if acc >= 0.99 => "SS",
            _ if acc >= 0.95 => "S",
            _ if acc >= 0.90 => "A",
            _ if acc >= 0.80 => "B",
            _ if acc >= 0.70 => "C",
            _ => "D",
        };
        Some(Screen::Results {
            score: s.score,
            max_combo: s.max_combo,
            perfects: s.perfects,
            greats: s.greats,
            goods: s.goods,
            bads: s.bads,
            misses: s.misses,
            accuracy: acc,
            total: s.total_notes,
            grade,
        })
    } else {
        None
    }
}
