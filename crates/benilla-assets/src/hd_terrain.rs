//! Hero Defense: vyskove pole herni plochy (plosiny aren, rampy, skaly mezi arenami).
//!
//! Kazda arena lezi na plosine `PLATEAU_H` yardu nad okolim hradu, z jejiho vychodu (zelene pole
//! mrizky) vede k hradu rampa. Mezi arenami, kolem nadvori a na vnejsim okraji jsou skaly se
//! strmymi srazy, takze se z areny do sousedni pesky nedostane (jen pres rampu a nadvori).
//!
//! Funkce vraci posun nad srovnanou planí (`hd_zone::FLAT_Z`). Plosiny a rampy musi souhlasit se
//! serverem (`hd_defs.h`, `HdArenaGroundOffset`), jinak by nepratele chodili pod terenem nebo nad
//! nim. Skaly server nepotrebuje, nikdo z jeho jednotek na ne nechodi.
//!
//! Prototyp a overeni (rovinnost aren, oddeleni aren, nedostupne vrcholy skal) jsou v
//! `core/tools/terrain` (Python); sum je deterministicky, pocita se v `f64` kvuli shode s nim.

/// Hrad (mapa 1, Kalimdor): poloha a natoceni jako na serveru, viz `hd_zone::CENTER`.
pub(crate) const CX: f64 = -6180.0;
pub(crate) const CY: f64 = -3710.0;
pub(crate) const CO: f64 = 4.7124;
pub(crate) const SLOTS: i32 = 9;

// Rozmery aren, stejne jako na serveru (hd_defs.h).
pub(crate) const FINAL_PATH: f64 = 100.0;
pub(crate) const ARENA_HALF: f64 = 30.0; // 15 poli po 4 yd
pub(crate) const ARENA_FAR: f64 = 200.0; // FINAL_PATH + 25 poli po 4 yd

/// O kolik je plosina areny vys nez okoli hradu (yd). Server: `HD_PLATEAU_HEIGHT`.
pub(crate) const PLATEAU_H: f64 = 5.0;
/// Rovny okraj plosiny kolem mrizky; vrcholy terenu jsou po 4,17 yd.
const MARGIN: f64 = 3.0;
/// Plosina konci aspon tolik pred osou mezi sousednimi arenami (tam stoji skala).
const GAP_HALF: f64 = 3.0;
/// Rampa: polovina sirky a delka. Server: `HD_RAMP_HALF`, `HD_RAMP_LENGTH`.
pub(crate) const RAMP_HALF: f64 = 6.0;
const RAMP_LEN: f64 = 18.0;
/// Volne nadvori kolem hradu (stredni polomer) a polovina sirky cesty bosse.
pub(crate) const COURT_R: f64 = 80.0;
pub(crate) const LANE_HALF: f64 = 18.0;
/// Dal od hradu se teren nemeni.
pub(crate) const OUTER_MAX: f64 = 252.0;
/// Sraz: hned za hranou pochozi plochy stoji skala aspon o tolik yardu nad ni (`CLIFF_MIN` az
/// `CLIFF_MIN + CLIFF_VAR`). Na siti terenu (vrcholy po 4,17 yd a stredy bunek) to dava stenu nad
/// 73 stupnu, kterou postava nevyjde ani nevyskace; mirny nabeh sel preskakat (21. 11. 2026),
/// hlida to `hd_mesh_check`.
const CLIFF_MIN: f64 = 10.0;
const CLIFF_VAR: f64 = 3.0;
/// Rovny lem kolem mrizky areny (yd): trojuhelnik site, ktery do mrizky zasahuje, ma vrcholy az
/// 4,17 yd od ni (delka hrany bunky). Vsechny musi lezet ve vysce plosiny, jinak se krajni pole
/// mrizky zvedaji ke skale. Plosina sama ma okraj jen `MARGIN` = 3 yd, proto lem navic.
const GRID_APRON: f64 = 4.2;

pub(crate) const P_NEAR: f64 = FINAL_PATH - MARGIN;
const P_FAR: f64 = ARENA_FAR + MARGIN;
const P_HALF: f64 = ARENA_HALF + MARGIN;
const RAMP_TOP: f64 = P_NEAR;
pub(crate) const RAMP_BOT: f64 = P_NEAR - RAMP_LEN;

pub(crate) fn step() -> f64 {
    std::f64::consts::TAU / SLOTS as f64
}

pub(crate) fn smooth(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ---- sum -------------------------------------------------------------------------------------

pub(crate) fn hash2(ix: i64, iy: i64, seed: i64) -> f64 {
    let mut h = (ix
        .wrapping_mul(374_761_393)
        .wrapping_add(iy.wrapping_mul(668_265_263))
        .wrapping_add(seed.wrapping_mul(2_147_483_647))) as u32;
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_295.0
}

pub(crate) fn vnoise(x: f64, y: f64, seed: i64) -> f64 {
    let (ix, iy) = (x.floor(), y.floor());
    let (fx, fy) = (x - ix, y - iy);
    let (ix, iy) = (ix as i64, iy as i64);
    let (ux, uy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let a = hash2(ix, iy, seed);
    let b = hash2(ix + 1, iy, seed);
    let c = hash2(ix, iy + 1, seed);
    let d = hash2(ix + 1, iy + 1, seed);
    let top = a + (b - a) * ux;
    top + ((c + (d - c) * ux) - top) * uy
}

pub(crate) fn fbm(mut x: f64, mut y: f64, seed: i64) -> f64 {
    let (mut s, mut amp, mut tot) = (0.0, 1.0, 0.0);
    for o in 0..3 {
        s += vnoise(x, y, seed + o * 17) * amp;
        tot += amp;
        x *= 2.03;
        y *= 2.03;
        amp *= 0.5;
    }
    s / tot
}

// ---- geometrie -------------------------------------------------------------------------------

/// Bod v mistnich souradnicich pozice `slot` v kruhu: (od hradu, doleva).
pub(crate) fn local(x: f64, y: f64, slot: i32) -> (f64, f64) {
    let o = CO + f64::from(slot) * step();
    let (dx, dy) = (x - CX, y - CY);
    (dx * o.cos() + dy * o.sin(), -dx * o.sin() + dy * o.cos())
}

fn rect_dist(f: f64, l: f64, f0: f64, f1: f64, half: f64) -> f64 {
    let df = (f0 - f).max(0.0).max(f - f1);
    let dl = (l.abs() - half).max(0.0);
    df.hypot(dl)
}

fn ramp_h(f: f64) -> f64 {
    PLATEAU_H * smooth((f - RAMP_BOT) / RAMP_LEN)
}

/// Bod lezi v rovnem lemu kolem mrizky nektere areny.
fn in_grid_apron(x: f64, y: f64) -> bool {
    (1..SLOTS).any(|k| {
        let (f, l) = local(x, y, k);
        rect_dist(f, l, FINAL_PATH, ARENA_FAR, ARENA_HALF) <= GRID_APRON
    })
}

/// Bod lezi na rovne zemi: na pochozi plose nebo v lemu kolem mrizky.
#[cfg(test)]
pub(crate) fn on_flat_ground(x: f64, y: f64) -> bool {
    let (d, _) = walk_height(x, y);
    d <= 0.0 || (d <= GRID_APRON && in_grid_apron(x, y))
}

fn court_radius(x: f64, y: f64) -> f64 {
    COURT_R + 8.0 * (vnoise(x / 23.0, y / 23.0, 41) - 0.5)
}

pub(crate) fn outer_radius(x: f64, y: f64) -> f64 {
    237.0 + 22.0 * (fbm(x / 60.0, y / 60.0, 51) - 0.5) + 6.0 * (vnoise(x / 13.0, y / 13.0, 7) - 0.5)
}

fn ridged(x: f64, y: f64, seed: i64) -> f64 {
    1.0 - (2.0 * fbm(x, y, seed) - 1.0).abs()
}

fn rock_top(x: f64, y: f64, d: f64) -> f64 {
    let big = fbm(x / 55.0, y / 55.0, 11);
    let crag = ridged(x / 16.0, y / 16.0, 23);
    8.0 + 15.0 * big * big + 8.0 * crag + 0.3 * d.min(30.0)
}

/// Vzdalenost od nejblizsi pochozi plochy (nadvori, cesta bosse, plosina, rampa) a vyska teto
/// plochy na jejim okraji. Uvnitr pochozi plochy je vzdalenost 0.
pub(crate) fn walk_height(x: f64, y: f64) -> (f64, f64) {
    let (mut best_d, mut best_h) = (1e9, 0.0);
    let r = (x - CX).hypot(y - CY);
    let d = (r - court_radius(x, y)).max(0.0);
    if d < best_d {
        (best_d, best_h) = (d, 0.0);
    }
    let (f, l) = local(x, y, 0);
    let d = rect_dist(
        f,
        l,
        0.0,
        1e9,
        LANE_HALF + 5.0 * (vnoise(x / 21.0, y / 21.0, 31) - 0.3),
    );
    if d < best_d {
        (best_d, best_h) = (d, 0.0);
    }
    let (sin_half, cos_half) = ((step() / 2.0).sin(), (step() / 2.0).cos());
    for k in 1..SLOTS {
        let (f, l) = local(x, y, k);
        let d = rect_dist(f, l, P_NEAR, P_FAR, P_HALF)
            .max(GAP_HALF - (f * sin_half - l * cos_half))
            .max(GAP_HALF - (f * sin_half + l * cos_half));
        if d < best_d {
            (best_d, best_h) = (d, PLATEAU_H);
        }
        let d = rect_dist(f, l, RAMP_BOT - 4.0, P_NEAR, RAMP_HALF);
        if d < best_d {
            (best_d, best_h) = (d, ramp_h(f.clamp(RAMP_BOT, RAMP_TOP)));
        }
    }
    (best_d, best_h)
}

/// Posun terenu nad srovnanou planí v bode (WoW x, y).
pub(crate) fn height(x: f64, y: f64) -> f64 {
    if (x - CX).hypot(y - CY) > OUTER_MAX {
        return 0.0;
    }
    let (d, base) = walk_height(x, y);
    height_with(x, y, d, base)
}

/// `height` pro bod, jehoz `walk_height` (`d`, `base`) uz volajici ma.
pub(crate) fn height_with(x: f64, y: f64, d: f64, base: f64) -> f64 {
    let r = (x - CX).hypot(y - CY);
    if r > OUTER_MAX {
        return 0.0;
    }
    if d <= 0.0 {
        return base;
    }
    // Lem kolem mrizky je ve vysce plosiny (bod v lemu je od plosiny nejvys `GRID_APRON`).
    if d <= GRID_APRON && in_grid_apron(x, y) {
        return PLATEAU_H;
    }
    // Vnejsi okraj skal: kolmy sraz dolu k plani.
    if r > outer_radius(x, y) {
        return 0.0;
    }
    // Nad srazem skala dal stoupa k vrcholum; jak rychle, se meni podle sumu.
    let noise = vnoise(x / 9.0, y / 9.0, 5);
    let wall = 2.6 + 1.8 * noise;
    let h = base + (rock_top(x, y, d) - base) * smooth(d / wall);
    h.max(base + CLIFF_MIN + CLIFF_VAR * noise)
}

/// Jak moc je bod skala (0 pochozi plocha, 1 skala), pro texturu. Klient ji pocita pres
/// `rockiness_with` (`hd_scenery::Site`); tahle podoba overuje shodu s prototypem.
#[cfg(test)]
pub(crate) fn rockiness(x: f64, y: f64) -> f64 {
    let (_, base) = walk_height(x, y);
    rockiness_with(
        height(x, y),
        base,
        (x - CX).hypot(y - CY),
        outer_radius(x, y),
    )
}

/// `rockiness` z hodnot, ktere uz volajici ma: vyska terenu `h`, vyska pochozi plochy `base`,
/// vzdalenost od hradu `r` a vnejsi okraj skal `edge`.
pub(crate) fn rockiness_with(h: f64, mut base: f64, r: f64, edge: f64) -> f64 {
    if r > edge - 4.0 {
        base = 0.0; // pod vnejsim srazem je plan ve vysce hradu
    }
    smooth((h - base - 0.6) / 2.0)
}

/// Vzdalenost bodu od hradu (yd).
pub(crate) fn distance(x: f64, y: f64) -> f64 {
    (x - CX).hypot(y - CY)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Body z prototypu (x, y, vyska, vzdalenost od pochozi plochy, jeji vyska, skalnatost).
    const VECTORS: &str = include_str!("hd_terrain_vectors.txt");

    #[test]
    fn matches_prototype() {
        for line in VECTORS.lines() {
            let v: Vec<f64> = line
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect();
            let (x, y) = (v[0], v[1]);
            let (d, b) = walk_height(x, y);
            assert!((height(x, y) - v[2]).abs() < 1e-6, "height at {x} {y}");
            assert!((d.min(1e6) - v[3]).abs() < 1e-6, "dist at {x} {y}");
            assert!((b - v[4]).abs() < 1e-9, "base at {x} {y}");
            assert!((rockiness(x, y) - v[5]).abs() < 1e-6, "rock at {x} {y}");
        }
    }

    #[test]
    fn arena_is_flat_plateau() {
        // Stred mrizky areny 4 a jeji rohy (s okrajem pol pole) jsou presne na plosine.
        for k in 1..SLOTS {
            for (f, l) in [(102.0, 0.0), (150.0, 28.0), (198.0, -28.0), (101.0, 29.0)] {
                let o = CO + f64::from(k) * step();
                let x = CX + f * o.cos() - l * o.sin();
                let y = CY + f * o.sin() + l * o.cos();
                assert_eq!(height(x, y), PLATEAU_H, "arena {k} ({f}, {l})");
            }
        }
    }
}
