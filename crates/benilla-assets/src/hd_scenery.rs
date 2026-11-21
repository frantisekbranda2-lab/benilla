//! Hero Defense: prostredi herni plochy. Flora na skalach a kolem aren, drobnosti v arenach,
//! povrch zeme (pisek, kaminky, zelen), louze na nadvori a rekvizity kolem nej (tabory, svatyne,
//! ohnove kose u ramp).
//!
//! Nic z toho neni v tabulce: rozmisteni se pocita z vyskoveho pole `hd_terrain` a z hashe
//! polohy, takze je u vsech hracu a pri kazdem spusteni stejne a po zmene terenu se prepocita
//! samo. Server o prostredi nevi a vedet nemusi: zadny z techto modelu nema kolizi
//! (`Doodad::no_collision`), louze jsou melke (hrdina se brodi, neplave) a lezi mimo cesty
//! nepratel, kteri chodi ve vysce, jakou jim pocita server.
//!
//! Modely a textury jsou z dat hry a vsechny uz jsou ve zmensenem klientu (stav 21. 11. 2026),
//! `core/tools/region/trim_client.js` je drzi vyslovne (`core/tools/terrain/scenery_models.json`).

use std::sync::OnceLock;

use crate::hd_rocks::ROCKS;
use crate::hd_terrain as t;

// ---- modely ----------------------------------------------------------------------------------

/// Modely prostredi (Stranglethorn, Durotar, Feralas, Tanaris).
pub(crate) const MODELS: [&str; 59] = [
    "world\\azeroth\\stranglethorn\\passivedoodads\\detail\\strangleplant06.mdx", // 0
    "world\\azeroth\\stranglethorn\\passivedoodads\\detail\\strangleplant07.mdx",
    "world\\azeroth\\stranglethorn\\passivedoodads\\detail\\strangleplant08.mdx",
    "world\\azeroth\\stranglethorn\\passivedoodads\\detail\\strangleplant09.mdx",
    "world\\azeroth\\stranglethorn\\passivedoodads\\detail\\strangleplant10.mdx",
    "world\\azeroth\\stranglethorn\\passivedoodads\\detail\\stranglethornplant01.mdx", // 5
    "world\\azeroth\\stranglethorn\\passivedoodads\\detail\\stranglethornplant02.mdx",
    "world\\azeroth\\stranglethorn\\passivedoodads\\detail\\stranglethornplant05.mdx",
    "world\\kalimdor\\durotar\\passivedoodads\\bushes\\durotarbush01.mdx",
    "world\\kalimdor\\durotar\\passivedoodads\\bushes\\durotarbush02.mdx",
    "world\\kalimdor\\feralas\\passivedoodads\\fern\\feralasferns01.mdx", // 10
    "world\\azeroth\\stranglethorn\\passivedoodads\\detail\\stranglethornfern05.mdx",
    "world\\azeroth\\stranglethorn\\passivedoodads\\detail\\stranglethornfern06.mdx",
    "world\\azeroth\\stranglethorn\\passivedoodads\\detail\\stranglethornfern01.mdx", // 13
    "world\\azeroth\\stranglethorn\\passivedoodads\\detail\\stranglethornfern02.mdx",
    "world\\azeroth\\stranglethorn\\passivedoodads\\detail\\stranglethornfern03.mdx",
    "world\\azeroth\\stranglethorn\\passivedoodads\\detail\\stranglethornfern04.mdx",
    "world\\kalimdor\\durotar\\passivedoodads\\trees\\durotarpalm01.mdx", // 17
    "world\\kalimdor\\durotar\\passivedoodads\\trees\\durotarpalm02.mdx",
    "world\\kalimdor\\durotar\\passivedoodads\\trees\\durotarpalm03.mdx",
    "world\\azeroth\\stranglethorn\\passivedoodads\\trees\\stranglethorntree02\\stranglethorntree02.mdx", // 20
    "world\\kalimdor\\tanaris\\passivedoodads\\rocks\\tanarisrock03.mdx", // 21
    "world\\kalimdor\\tanaris\\passivedoodads\\rocks\\tanarisrock04.mdx",
    "world\\kalimdor\\tanaris\\passivedoodads\\rocks\\tanarisrock05.mdx",
    "world\\kalimdor\\tanaris\\passivedoodads\\rocks\\tanarisrock06.mdx",
    "world\\kalimdor\\tanaris\\passivedoodads\\rocks\\tanarisrock07.mdx",
    "world\\azeroth\\stranglethorn\\passivedoodads\\trollshrine\\stranglethorntikihead.mdx", // 26
    // Rekvizity nadvori: tabor goblinu.
    "world\\generic\\goblin\\passivedoodads\\goblintents\\goblintent01.mdx", // 27
    "world\\generic\\goblin\\passivedoodads\\goblintents\\goblintent03.mdx",
    "world\\generic\\goblin\\passivedoodads\\goblintents\\goblintent06.mdx",
    "world\\azeroth\\elwynn\\passivedoodads\\campfire\\elwynncampfire.mdx", // 30
    "world\\generic\\passivedoodads\\crate01\\crate01.mdx",
    "world\\generic\\passivedoodads\\crate02\\crate02.mdx",
    "world\\generic\\passivedoodads\\barrel\\barrel01.mdx", // 33
    "world\\azeroth\\stranglethorn\\passivedoodads\\trolldungeonsacks\\trolldungeonsacks.mdx",
    // Svatyne trollu.
    "world\\generic\\troll\\passive doodads\\tikimasks\\troll_tikimask01.mdx", // 35
    "world\\generic\\passivedoodads\\barrel\\brokenbarrel01.mdx", // 36
    "world\\generic\\troll\\passive doodads\\tikimasks\\troll_tikimask03.mdx",
    "world\\generic\\troll\\passive doodads\\trollskullpile\\trollskullpile.mdx", // 38
    "world\\generic\\troll\\passive doodads\\trolltablets\\trolltablet.mdx",
    "world\\azeroth\\redridge\\passivedoodads\\rowboat\\rowboat01.mdx", // 40
    "world\\azeroth\\stranglethorn\\passivedoodads\\trolldrum\\trollruinsdrum01.mdx",
    "world\\azeroth\\stranglethorn\\passivedoodads\\trollruinsgong\\trollruinsgong03.mdx",
    "world\\azeroth\\stranglethorn\\passivedoodads\\trolldrum\\trolldungeondrum03.mdx", // 43
    "world\\generic\\orc\\passive doodads\\braziers\\tallbrazier01.mdx", // 44
    "world\\generic\\passivedoodads\\lights\\freestandingtorch01.mdx",
    // Plaz: vrak lodky, rybarska bedna, musle.
    "world\\azeroth\\westfall\\passivedoodads\\wreckedrowboat\\wreckedrowboat.mdx", // 46
    "world\\azeroth\\bootybay\\passivedoodad\\fishingbox\\fishingbox.mdx",
    "world\\generic\\makura\\passive doodads\\makurashells\\makurashells01.mdx", // 48
    "world\\generic\\makura\\passive doodads\\makurashells\\makurashells04.mdx",
    "world\\generic\\makura\\passive doodads\\makurashells\\makurashells07.mdx",
    // Louze: lekniny, rakosi, bahenni rostliny.
    "world\\azeroth\\swamposorrow\\passivedoodads\\plants\\swampofsorrowlilypad01.mdx", // 51
    "world\\azeroth\\swamposorrow\\passivedoodads\\plants\\swampofsorrowlilypad02.mdx",
    "world\\azeroth\\westfall\\passivedoodads\\detail\\westfallreeds01.mdx", // 53
    "world\\azeroth\\swamposorrow\\passivedoodads\\plants\\swampplant03.mdx",
    "world\\azeroth\\swamposorrow\\passivedoodads\\plants\\swampplant05.mdx",
    "world\\azeroth\\stranglethorn\\passivedoodads\\trolldungeonpottery\\trolldungeonpottery01.mdx", // 56
    "world\\azeroth\\stranglethorn\\passivedoodads\\trolldungeonpottery\\trolldungeonpottery02.mdx",
    "world\\azeroth\\stranglethorn\\passivedoodads\\trolldungeonpottery\\trolldungeonpottery03.mdx",
];

/// Palmy na nadvori: s uzkou korunou a nepresahujici meritko, at nezakryvaji vyhled shora.
const COURT_PALMS: [Kind; 2] = [kind(17, 3.7, 0.5, 1.0, 1.25), kind(19, 6.0, 0.5, 0.8, 0.95)];
/// Prvni model rekvizit nadvori v `MODELS` (pred nim flora, kaminky a soska tiki).
const FIRST_PROP: u8 = 27;
/// Ohnovy kos (u ramp a ve shlucich).
const BRAZIER: u8 = 44;

/// Druh objektu: model, polomer pudorysu a zahloubeni pri meritku 1 (yd), rozsah meritka.
#[derive(Clone, Copy)]
struct Kind {
    model: u8,
    radius: f32,
    sink: f32,
    scale: (f32, f32),
}

const fn kind(model: u8, radius: f32, sink: f32, lo: f32, hi: f32) -> Kind {
    Kind {
        model,
        radius,
        sink,
        scale: (lo, hi),
    }
}

/// Palmy (Durotar, stejne rostou v Tanaris) a strom z dzungle Stranglethornu.
const PALMS: [Kind; 3] = [
    kind(17, 3.7, 0.5, 1.0, 1.5),
    kind(18, 6.4, 0.5, 0.9, 1.3),
    kind(19, 6.0, 0.5, 0.9, 1.3),
];
const JUNGLE_TREE: Kind = kind(20, 7.5, 0.6, 0.8, 1.15);
/// Kere a rostliny do pasu az nad hlavu.
const BUSHES: [Kind; 11] = [
    kind(0, 2.5, 0.15, 0.8, 1.3),
    kind(1, 3.9, 0.15, 0.7, 1.1),
    kind(2, 3.3, 0.15, 0.7, 1.1),
    kind(3, 3.9, 0.15, 0.7, 1.1),
    kind(4, 2.5, 0.15, 0.8, 1.3),
    kind(5, 1.7, 0.2, 0.8, 1.3),
    kind(6, 1.6, 0.2, 0.8, 1.3),
    kind(7, 1.5, 0.15, 0.8, 1.3),
    kind(8, 1.6, 0.1, 0.8, 1.3),
    kind(9, 2.4, 0.1, 0.8, 1.2),
    kind(10, 3.1, 0.15, 0.7, 1.1),
];
/// Velke vejire kapradi; jen na skaly a za ne.
const BIG_FERNS: [Kind; 2] = [kind(11, 6.0, 0.0, 0.5, 0.9), kind(12, 6.7, 0.0, 0.5, 0.9)];
/// Nizke kapradi (do 1,3 yd).
const LOW_FERNS: [Kind; 4] = [
    kind(13, 2.4, 0.1, 0.6, 1.1),
    kind(14, 0.6, 0.1, 0.9, 1.6),
    kind(15, 2.7, 0.1, 0.6, 1.0),
    kind(16, 1.6, 0.05, 0.7, 1.2),
];
/// Kaminky: male skaly z Tanaris hodne zmensene (0,4 az 1 yd).
const PEBBLES: [Kind; 5] = [
    kind(21, 1.3, 0.5, 0.22, 0.4),
    kind(22, 0.9, 0.3, 0.28, 0.5),
    kind(23, 0.7, 0.4, 0.28, 0.5),
    kind(24, 0.75, 0.25, 0.3, 0.55),
    kind(25, 0.8, 0.2, 0.3, 0.6),
];
const TIKI: Kind = kind(26, 2.3, 0.4, 1.0, 1.4);

/// Polomery pudorysu modelu skal v poradi `hd_rocks::ROCK_MODELS` (z `doodads.py`).
const ROCK_RADIUS: [f32; 16] = [
    2.2, 1.6, 20.3, 2.4, 2.6, 9.5, 9.8, 7.8, 8.2, 12.7, 9.3, 6.8, 12.8, 7.1, 3.8, 5.7,
];

// ---- rozmery ---------------------------------------------------------------------------------

/// Dal od hradu flora neni: do 270 yd klient schovava objekty mapy, za tim stoji zavodni draha.
const FLORA_MAX: f64 = 264.0;
/// Kaminky a zelen na zemi konci tady (dlazdice s plnym piskem sahaji jiste do 262 yd).
const GROUND_FADE: f64 = 250.0;
const GROUND_END: f64 = 262.0;
/// Pisek: plny do `SAND_FULL` (i se zvlnenim okraje), za `SAND_END` uz zadny.
pub(crate) const SAND_FULL: f64 = 310.0;
pub(crate) const SAND_END: f64 = 354.0;

/// Louze: nejvetsi hloubka doliku a o kolik je hladina pod rovinou nadvori (yd).
const PUDDLE_DEPTH: f64 = 0.55;
pub(crate) const PUDDLE_WATER_BELOW: f64 = 0.16;
const PUDDLE_COUNT: usize = 6;
/// Polovina sirky pruhu, kterym jdou nepratele z rampy k hradu; louze do nej nesahaji.
const PATH_HALF: f64 = 5.0;

/// Rozestup vrcholu terenu (yd) a stred mapy, pro prichyceni louze k vrcholu.
const UNIT: f64 = 1600.0 / 3.0 / 128.0;
const MAP_CENTER: f64 = 32.0 * 1600.0 / 3.0;

const SEED: i64 = 4391;

fn rnd(a: i64, b: i64, k: i64) -> f64 {
    t::hash2(a, b, SEED + k * 131)
}

/// Bod pozice `slot` z mistnich souradnic (od hradu, doleva) do sveta.
fn world(f: f64, l: f64, slot: i32) -> (f64, f64) {
    let o = t::CO + f64::from(slot) * t::step();
    (
        t::CX + f * o.cos() - l * o.sin(),
        t::CY + f * o.sin() + l * o.cos(),
    )
}

// ---- misto -----------------------------------------------------------------------------------

/// Co je v bode za teren.
pub(crate) struct Site {
    /// Vzdalenost od pochozi plochy (0 uvnitr) a vyska teto plochy.
    pub d: f64,
    pub base: f64,
    /// Vyska terenu nad plani.
    pub h: f64,
    /// Vzdalenost od hradu a polomer vnejsiho okraje skal v tomto smeru.
    pub r: f64,
    pub edge: f64,
}

impl Site {
    pub(crate) fn at(x: f64, y: f64) -> Site {
        let (d, base) = t::walk_height(x, y);
        Site {
            d,
            base,
            h: t::height_with(x, y, d, base),
            r: t::distance(x, y),
            edge: t::outer_radius(x, y),
        }
    }

    pub(crate) fn walkable(&self) -> bool {
        self.d <= 0.0
    }

    /// Plan za vnejsim srazem skal (ne cesta bosse, ta je pochozi).
    pub(crate) fn plain(&self) -> bool {
        self.d > 0.0 && self.r >= self.edge
    }

    /// Jak moc je bod skala, stejne jako `hd_terrain::rockiness`.
    pub(crate) fn rockiness(&self) -> f64 {
        t::rockiness_with(self.h, self.base, self.r, self.edge)
    }
}

/// Sklon terenu (prevyseni na yard).
pub(crate) fn slope(x: f64, y: f64) -> f64 {
    const E: f64 = 1.5;
    let gx = (t::height(x + E, y) - t::height(x - E, y)) / (2.0 * E);
    let gy = (t::height(x, y + E) - t::height(x, y - E)) / (2.0 * E);
    gx.hypot(gy)
}

/// Mrizka areny pod bodem: (pozice 1..8, od hradu, doleva).
fn arena_grid_at(x: f64, y: f64) -> Option<(i32, f64, f64)> {
    (1..t::SLOTS).find_map(|k| {
        let (f, l) = t::local(x, y, k);
        (f >= t::FINAL_PATH && f <= t::ARENA_FAR && l.abs() <= t::ARENA_HALF).then_some((k, f, l))
    })
}

/// Kde server stavi postavy u kazde areny (`hd_defs.h`: trh `HD_MARKET_*`, zkusebni postava
/// `HD_TESTER_ARENA_*`), mistne (od hradu, doleva). Do `NPC_CLEAR` yardu od nich nic neroste.
const NPC_SPOTS: [(f64, f64); 2] = [(98.5, 14.0), (72.0, 10.0)];
const NPC_CLEAR: f64 = 4.0;

fn near_npc_spot(x: f64, y: f64) -> bool {
    (1..t::SLOTS).any(|k| {
        let (f, l) = t::local(x, y, k);
        NPC_SPOTS
            .iter()
            .any(|&(sf, sl)| (f - sf).hypot(l - sl) < NPC_CLEAR)
    })
}

/// Bod lezi na rampe nebo v jejim usti.
fn on_ramp(x: f64, y: f64) -> bool {
    (1..t::SLOTS).any(|k| {
        let (f, l) = t::local(x, y, k);
        f >= t::RAMP_BOT - 6.0 && f <= t::P_NEAR + 1.0 && l.abs() <= t::RAMP_HALF + 1.0
    })
}

// ---- povrch zeme -----------------------------------------------------------------------------

/// Sum pro ostrovy na zemi. Hodnotovy sum ma hrany podel sve mrizky (ostrov by mel misty rovny
/// okraj jako podle pravitka), proto se souradnice nejdriv zvlni jinym sumem a pootoci.
fn patch_noise(x: f64, y: f64, scale: f64, seed: i64) -> f64 {
    let wx = x + 9.0 * (t::fbm(x / 21.0, y / 21.0, seed + 5) - 0.5);
    let wy = y + 9.0 * (t::fbm(x / 21.0, y / 21.0, seed + 6) - 0.5);
    let (u, v) = (0.84 * wx - 0.54 * wy, 0.54 * wx + 0.84 * wy);
    t::fbm(u / scale, v / scale, seed)
}

fn ground_fade(r: f64) -> f64 {
    1.0 - t::smooth((r - GROUND_FADE) / (GROUND_END - GROUND_FADE))
}

/// Kolik pisku s kaminky je v bode (0 az 1): ostrovy na pochozich plochach. Skala je v poradi
/// vrstev nad nim, takze na ni videt neni.
pub(crate) fn stones(x: f64, y: f64, s: &Site) -> f64 {
    if s.r >= GROUND_END {
        return 0.0;
    }
    let n = patch_noise(x, y, 17.0, 81);
    0.9 * t::smooth((n - 0.53) / 0.09) * ground_fade(s.r)
}

/// Kolik zelene (porostu) je v bode (0 az 1). Na pochozich plochach ostrovy, v arenach svetlejsi,
/// at je videt mrizka; na skalach nahore a na mirnem sklonu; za skalami lem.
pub(crate) fn green(x: f64, y: f64, s: &Site, slope: f64) -> f64 {
    if s.r >= GROUND_END {
        return 0.0;
    }
    let fade = ground_fade(s.r);
    // Ostrovy na pochozi plose. Na stenu nepokracuji: kde se sit terenu u paty steny zveda,
    // prekryje je skala (`hd_zone::paint_ground`).
    if s.walkable() {
        let n = patch_noise(x, y, 13.0, 71);
        // Nadvori a cesta bosse 0,85, plosina areny 0,5; rampa mezi tim.
        let strength = 0.85 - 0.35 * (s.base / t::PLATEAU_H).clamp(0.0, 1.0);
        return strength * t::smooth((n - 0.6) / 0.06) * fade;
    }
    if s.plain() {
        let n = patch_noise(x, y, 30.0, 72);
        return t::smooth((n - 0.42) / 0.12) * fade;
    }
    // Skala: zelen nahore (ne na pate steny) a kde neni strmo.
    let top = t::smooth((s.h - s.base - 2.5) / 3.0);
    let flat = 1.0 - t::smooth((slope - 0.45) / 0.5);
    let n = patch_noise(x, y, 14.0, 73);
    top * flat * (0.35 + 0.65 * t::smooth((n - 0.3) / 0.25)) * fade
}

/// Kolik pisku je v bode (1 uvnitr, 0 za `SAND_END`), s nepravidelnym okrajem.
pub(crate) fn sand(x: f64, y: f64) -> f64 {
    let r = t::distance(x, y);
    let start = SAND_FULL + 4.0 + 8.0 * (t::vnoise(x / 19.0, y / 19.0, 91) - 0.5);
    1.0 - t::smooth((r - start) / (SAND_END - 8.0 - SAND_FULL))
}

// ---- louze -----------------------------------------------------------------------------------

/// Jeden lalok louze na nadvori: stred a polomer doliku (yd). Louze je hlavni lalok a dva mensi
/// u sousednich vrcholu terenu, at nema tvar ctverce site.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Puddle {
    pub x: f64,
    pub y: f64,
    pub r: f64,
}

/// Souradnice nejblizsiho rohu bunky terenu (po 4,17 yd), at ma louze dno presne na vrcholu.
fn snap_to_corner(v: f64) -> f64 {
    MAP_CENTER - ((MAP_CENTER - v) / UNIT).round() * UNIT
}

/// Posuny laloku od stredu louze v pulkach rozestupu vrcholu: na sousedni roh bunky (4,17 yd)
/// nebo na jeji stred (2,95 yd).
const LOBE_OFFSETS: [(f64, f64); 8] = [
    (2.0, 0.0),
    (-2.0, 0.0),
    (0.0, 2.0),
    (0.0, -2.0),
    (1.0, 1.0),
    (1.0, -1.0),
    (-1.0, 1.0),
    (-1.0, -1.0),
];

/// Lalok lezi cely na nadvori, dal od hradu a mimo cesty nepratel.
fn lobe_fits(p: &Puddle) -> bool {
    let reach = p.r * 1.25 + 2.0;
    // Nejmensi polomer nadvori je 76 yd.
    let r = t::distance(p.x, p.y);
    if r + reach > t::COURT_R - 4.0 || r - reach < 20.0 {
        return false;
    }
    // Mimo cestu bosse (pozice 0) a mimo pruhy, kterymi jdou nepratele z aren k hradu.
    let (f, l) = t::local(p.x, p.y, 0);
    if f > -reach && l.abs() < t::LANE_HALF + 4.0 + reach {
        return false;
    }
    // Dolik saha nejdal 1,25 nasobek polomeru; od osy cesty ma byt aspon `PATH_HALF` a kousek.
    if (1..t::SLOTS).any(|k| {
        let (f, l) = t::local(p.x, p.y, k);
        f > -reach && l.abs() < PATH_HALF + p.r * 1.25 + 0.5
    }) {
        return false;
    }
    // Mimo shluky rekvizit u paty skal, at ma kazda mezera svuj: louze smi lezet pred shlukem
    // smerem k hradu, ne v nem ani vedle nej.
    let (pr, pa) = (r, (p.y - t::CY).atan2(p.x - t::CX));
    (0..GAP_THEMES.len()).all(|gap| {
        let (theta, anchor) = cluster_anchor(gap);
        let (inward, side) = (anchor - pr * (pa - theta).cos(), pr * (pa - theta).sin());
        inward >= CLUSTER_DEPTH + p.r * 1.25 || side.abs() >= CLUSTER_HALF + p.r * 1.25
    })
}

/// Vodni plochy na nadvori: vsechny laloky za sebou a rozsahy laloku jednotlivych ploch. Prvni
/// plocha je laguna, dalsi jsou male louze.
struct Pools {
    lobes: Vec<Puddle>,
    groups: Vec<std::ops::Range<usize>>,
}

/// Mezera mezi osami, ve ktere lezi laguna: pred plazi (`GAP_THEMES`).
const LAGOON_GAP: usize = 3;
/// Laloky laguny: (vzdalenost od hradu, do strany od stredu mezery, polomer), vse v yardech.
/// Laguna je protahla od hradu ke skale a cela lezi v jedne mezere, takze nekrizi cestu
/// nepratel. Hloubka je stejna jako u louzi (`PUDDLE_DEPTH`): brodi se, neplave.
const LAGOON: [(f64, f64, f64); 6] = [
    (36.5, 0.0, 5.0),
    (41.5, 0.5, 6.0),
    (47.0, -0.5, 7.4),
    (52.5, 0.5, 7.5),
    (47.5, 5.0, 4.5),
    (46.5, -5.0, 4.3),
];

fn build_pools() -> Pools {
    let mut out: Vec<Puddle> = Vec::new();
    let mut groups = Vec::new();

    // Laguna: pevny tvar ve sve mezere. Lalok blizko skaly se posune k hradu tak, aby nesahal do
    // plaze; lalok, ktery by presto sahal do cesty, vypadne. K vrcholum terenu se nechytaji, jsou
    // velke a prekryvaji se, vrcholu v nich lezi dost.
    let (theta, anchor) = cluster_anchor(LAGOON_GAP);
    for &(rho, side, r) in &LAGOON {
        let rho = rho.min(anchor - CLUSTER_DEPTH - r * 1.25 - 0.3);
        let p = Puddle {
            x: t::CX + rho * theta.cos() - side * theta.sin(),
            y: t::CY + rho * theta.sin() + side * theta.cos(),
            r,
        };
        if lobe_fits(&p) {
            out.push(p);
        }
    }
    groups.push(0..out.len());

    // Male louze: nahodne, kazda hlavni lalok a dva mensi u sousednich vrcholu terenu.
    let mut count = 0;
    for i in 0..6000 {
        if count == PUDDLE_COUNT {
            break;
        }
        let ang = rnd(i, 1, 1) * std::f64::consts::TAU;
        let rr = 30.0 + rnd(i, 2, 1) * 40.0;
        let r = 4.5 + rnd(i, 3, 1) * 1.5;
        let x = snap_to_corner(t::CX + rr * ang.cos());
        let y = snap_to_corner(t::CY + rr * ang.sin());
        let mut lobes = vec![Puddle { x, y, r }];
        for j in 0..2 {
            let (dx, dy) = LOBE_OFFSETS[((rnd(i, 4 + j, 1) * 8.0) as usize).min(7)];
            lobes.push(Puddle {
                x: x + dx * UNIT / 2.0,
                y: y + dy * UNIT / 2.0,
                r: r * (0.65 + 0.2 * rnd(i, 6 + j, 1)),
            });
        }
        if !lobes.iter().all(lobe_fits) {
            continue;
        }
        // Rozestup od drive polozenych louzi a od laguny.
        if lobes.iter().any(|p| {
            out.iter()
                .any(|q| (p.x - q.x).hypot(p.y - q.y) < (p.r + q.r) * 1.25 + 6.0)
        }) {
            continue;
        }
        let start = out.len();
        out.extend(lobes);
        groups.push(start..out.len());
        count += 1;
    }
    Pools { lobes: out, groups }
}

fn pools() -> &'static Pools {
    static POOLS: OnceLock<Pools> = OnceLock::new();
    POOLS.get_or_init(build_pools)
}

/// Vsechny laloky vody na nadvori (laguna i louze).
pub(crate) fn puddles() -> &'static [Puddle] {
    &pools().lobes
}

/// Vodni plochy po jedne, kazda jako sve laloky. Prvni je laguna.
fn puddle_groups() -> impl Iterator<Item = &'static [Puddle]> {
    let p = pools();
    p.groups.iter().map(move |g| &p.lobes[g.clone()])
}

/// O kolik je teren v bode pod rovinou nadvori (dolik louze), 0 mimo louze.
pub(crate) fn puddle_depth(x: f64, y: f64) -> f64 {
    if t::distance(x, y) > t::COURT_R {
        return 0.0;
    }
    let mut depth: f64 = 0.0;
    for p in puddles() {
        let dist = (x - p.x).hypot(y - p.y);
        if dist >= p.r * 1.25 {
            continue;
        }
        // Nepravidelny breh: polomer se meni podle sumu (0,85 az 1,25 nasobek).
        let rr = p.r * (0.85 + 0.4 * t::vnoise(x / 4.0, y / 4.0, 97));
        depth = depth.max(PUDDLE_DEPTH * t::smooth((1.0 - dist / rr) / 0.7));
    }
    depth
}

// ---- flora -----------------------------------------------------------------------------------

/// Jeden objekt prostredi. Vysku doplni `hd_zone` z terenu, `sink` je zahloubeni pod nej.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Flora {
    pub model: u8,
    pub x: f32,
    pub y: f32,
    pub heading: f32,
    pub tilt_x: f32,
    pub tilt_z: f32,
    pub scale: f32,
    pub sink: f32,
    /// Lezi na hladine louze (leknin), ne na terenu.
    pub on_water: bool,
}

struct Builder {
    out: Vec<Flora>,
    /// Stromy: (x, y, polomer koruny), pro rozestupy.
    trees: Vec<(f64, f64, f64)>,
    /// Poradove cislo kandidata, at ma kazdy vlastni nahodu.
    serial: i64,
}

impl Builder {
    /// Prida objekt druhu `k` v bode; `tilt` je nejvetsi naklon ve stupnich.
    fn put(&mut self, k: Kind, x: f64, y: f64, scale_mul: f64, tilt: f64) {
        if near_npc_spot(x, y) {
            return;
        }
        self.serial += 1;
        let s = self.serial;
        let scale =
            (f64::from(k.scale.0) + rnd(s, 1, 7) * f64::from(k.scale.1 - k.scale.0)) * scale_mul;
        self.out.push(Flora {
            model: k.model,
            x: x as f32,
            y: y as f32,
            heading: (rnd(s, 2, 7) * 360.0) as f32,
            tilt_x: ((rnd(s, 3, 7) - 0.5) * 2.0 * tilt) as f32,
            tilt_z: ((rnd(s, 4, 7) - 0.5) * 2.0 * tilt) as f32,
            scale: scale as f32,
            sink: k.sink * scale as f32,
            on_water: false,
        });
    }

    /// Prida rekvizitu presne: natoceni ve stupnich, meritko, bez nahody a naklonu.
    fn put_prop(&mut self, model: u8, x: f64, y: f64, heading: f64, scale: f32, on_water: bool) {
        if near_npc_spot(x, y) {
            return;
        }
        self.out.push(Flora {
            model,
            x: x as f32,
            y: y as f32,
            heading: heading.rem_euclid(360.0) as f32,
            tilt_x: 0.0,
            tilt_z: 0.0,
            scale,
            sink: 0.05 * scale,
            on_water,
        });
    }

    /// Bod nelezi v modelu skaly (`part` = jaka cast jejiho pudorysu se pocita).
    fn clear_of_rocks(x: f64, y: f64, part: f64, extra: f64) -> bool {
        ROCKS.iter().all(|&(model, rx, ry, _, _, _, scale, _)| {
            let fp = f64::from(ROCK_RADIUS[model as usize] * scale);
            (f64::from(rx) - x).hypot(f64::from(ry) - y) >= fp * part + extra
        })
    }
}

/// Kandidati na pravidelne mrizce s rozestupem `step` a nahodnym posunem, v kruhu `FLORA_MAX`.
fn grid(step: f64, pass: i64, mut each: impl FnMut(i64, i64, f64, f64)) {
    let n = (FLORA_MAX / step).ceil() as i64;
    for a in -n..=n {
        for b in -n..=n {
            let x = t::CX + (a as f64 + (rnd(a, b, pass) - 0.5) * 0.9) * step;
            let y = t::CY + (b as f64 + (rnd(a, b, pass + 1) - 0.5) * 0.9) * step;
            if t::distance(x, y) <= FLORA_MAX {
                each(a, b, x, y);
            }
        }
    }
}

fn pick<const N: usize>(kinds: &[Kind; N], u: f64) -> Kind {
    kinds[((u * N as f64) as usize).min(N - 1)]
}

fn build_flora() -> Vec<Flora> {
    let mut b = Builder {
        out: Vec::new(),
        trees: Vec::new(),
        serial: 0,
    };

    // 1) Stromy: na skalach dal od pochozich ploch (koruna nesmi stinit arenu) a za skalami.
    grid(8.5, 10, |ga, gb, x, y| {
        let s = Site::at(x, y);
        if s.walkable() {
            return;
        }
        let u = rnd(ga, gb, 12);
        let k = if s.plain() {
            if s.r < s.edge + 4.0 || s.d < 10.0 || u > 0.4 {
                return;
            }
            pick(&PALMS, rnd(ga, gb, 13))
        } else {
            if s.r > s.edge - 7.0 || s.h - s.base < 4.0 || slope(x, y) > 0.75 || u > 0.6 {
                return;
            }
            if rnd(ga, gb, 14) < 0.3 {
                JUNGLE_TREE
            } else {
                pick(&PALMS, rnd(ga, gb, 13))
            }
        };
        // Koruna s rezervou neprecniva nad pochozi plochu.
        let crown = f64::from(k.radius * k.scale.1);
        if s.d < crown + 2.0 || !Builder::clear_of_rocks(x, y, 0.6, 1.0) {
            return;
        }
        if b.trees
            .iter()
            .any(|&(tx, ty, tc)| (tx - x).hypot(ty - y) < 0.5 * (tc + crown) + 1.0)
        {
            return;
        }
        b.trees.push((x, y, crown));
        b.put(k, x, y, 1.0, 3.0);
    });

    // 2) Sosky tiki na hranach skal nad arenami a nadvorim.
    grid(13.0, 20, |ga, gb, x, y| {
        let s = Site::at(x, y);
        if s.walkable() || s.plain() || s.d < 4.0 || s.d > 9.0 || s.h - s.base < 5.0 {
            return;
        }
        if rnd(ga, gb, 22) > 0.1 || slope(x, y) > 0.6 || !Builder::clear_of_rocks(x, y, 0.6, 1.0) {
            return;
        }
        b.put(TIKI, x, y, 1.0, 4.0);
    });

    // 3) Podrost na skalach a za nimi.
    grid(5.0, 30, |ga, gb, x, y| {
        let s = Site::at(x, y);
        if s.walkable() {
            return;
        }
        let u = rnd(ga, gb, 32);
        if s.plain() {
            if s.d < 6.0 || u > 0.3 {
                return;
            }
        } else if s.d < 2.0 || s.h - s.base < 2.0 || slope(x, y) > 1.0 || u > 0.42 {
            return;
        }
        if !Builder::clear_of_rocks(x, y, 0.5, 0.0)
            || b.trees
                .iter()
                .any(|&(tx, ty, _)| (tx - x).hypot(ty - y) < 1.5)
        {
            return;
        }
        let v = rnd(ga, gb, 33);
        let k = if v < 0.55 {
            pick(&BUSHES, rnd(ga, gb, 34))
        } else if v < 0.7 && s.d > 6.0 {
            pick(&BIG_FERNS, rnd(ga, gb, 34))
        } else {
            pick(&LOW_FERNS, rnd(ga, gb, 34))
        };
        b.put(k, x, y, 1.0, 6.0);
    });

    // 4) Lem pri pate skal na pochozich plochach (okraj nadvori, okraj plosin mimo mrizku,
    //    kraje cesty bosse). Rampy zustavaji ciste.
    const EDGE: f64 = 2.6;
    grid(3.5, 40, |ga, gb, x, y| {
        let s = Site::at(x, y);
        if !s.walkable() || rnd(ga, gb, 42) > 0.38 || on_ramp(x, y) {
            return;
        }
        let near_wall = (0..8).any(|i| {
            let a = f64::from(i) * std::f64::consts::FRAC_PI_4;
            t::walk_height(x + EDGE * a.cos(), y + EDGE * a.sin()).0 > 0.0
        });
        if !near_wall || arena_grid_at(x, y).is_some() {
            return;
        }
        let v = rnd(ga, gb, 43);
        if v < 0.6 {
            b.put(pick(&LOW_FERNS, rnd(ga, gb, 44)), x, y, 0.9, 5.0);
        } else if v < 0.85 {
            // Jen male kere, at nezakryvaji okraj areny.
            b.put(
                pick(&[BUSHES[7], BUSHES[8], BUSHES[6]], rnd(ga, gb, 44)),
                x,
                y,
                0.7,
                5.0,
            );
        } else {
            b.put(pick(&PEBBLES, rnd(ga, gb, 44)), x, y, 1.2, 12.0);
        }
    });

    // 5) Drobnosti v arenach: v rozich poli mrizky (mezi vezemi), kaminky a nizke kapradi.
    for k in 1..t::SLOTS {
        for row in 0..=25i64 {
            for col in 0..=15i64 {
                let id = i64::from(k) * 1000 + row * 20 + col;
                if rnd(id, 0, 50) > 0.07 {
                    continue;
                }
                let f = t::FINAL_PATH + row as f64 * 4.0 + (rnd(id, 1, 50) - 0.5);
                let l = -t::ARENA_HALF + col as f64 * 4.0 + (rnd(id, 2, 50) - 0.5);
                let (x, y) = world(f, l, k);
                if rnd(id, 3, 50) < 0.6 {
                    b.put(pick(&PEBBLES, rnd(id, 4, 50)), x, y, 1.0, 12.0);
                } else {
                    // Male kapradi: model 14 je drobny, model 16 zmenseny.
                    let fern = if rnd(id, 4, 50) < 0.5 {
                        (LOW_FERNS[1], 0.9)
                    } else {
                        (LOW_FERNS[3], 0.5)
                    };
                    b.put(fern.0, x, y, fern.1, 4.0);
                }
            }
        }
    }

    // 6) Nadvori: par kaminku a kapradi mimo cesty nepratel, hrad a louze.
    grid(6.0, 60, |ga, gb, x, y| {
        let s = Site::at(x, y);
        if !s.walkable() || s.base != 0.0 || s.r < 17.0 || s.r > t::COURT_R - 6.0 {
            return;
        }
        if rnd(ga, gb, 62) > 0.14 || puddle_depth(x, y) > 0.0 {
            return;
        }
        let (f, l) = t::local(x, y, 0);
        if f > 0.0 && l.abs() < t::LANE_HALF {
            return;
        }
        if (1..t::SLOTS).any(|k| {
            let (f, l) = t::local(x, y, k);
            f > 0.0 && l.abs() < PATH_HALF
        }) {
            return;
        }
        if rnd(ga, gb, 63) < 0.5 {
            b.put(pick(&PEBBLES, rnd(ga, gb, 64)), x, y, 1.0, 12.0);
        } else {
            b.put(pick(&LOW_FERNS, rnd(ga, gb, 64)), x, y, 0.7, 4.0);
        }
    });

    // 7) Brehy louzi: par nizkych rostlin.
    for (i, p) in puddles().iter().enumerate() {
        for j in 0..2i64 {
            let id = i as i64 * 10 + j;
            let a = (j as f64 + rnd(id, 0, 70)) / 2.0 * std::f64::consts::TAU;
            let dist = p.r * (1.05 + 0.2 * rnd(id, 1, 70));
            let (x, y) = (p.x + dist * a.cos(), p.y + dist * a.sin());
            if puddle_depth(x, y) > 0.5 * PUDDLE_WATER_BELOW {
                continue; // stala by ve vode (v sousednim laloku)
            }
            b.put(pick(&LOW_FERNS, rnd(id, 2, 70)), x, y, 0.8, 4.0);
        }
    }

    build_courtyard(&mut b);

    b.out
}

// ---- nadvori: tabory, svatyne, ohnove kose u ramp, louze -------------------------------------

/// Rekvizita ve shluku: (model, k hradu yd, do strany yd, natoceni vuci smeru k hradu st., meritko).
type Piece = (u8, f64, f64, f64, f32);

/// Tabor goblinu: stany zady ke skale, pred nimi ohniste, kolem bedny, sudy a pytle.
const CAMP: [Piece; 11] = [
    (27, -1.0, -3.5, 0.0, 1.0),
    (28, 0.0, 6.0, 20.0, 1.0),
    (30, 7.0, 1.0, 0.0, 1.0),
    (31, 2.5, -9.0, 15.0, 1.0),
    (32, 3.6, -10.2, 50.0, 1.0),
    (31, 1.4, -10.4, 80.0, 0.8),
    (33, 1.5, 10.5, 0.0, 1.0),
    (33, 2.7, 11.3, 0.0, 1.0),
    (36, 4.0, 10.0, 70.0, 1.0),
    (34, 4.5, -6.5, 30.0, 1.2),
    (45, 5.5, 6.5, 0.0, 1.2),
];
/// Druhy tabor: jiny stan, veci prohozene na druhou stranu.
const CAMP_B: [Piece; 10] = [
    (29, -1.0, 3.0, 0.0, 1.0),
    (28, 0.5, -6.5, -25.0, 1.0),
    (30, 7.0, -1.0, 0.0, 1.0),
    (32, 2.5, 9.5, -20.0, 1.0),
    (31, 3.6, 10.6, 40.0, 1.0),
    (33, 1.6, -11.0, 0.0, 1.0),
    (36, 3.0, -11.6, 120.0, 1.0),
    (34, 4.5, 7.0, -30.0, 1.2),
    (34, 5.2, 8.0, 60.0, 1.0),
    (45, 5.5, -6.5, 0.0, 1.2),
];
/// Svatyne trollu: soska tiki, tabulky, masky, bubny a gong, lebky, keramika, dva ohnove kose.
/// Masky maji licovou stranu proti tabulkam, proto jsou otocene o pul kruhu.
const SHRINE: [Piece; 15] = [
    (26, 0.0, 0.0, 0.0, 1.3),
    (39, 0.5, 6.0, -15.0, 1.0),
    (39, 0.5, -6.0, 15.0, 1.0),
    (35, 2.2, 3.4, 170.0, 1.0),
    (37, 2.2, -3.4, 190.0, 1.0),
    (44, 6.0, 4.0, 0.0, 1.0),
    (44, 6.0, -4.0, 0.0, 1.0),
    (41, 3.0, -10.0, 20.0, 0.8),
    (43, 5.2, -8.4, 0.0, 1.0),
    (42, 2.0, 10.5, -20.0, 0.8),
    (38, 7.0, 7.0, 0.0, 1.0),
    (38, 6.5, -7.2, 90.0, 1.0),
    (56, 4.6, -1.3, 0.0, 1.3),
    (57, 5.2, 1.0, 40.0, 1.3),
    (58, 4.2, 2.0, 100.0, 1.3),
];
/// Plaz: vrak lodky, rybarska bedna, musle, bedna a sud, pochoden, bahenni rostliny.
const BEACH: [Piece; 11] = [
    (46, 1.0, 0.0, 75.0, 1.0),
    (47, 4.5, 4.5, 30.0, 1.5),
    (48, 5.5, -5.0, 0.0, 0.4),
    (49, 7.0, 2.0, 120.0, 0.35),
    (50, 3.5, 8.0, 200.0, 0.4),
    (48, 7.5, -2.5, 60.0, 0.3),
    (31, 2.0, -7.5, 25.0, 1.0),
    (33, 3.2, -8.3, 0.0, 1.0),
    (45, 5.0, 6.5, 0.0, 1.2),
    (54, -1.5, -5.0, 0.0, 1.0),
    (55, -1.0, 6.0, 0.0, 1.0),
];
/// Co stoji v ktere mezere mezi osami (0 = mezi cestou bosse a arenou 1, 8 = mezi arenou 8 a cestou).
/// Krajni mezery u cesty bosse shluk nemaji, stoji v nich jen palma.
const GAP_THEMES: [&[Piece]; 9] = [
    &[],
    &CAMP,
    &SHRINE,
    &BEACH,
    &CAMP_B,
    &SHRINE,
    &BEACH,
    &CAMP,
    &[],
];

/// Misto, ktere shluk zabira od sve kotvy: k hradu a do stran (yd). Louze do nej nesmi.
const CLUSTER_DEPTH: f64 = 10.0;
const CLUSTER_HALF: f64 = 14.0;
/// Jak daleko pred skalou na okraji nadvori stoji stred shluku (yd).
const CLUSTER_INSET: f64 = 9.0;
/// O kolik stupnu je natoceni modelu posunute proti smeru ve svete (zjisteno na snimku).
const HEADING_OFFSET: f64 = 0.0;

/// Volne misto na nadvori pro rekvizitu: na pochozi plose ve vysce hradu, dal od hradu (kde stoji
/// postavy serveru), mimo cestu bosse, mimo pruhy nepratel k hradu a mimo louze, i s okrajem.
fn court_free(x: f64, y: f64, margin: f64) -> bool {
    let free = |x: f64, y: f64| {
        let (d, base) = t::walk_height(x, y);
        if d > 0.0 || base != 0.0 || puddle_depth(x, y) > 0.0 {
            return false;
        }
        let (f, l) = t::local(x, y, 0);
        if f > 0.0 && l.abs() < t::LANE_HALF + 4.0 {
            return false;
        }
        !(1..t::SLOTS).any(|k| {
            let (f, l) = t::local(x, y, k);
            f > 0.0 && l.abs() < PATH_HALF + 1.0
        })
    };
    t::distance(x, y) >= 32.0
        && free(x, y)
        && (0..4).all(|i| {
            let a = f64::from(i) * std::f64::consts::FRAC_PI_2;
            free(x + margin * a.cos(), y + margin * a.sin())
        })
}

/// Bod shluku: `inward` yardu od kotvy k hradu a `side` yardu do strany (proti smeru hodin).
fn cluster_point(theta: f64, r: f64, inward: f64, side: f64) -> (f64, f64) {
    (
        t::CX + (r - inward) * theta.cos() - side * theta.sin(),
        t::CY + (r - inward) * theta.sin() + side * theta.cos(),
    )
}

/// Kotva shluku v mezere `gap`: (uhel od hradu, vzdalenost). Stred mezery mezi osami, kousek
/// pred skalou; krajni mezery uhybaji od cesty bosse. Louze se kotvam vyhybaji (`lobe_fits`).
fn cluster_anchor(gap: usize) -> (f64, f64) {
    let away = match gap {
        0 => 8.0f64,
        8 => -8.0,
        _ => 0.0,
    };
    let theta = t::CO + (gap as f64 + 0.5) * t::step() + away.to_radians();
    let edge = t::court_radius(t::CX + 80.0 * theta.cos(), t::CY + 80.0 * theta.sin());
    (theta, edge - CLUSTER_INSET)
}

/// Postavi shluk rekvizit v mezere `gap` u paty skaly.
fn place_cluster(b: &mut Builder, gap: usize, pieces: &[Piece]) {
    let (theta, r) = cluster_anchor(gap);
    // Smer k hradu ve stupnich: rekvizity se divaji do nadvori.
    let face = theta.to_degrees() + 180.0 + HEADING_OFFSET;
    for &(model, inward, side, turn, scale) in pieces {
        let (x, y) = cluster_point(theta, r, inward, side);
        // Kus, ktery by stal v ceste nebo mimo nadvori, se vynecha; stejne tak ohnovy kos hned
        // vedle kose u rampy.
        let doubled = model == BRAZIER
            && b.out.iter().any(|f| {
                f.model == BRAZIER && (f64::from(f.x) - x).hypot(f64::from(f.y) - y) < 7.0
            });
        if court_free(x, y, 1.0) && !doubled {
            b.put_prop(model, x, y, face + turn, scale, false);
        }
    }
}

fn build_courtyard(b: &mut Builder) {
    // 8) Ohnove kose po stranach cesty ke kazde rampe.
    for k in 1..t::SLOTS {
        for side in [-8.5, 8.5] {
            let (x, y) = world(68.0, side, k);
            if court_free(x, y, 1.0) {
                b.put_prop(BRAZIER, x, y, 0.0, 1.0, false);
            }
        }
    }

    // 9) Shluky u paty skal mezi rampami: tabory, svatyne, plaze, straz u cesty bosse.
    for (gap, pieces) in GAP_THEMES.iter().enumerate() {
        place_cluster(b, gap, pieces);
    }

    // 10) Voda: lekniny na hladine, rakosi a bahenni rostliny na brehu, musle v pisku kolem.
    for (i, lobes) in puddle_groups().enumerate() {
        let id = i as i64;
        for (j, lobe) in lobes.iter().enumerate() {
            let jd = id * 16 + j as i64;
            // Velky leknin na prvnim laloku, maly na dalsich, kde je dost vody.
            if j == 0 {
                let turn = rnd(id, 0, 80) * 360.0;
                b.put_prop(51, lobe.x + 0.6, lobe.y - 0.4, turn, 0.8, true);
            } else if puddle_depth(lobe.x, lobe.y) > PUDDLE_WATER_BELOW + 0.15 {
                b.put_prop(52, lobe.x, lobe.y, rnd(jd, 0, 81) * 360.0, 1.0, true);
            }
            // U kazdeho laloku rakosi na brehu; u prvniho navic druhe, rostlina a musle.
            let extras: i64 = if j == 0 { 4 } else { 1 };
            for e in 0..extras {
                let a = (e as f64 + rnd(jd, e, 82)) / extras as f64 * std::f64::consts::TAU;
                let Some(shore) = shore_from(lobe, a) else {
                    continue; // timhle smerem je dal voda (sousedni lalok)
                };
                let at = |dist: f64| (lobe.x + dist * a.cos(), lobe.y + dist * a.sin());
                let turn = rnd(jd, e, 83) * 360.0;
                match e {
                    0 | 1 => {
                        let (x, y) = at(shore - 0.2);
                        b.put_prop(53, x, y, turn, 1.0 + 0.3 * rnd(jd, e, 84) as f32, false);
                    }
                    2 => {
                        let (x, y) = at(shore + 0.6);
                        b.put_prop(if i % 2 == 0 { 54 } else { 55 }, x, y, turn, 0.8, false);
                    }
                    _ => {
                        let (x, y) = at(shore + 3.0);
                        if court_free(x, y, 1.0) {
                            b.put_prop([48u8, 49, 50][i % 3], x, y, turn, 0.35, false);
                        }
                    }
                }
            }
        }
    }

    // 11) Lodka vytazena na breh laguny, na konci blizsim k plazi.
    if let Some(lobes) = puddle_groups().next() {
        let theta = t::CO + (LAGOON_GAP as f64 + 0.5) * t::step();
        let far = lobes
            .iter()
            .max_by(|a, b| t::distance(a.x, a.y).total_cmp(&t::distance(b.x, b.y)));
        if let Some((lobe, shore)) = far.and_then(|l| shore_from(l, theta).map(|s| (l, s))) {
            let (x, y) = (
                lobe.x + (shore + 1.5) * theta.cos(),
                lobe.y + (shore + 1.5) * theta.sin(),
            );
            b.put_prop(
                40,
                x,
                y,
                theta.to_degrees() + 115.0 + HEADING_OFFSET,
                0.9,
                false,
            );
        }
    }

    // 12) Palmy: jedna u kazde louze, tri u laguny a po jedne na obou stranach cesty bosse. Dal od
    //     cest a s rozestupem, at koruny nestini nepratele jdouci k hradu.
    let mut palms: Vec<(f64, f64)> = Vec::new();
    for (i, lobes) in puddle_groups().enumerate() {
        let picks: &[usize] = if i == 0 { &[0, 2, 3] } else { &[0] };
        for &j in picks {
            let Some(lobe) = lobes.get(j) else {
                continue;
            };
            let id = i as i64 * 8 + j as i64;
            // Osm smeru tesne u brehu, pak osm o kus dal.
            for attempt in 0..16i64 {
                let a = (rnd(id, 0, 90) + attempt as f64 / 8.0) * std::f64::consts::TAU;
                let dist = lobe.r * 1.25 + if attempt < 8 { 2.5 } else { 5.5 };
                let (x, y) = (lobe.x + dist * a.cos(), lobe.y + dist * a.sin());
                let crowded = palms.iter().any(|&(px, py)| (px - x).hypot(py - y) < 9.0)
                    || b.out.iter().any(|f| {
                        f.model >= FIRST_PROP
                            && (f64::from(f.x) - x).hypot(f64::from(f.y) - y) < 3.5
                    });
                if court_free(x, y, 4.0) && !crowded {
                    palms.push((x, y));
                    b.put(COURT_PALMS[(i + j) % COURT_PALMS.len()], x, y, 1.0, 4.0);
                    break;
                }
            }
        }
    }
    for gap in [0, GAP_THEMES.len() - 1] {
        let (theta, r) = cluster_anchor(gap);
        let (x, y) = cluster_point(theta, r, 1.0, 0.0);
        if court_free(x, y, 4.0) {
            b.put(COURT_PALMS[gap % COURT_PALMS.len()], x, y, 1.0, 4.0);
        }
    }
}

/// Jak daleko od stredu laloku smerem `a` konci voda (teren vystoupi nad hladinu). `None`, kdyz
/// je timhle smerem voda dal nez 12 yd (sousedni laloky laguny).
fn shore_from(lobe: &Puddle, a: f64) -> Option<f64> {
    (0..48).map(|s| f64::from(s) * 0.25).find(|&dist| {
        puddle_depth(lobe.x + dist * a.cos(), lobe.y + dist * a.sin()) < PUDDLE_WATER_BELOW
    })
}

pub(crate) fn flora() -> &'static [Flora] {
    static FLORA: OnceLock<Vec<Flora>> = OnceLock::new();
    FLORA.get_or_init(build_flora)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_tree(model: u8) -> bool {
        (17..=20).contains(&model)
    }

    #[test]
    fn flora_is_plentiful_and_inside_the_cleared_area() {
        let all = flora();
        let trees = all.iter().filter(|f| is_tree(f.model)).count();
        println!(
            "flora: {} objektu, z toho {} stromu; louze: {}",
            all.len(),
            trees,
            puddle_groups().count()
        );
        assert!((700..3500).contains(&all.len()), "objektu {}", all.len());
        assert!(trees >= 150, "stromu {trees}");
        for f in all {
            assert!(t::distance(f64::from(f.x), f64::from(f.y)) <= FLORA_MAX + 0.01);
            assert!((f.model as usize) < MODELS.len());
        }
    }

    #[test]
    fn trees_never_stand_or_hang_over_walkable_ground() {
        let mut court = 0;
        for f in flora().iter().filter(|f| is_tree(f.model)) {
            let (x, y) = (f64::from(f.x), f64::from(f.y));
            let (d, base) = t::walk_height(x, y);
            // Na skale dal od pochozi plochy, nebo palma na nadvori dal od cest a od hradu.
            let on_court = d <= 0.0 && base == 0.0 && court_free(x, y, 3.0);
            court += usize::from(on_court);
            assert!(
                d >= 4.0 || on_court,
                "strom {d} yd od pochozi plochy mimo nadvori"
            );
        }
        println!("palem na nadvori: {court}");
        assert!((4..=16).contains(&court), "palem na nadvori {court}");
    }

    #[test]
    fn arena_grid_holds_only_small_things() {
        // V mrizce areny jen kaminky (21 az 25) a nizke kapradi (13 az 16), zadny vyssi nez asi 1 yd.
        let mut inside = 0;
        for f in flora() {
            if arena_grid_at(f64::from(f.x), f64::from(f.y)).is_none() {
                continue;
            }
            let (d, _) = t::walk_height(f64::from(f.x), f64::from(f.y));
            if d > 0.0 {
                continue;
            }
            inside += 1;
            assert!(
                (13..=16).contains(&f.model) || (21..=25).contains(&f.model),
                "model {} v mrizce areny",
                f.model
            );
        }
        println!("drobnosti v mrizkach aren: {inside}");
        assert!(inside >= 100, "v arenach jen {inside} drobnosti");
    }

    #[test]
    fn courtyard_props_stand_clear_of_paths_and_npcs() {
        let props: Vec<&Flora> = flora().iter().filter(|f| f.model >= FIRST_PROP).collect();
        println!("rekvizit na nadvori: {}", props.len());
        assert!(props.len() >= 80, "rekvizit jen {}", props.len());
        for f in props {
            let (x, y) = (f64::from(f.x), f64::from(f.y));
            // Na nadvori (pochozi plocha ve vysce hradu) a dal od hradu, kde stoji postavy serveru.
            let (d, base) = t::walk_height(x, y);
            assert!(d <= 0.0 && base == 0.0, "model {} mimo nadvori", f.model);
            assert!(
                t::distance(x, y) >= 30.0,
                "model {} moc blizko hradu",
                f.model
            );
            // Mimo cestu bosse a pruhy, kterymi jdou nepratele z ramp k hradu.
            let (fw, l) = t::local(x, y, 0);
            assert!(
                !(fw > 0.0 && l.abs() < t::LANE_HALF),
                "model {} v ceste bosse",
                f.model
            );
            for k in 1..t::SLOTS {
                let (fw, l) = t::local(x, y, k);
                assert!(
                    !(fw > 0.0 && l.abs() < PATH_HALF),
                    "model {} v ceste areny {k}",
                    f.model
                );
            }
            // Ve vode jen lekniny; rakosi a bahenni rostliny (53 az 55) smi stat na brehu.
            if !f.on_water && !(53..=55).contains(&f.model) {
                assert_eq!(puddle_depth(x, y), 0.0, "model {} v louzi", f.model);
            }
        }
        // Kazda rampa ma sve dva ohnove kose.
        let braziers = flora().iter().filter(|f| f.model == BRAZIER).count();
        assert!(braziers >= 16, "ohnovych kosu jen {braziers}");
    }

    #[test]
    fn nothing_grows_where_the_server_stands_its_npcs() {
        for k in 1..t::SLOTS {
            for &(f, l) in &NPC_SPOTS {
                let (x, y) = world(f, l, k);
                for o in flora() {
                    let d = (f64::from(o.x) - x).hypot(f64::from(o.y) - y);
                    assert!(d >= NPC_CLEAR - 0.01, "objekt {d} yd od postavy");
                }
            }
        }
    }

    #[test]
    fn puddles_are_shallow_and_off_the_enemy_paths() {
        let all = puddles();
        println!("louze: {all:?}");
        // Laguna (prvni plocha) ma aspon 5 laloku, louzi je aspon 4.
        let lagoon = puddle_groups().next().unwrap().len();
        assert!(lagoon >= 5, "laguna ma jen {lagoon} laloku");
        assert!(
            puddle_groups().count() >= 5,
            "vodnich ploch {}",
            puddle_groups().count()
        );
        for p in all {
            // Dno je na vrcholu terenu a hlubsi nez hladina, ale melke (plave se az od 0,75 vysky
            // postavy, tedy nad 1 yd).
            let depth = puddle_depth(p.x, p.y);
            assert!(
                depth > PUDDLE_WATER_BELOW + 0.2 && depth <= PUDDLE_DEPTH,
                "{depth}"
            );
            // Cela na nadvori.
            for i in 0..16 {
                let a = f64::from(i) * std::f64::consts::TAU / 16.0;
                let (x, y) = (p.x + p.r * 1.25 * a.cos(), p.y + p.r * 1.25 * a.sin());
                let (d, base) = t::walk_height(x, y);
                assert!(d <= 0.0 && base == 0.0, "louze mimo nadvori");
            }
        }
        // Cesta bosse a pruhy z aren k hradu jsou suche, stejne jako okoli hradu.
        for step in 0..160 {
            let f = f64::from(step) * 0.5;
            for l in [-4.5, -2.0, 0.0, 2.0, 4.5] {
                for k in 0..t::SLOTS {
                    let (x, y) = world(f, l, k);
                    assert_eq!(puddle_depth(x, y), 0.0, "louze v ceste {k} ({f}, {l})");
                }
            }
        }
    }

    #[test]
    fn ground_cover_stays_inside_full_sand() {
        // Kaminky a zelen konci driv, nez muze zacit dlazdice s prechodem pisku (viz `hd_zone`).
        for i in 0..360 {
            let a = f64::from(i).to_radians();
            for r in [GROUND_END, GROUND_END + 10.0, 300.0] {
                let (x, y) = (t::CX + r * a.cos(), t::CY + r * a.sin());
                let s = Site::at(x, y);
                assert_eq!(stones(x, y, &s), 0.0);
                assert_eq!(green(x, y, &s, slope(x, y)), 0.0);
                assert_eq!(sand(x, y), 1.0);
            }
            let (x, y) = (t::CX + SAND_END * a.cos(), t::CY + SAND_END * a.sin());
            assert_eq!(sand(x, y), 0.0);
        }
    }
}
