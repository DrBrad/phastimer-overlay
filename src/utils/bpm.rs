use std::time::{Duration, Instant};

const BPM_PRECISION: usize = 5; // matches JS: const bpm_precision = 5 :contentReference[oaicite:4]{index=4}
const RESET_GAP: Duration = Duration::from_millis(5000);
const MAX_TAPS: usize = 24;

#[derive(Default)]
pub struct TapState {
    taps: Vec<Instant>,
    start_ts: Option<Instant>,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

// JS em = (bm||c?1.15:1.0)*(fm?1.05:1.0) :contentReference[oaicite:5]{index=5}
fn em(blood_moon: bool, forest_minion: bool, coal: bool) -> f64 {
    (if blood_moon || coal { 1.15 } else { 1.0 }) * (if forest_minion { 1.05 } else { 1.0 })
}

// JS bpmToSpeed[2]: x/(1.0*em*(60+x*0.075)) :contentReference[oaicite:6]{index=6}
fn bpm_to_speed_idx2(bpm: f64, blood_moon: bool, forest_minion: bool, coal: bool) -> f64 {
    let e = em(blood_moon, forest_minion, coal);
    bpm / (1.0 * e * (60.0 + bpm * 0.075))
}

// JS get_bpm_average: sum last up to precision, but divides by bpm_precision (NOT by count!) :contentReference[oaicite:7]{index=7}
fn get_bpm_average(avg_taps: &[f64]) -> f64 {
    let mut n = 0.0;
    for (k, v) in avg_taps.iter().rev().enumerate() {
        n += *v;
        if k + 1 >= BPM_PRECISION {
            break;
        }
    }
    n / (BPM_PRECISION as f64)
}

impl TapState {

    pub fn tap_and_compute(&mut self) -> Option<(u32, f64)> {
        let now = Instant::now();

        // JS resets if gap > 5000ms since last tap :contentReference[oaicite:8]{index=8}
        if let Some(prev) = self.taps.last().copied() {
            if now.duration_since(prev) > RESET_GAP {
                self.taps.clear();
                self.start_ts = None;
            }
        }

        self.taps.push(now);

        // JS: if taps.length == 5 start_ts = last tap :contentReference[oaicite:9]{index=9}
        if self.taps.len() == 5 {
            self.start_ts = self.taps.last().copied();
        }

        // JS: if taps.length >= 24 taps.shift() :contentReference[oaicite:10]{index=10}
        if self.taps.len() >= MAX_TAPS {
            self.taps.remove(0);
        }

        // JS builds avg_taps from consecutive intervals:
        // avg_taps.push( round( 60 / (t[i]/1000 - t[i-1]/1000) * 100)/100 ) :contentReference[oaicite:11]{index=11}
        let mut avg_taps: Vec<f64> = Vec::new();
        if self.taps.len() >= 2 {
            for i in 1..self.taps.len() {
                let dt = self.taps[i].duration_since(self.taps[i - 1]).as_secs_f64();
                if dt > 0.0 {
                    let bpm_i = round2(60.0 / dt);
                    avg_taps.push(bpm_i);
                }
            }
        }

        // JS only outputs when avg_taps.length >= 2 (or forced) :contentReference[oaicite:12]{index=12}
        if avg_taps.len() < 2 {
            return None;
        }

        // Same “inaccurate-looking” average as JS (divide by 5 no matter what)
        let mut bpm = get_bpm_average(&avg_taps);

        // JS caps bpm at 600 :contentReference[oaicite:13]{index=13}
        if bpm > 600.0 {
            bpm = 600.0;
        }
        if bpm < 0.0 {
            bpm = 0.0;
        }

        // JS get_ms_exact:
        // speed_idx = calibrating ? 2 : UI; here we hardcode idx=2 (100%)
        // cur_ms = bpmToSpeed[idx](bpm/(1+offset/100), bm && !calibrating, fm && !calibrating, coal && !calibrating) :contentReference[oaicite:14]{index=14}
        let offset_pct = 0.0_f64;
        let blood_moon = false;
        let forest_minion = false;
        let coal = false;

        let bpm_adj = bpm / (1.0 + (offset_pct / 100.0));
        let mut ms = bpm_to_speed_idx2(bpm_adj, blood_moon, forest_minion, coal);

        // JS: return cur_ms < 0 ? 0.01 : cur_ms.toFixed(2) :contentReference[oaicite:15]{index=15}
        if ms < 0.0 {
            ms = 0.01;
        }

        // JS later caps displayed speed to 5.0 :contentReference[oaicite:16]{index=16}
        if ms > 5.0 {
            ms = 5.0;
        }

        // JS displays Math.round(input_bpm) :contentReference[oaicite:17]{index=17}
        let bpm_display = bpm.round().clamp(0.0, 600.0) as u32;
        let ms_display = round2(ms);

        Some((bpm_display, ms_display))
    }
}
