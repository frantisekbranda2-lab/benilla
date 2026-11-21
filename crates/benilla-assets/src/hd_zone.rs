//! Hero Defense: herni plocha kolem hradu bez objektu mapy, se srovnanym terenem, vyvysenymi
//! arenami, skalami mezi nimi a tropickym prostredim (pisek, palmy, podrost, louze).
//!
//! Upravuje se primo nactena dlazdice ADT, takze zmenu prevezme vykresleni, kolize, dotazy na
//! vysku i trava. Kruh kolem hradu je srovnany na vysku plane, za nim teren plynule prechazi
//! zpet do puvodniho. Objekty mapy (doodady a budovy WMO), jejichz stred lezi v kruhu vcetne
//! prechodu, se nekresli. Na srovnanou plan se pricte vyskove pole `hd_terrain` (plosiny aren,
//! rampy, skaly), skaly dostanou skalni texturu Thousand Needles a modely skal a jehel.
//!
//! Prostredi (`hd_scenery`): zem je pisek z Tanaris s ostrovy kaminku a zelene, na skalach a za
//! nimi rostou palmy a podrost ze Stranglethornu, v arenach lezi kaminky a nizke kapradi, na
//! nadvori jsou melke louze. Nic z toho nema kolizi.
//!
//! Server pocita vysku plosin a ramp stejne (`hd_defs.h`, `HdArenaGroundOffset`), skaly ani
//! prostredi nezna.

use benilla_formats::{
    ChunkMesh, Doodad, LiquidKind, LiquidMesh, TileMesh, ALPHA_MAP_SIZE, CHUNK_SIZE,
};

use crate::hd_rocks::{ROCKS, ROCK_MODELS};
use crate::hd_scenery::{self, Site};
use crate::hd_terrain;

/// Hrad (mapa 1, Kalimdor), stejne jako na serveru (HERO_DEFENSE_PLAN.md, cast Mapa).
const CENTER: [f32; 2] = [-6180.0, -3710.0];
/// Vyska srovnane plochy (rovina Shimmering Flats) a vyska hradu na serveru.
const FLAT_Z: f32 = -58.75;
/// Do teto vzdalenosti je teren rovny: areny konci ve 202 yd, cesta bosse ve 240 yd.
const FLAT_RADIUS: f32 = 245.0;
/// Od `FLAT_RADIUS` do teto vzdalenosti teren plynule prechazi do puvodniho.
const BLEND_RADIUS: f32 = 270.0;

/// Textury zeme z dat hry. Vrstvy se v shaderu michaji postupne (`mix` po vrstvach 1 az 3), takze
/// pozdejsi prekryva drivejsi: pisek, na nem kaminky, pres ne skala a nejvys zelen (porost i na
/// vrcholcich skal).
const SAND_TEXTURE: &str = "Tileset\\Tanaris\\TanarisSandBase01.blp";
const STONES_TEXTURE: &str = "Tileset\\Tanaris\\TanarisSandStones.blp";
const ROCK_TEXTURE: &str = "Tileset\\ThousandNeedles\\ThousandNeedlesRock.blp";
const GREEN_TEXTURE: &str = "Tileset\\Feralas\\FeralasBrush.blp";
/// Poradi vrstev plne upraveneho chunku.
const LAYER_SAND: usize = 0;
const LAYER_STONES: usize = 1;
const LAYER_ROCK: usize = 2;
const LAYER_GREEN: usize = 3;
/// Drobnosti na zemi (`GroundEffectTexture.dbc`, modely `World\NoDXT\Detail`, bez kolize): na
/// pisku sem tam kaminek (BarRoc, BlaRoc), na kaminkach sterk a sucha trava (hustota 4), na
/// zeleni trava a kvety ze Stranglethornu (StlGra, StlFlo, hustota 8).
const SAND_EFFECT: u32 = 22849;
const STONES_EFFECT: u32 = 1934;
const GREEN_EFFECT: u32 = 820;
/// MCLY effectId bez travy a drobnosti.
const NO_EFFECT: u32 = 0xFFFF_FFFF;
/// uniqueId nasich objektu: mimo rozsah originalnich objektu, sdileny mezi dlazdicemi.
const ROCK_UNIQUE_BASE: u32 = 0x7D00_0000;
const FLORA_UNIQUE_BASE: u32 = 0x7D10_0000;
/// Krok pro vypocet normal ze sklonu (yd), zhruba polovina rozestupu vrcholu terenu.
const NORMAL_EPS: f64 = 1.5;
/// Barva vody louze: souradnice do vzorniku hloubky (0 pruzracna melcina, 1 hluboka voda) na
/// brehu a kolik pribyva na yard hloubky. Melcina vychazi mlecne bila, proto zacina vys.
const PUDDLE_SHADE: f32 = 0.35;
const PUDDLE_SHADE_PER_YARD: f32 = 1.6;

/// Barva vody louzi pro kameru v bode `wow` na mape `map`: (melka, hluboka) a jejich kryti.
/// `None` mimo herni plochu, tam plati barvy zony.
///
/// Zona Shimmering Flats ma v `Light.dbc` vodu mlecne bilou (zadna v ni neni), louze by vypadaly
/// jako sul. Jina voda v dohledu herni plochy neni, takze se zmena tyka jen louzi.
pub fn puddle_tint(map: u32, wow: [f32; 3]) -> Option<([[f32; 3]; 2], [f32; 2])> {
    const TINT_RADIUS: f32 = 500.0;
    (map == 1 && (wow[0] - CENTER[0]).hypot(wow[1] - CENTER[1]) < TINT_RADIUS)
        .then_some(([[0.30, 0.62, 0.58], [0.08, 0.34, 0.42]], [0.55, 0.9]))
}

/// Co se s dlazdici stalo, pro vypis.
#[derive(Default)]
struct Counts {
    flattened: usize,
    raised: usize,
    painted: usize,
    pools: usize,
}

/// Upravi dlazdici, pokud je to Kalimdor a zasahuje do herni plochy.
pub(crate) fn apply(path: &str, tile: &mut TileMesh) {
    if !path
        .to_ascii_lowercase()
        .replace('\\', "/")
        .contains("kalimdor/kalimdor_")
    {
        return;
    }

    let dist = |p: [f32; 3]| (p[0] - CENTER[0]).hypot(p[1] - CENTER[1]);

    let (doodads, wmos) = (tile.doodads.len(), tile.wmos.len());
    tile.doodads.retain(|d| dist(d.position) > BLEND_RADIUS);
    tile.wmos.retain(|w| dist(w.position) > BLEND_RADIUS);
    let removed = (doodads - tile.doodads.len(), wmos - tile.wmos.len());

    // Plosiny, rampy a skaly, na nadvori doliky louzi.
    let ground = |x: f64, y: f64| hd_terrain::height(x, y) - hd_scenery::puddle_depth(x, y);

    let mut n = Counts::default();
    for chunk in &mut tile.chunks {
        for (i, p) in chunk.positions.iter_mut().enumerate() {
            let d = dist(*p);
            if d >= BLEND_RADIUS {
                continue;
            }
            // 0 v rovne casti, 1 na vnejsim okraji prechodu (smoothstep).
            let t = ((d - FLAT_RADIUS) / (BLEND_RADIUS - FLAT_RADIUS)).clamp(0.0, 1.0);
            let t = t * t * (3.0 - 2.0 * t);
            p[2] = FLAT_Z + (p[2] - FLAT_Z) * t;
            if let Some(nrm) = chunk.normals.get_mut(i) {
                let blended = [nrm[0] * t, nrm[1] * t, nrm[2] * t + (1.0 - t)];
                let len =
                    (blended[0] * blended[0] + blended[1] * blended[1] + blended[2] * blended[2])
                        .sqrt();
                if len > 0.0 {
                    *nrm = [blended[0] / len, blended[1] / len, blended[2] / len];
                }
            }
            n.flattened += 1;

            let (x, y) = (f64::from(p[0]), f64::from(p[1]));
            if hd_terrain::distance(x, y) > hd_terrain::OUTER_MAX + NORMAL_EPS {
                continue;
            }
            let h = ground(x, y);
            let gx = ground(x + NORMAL_EPS, y) - ground(x - NORMAL_EPS, y);
            let gy = ground(x, y + NORMAL_EPS) - ground(x, y - NORMAL_EPS);
            if h == 0.0 && gx == 0.0 && gy == 0.0 {
                continue;
            }
            p[2] += h as f32;
            if let Some(nrm) = chunk.normals.get_mut(i) {
                let (nx, ny) = (-gx / (2.0 * NORMAL_EPS), -gy / (2.0 * NORMAL_EPS));
                let len = (nx * nx + ny * ny + 1.0).sqrt();
                *nrm = [(nx / len) as f32, (ny / len) as f32, (1.0 / len) as f32];
            }
            n.raised += 1;
        }
        if paint_ground(chunk) {
            n.painted += 1;
        }
        if add_puddle_water(chunk) {
            n.pools += 1;
        }
    }

    let rocks = add_rocks(tile);
    let flora = add_flora(tile);

    if n.flattened > 0 || n.painted > 0 || removed != (0, 0) || rocks + flora > 0 {
        bevy::log::info!(
            "hero defense: {path}: srovnano {} bodu terenu, zvednuto {}, povrch na {} chunkach, \
             voda louzi na {}, schovano {} doodadu a {} budov, pridano {rocks} skal a {flora} \
             objektu prostredi",
            n.flattened,
            n.raised,
            n.painted,
            n.pools,
            removed.0,
            removed.1
        );
    }
}

fn to_alpha(w: f64) -> u8 {
    (w.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Prebarvi povrch chunku. Vrati, jestli se chunk zmenil.
///
/// Chunk cely v plnem pisku (`hd_scenery::SAND_FULL`) dostane vlastni ctyri vrstvy: pisek,
/// kaminky, skalu a zelen. Chunk na okraji si puvodni vrstvy necha a pisek v nem jen doznieva:
/// zelen, kaminky ani skala tak daleko nesahaji (`hd_scenery::GROUND_END`, `OUTER_MAX`).
fn paint_ground(chunk: &mut ChunkMesh) -> bool {
    let Some(nw) = chunk.positions.first().copied() else {
        return false;
    };
    let size = f64::from(CHUNK_SIZE);
    let (nx, ny) = (f64::from(nw[0]), f64::from(nw[1]));
    let (cx, cy) = (f64::from(CENTER[0]), f64::from(CENTER[1]));
    // Nejblizsi a nejvzdalenejsi bod chunku od hradu.
    let near = (cx.clamp(nx - size, nx) - cx).hypot(cy.clamp(ny - size, ny) - cy);
    if near > hd_scenery::SAND_END {
        return false;
    }
    let far = [
        (nx, ny),
        (nx - size, ny),
        (nx, ny - size),
        (nx - size, ny - size),
    ]
    .iter()
    .map(|&(x, y)| (x - cx).hypot(y - cy))
    .fold(0.0, f64::max);

    let n = ALPHA_MAP_SIZE as usize;
    let step = size / n as f64;
    if far > hd_scenery::SAND_FULL {
        return paint_sand_edge(chunk, nx, ny, step);
    }

    // Teren v kazdem texelu, s okrajem `M` texelu kolem chunku kvuli sklonu. Texel (tx, ty): tx
    // na vychod (sloupce chunku), ty na jih (radky chunku).
    const M: usize = 3;
    let w = n + 2 * M;
    let at = |g: usize| (g as f64 - M as f64 + 0.5) * step;
    let mut sites = Vec::with_capacity(w * w);
    for gy in 0..w {
        for gx in 0..w {
            sites.push(Site::at(nx - at(gy), ny - at(gx)));
        }
    }
    let h = |gx: usize, gy: usize| sites[gy * w + gx].h;
    let run = 2.0 * M as f64 * step;
    // Vyska site v kazdem texelu nad plani. Stena zacina u vrcholu site, ne presne tam, kde podle
    // vyskoveho pole: bez toho po ni lezou do vysky trojuhelniky pisku.
    let lifted: Vec<f64> = (0..n * n)
        .map(|i| {
            let (x, y) = (nx - at(i / n + M), ny - at(i % n + M));
            chunk_height(chunk, x as f32, y as f32).map_or(0.0, |z| f64::from(z - FLAT_Z))
        })
        .collect();

    let mut alpha = vec![0u8; n * n * 4];
    // Soucet vah vrstev v kazde z 8 x 8 bunek chunku: prevazujici vrstva urci drobnosti na zemi.
    let mut cells = [[0.0f64; 4]; 64];
    for ty in 0..n {
        for tx in 0..n {
            let (gx, gy) = (tx + M, ty + M);
            let s = &sites[gy * w + gx];
            let (x, y) = (nx - at(gy), ny - at(gx));
            let slope = ((h(gx + M, gy) - h(gx - M, gy)) / run)
                .hypot((h(gx, gy + M) - h(gx, gy - M)) / run);
            // Jen kde je teren cely nas (do `FLAT_RADIUS`); dal je v siti puvodni teren mapy.
            let on_mesh = if s.r < f64::from(FLAT_RADIUS) - 1.0 {
                hd_terrain::rockiness_with(lifted[ty * n + tx], s.base, s.r, s.edge)
            } else {
                0.0
            };
            let rock = s.rockiness().max(on_mesh);
            let stones = hd_scenery::stones(x, y, s);
            let mut green = hd_scenery::green(x, y, s, slope);
            if s.walkable() {
                green *= 1.0 - rock; // ostrov zelene nesplha po stene
            }
            let px = &mut alpha[(ty * n + tx) * 4..][..4];
            px[LAYER_STONES - 1] = to_alpha(stones);
            px[LAYER_ROCK - 1] = to_alpha(rock);
            px[LAYER_GREEN - 1] = to_alpha(green);
            px[3] = 255;

            // Co je z vrstev po smichani videt.
            let seen_rock = rock * (1.0 - green);
            let seen_stones = stones * (1.0 - rock) * (1.0 - green);
            let cell = &mut cells[(ty * 8 / n) * 8 + tx * 8 / n];
            cell[LAYER_SAND] += 1.0 - green - seen_rock - seen_stones;
            cell[LAYER_STONES] += seen_stones;
            cell[LAYER_ROCK] += seen_rock;
            cell[LAYER_GREEN] += green;
        }
    }

    chunk.layer_textures = [SAND_TEXTURE, STONES_TEXTURE, ROCK_TEXTURE, GREEN_TEXTURE]
        .iter()
        .map(|t| t.to_string())
        .collect();
    chunk.layer_effect_ids = vec![SAND_EFFECT, STONES_EFFECT, NO_EFFECT, GREEN_EFFECT];
    chunk.base_texture = Some(SAND_TEXTURE.to_string());
    chunk.alpha_map = Some(alpha);
    // Zapecene stiny patrily objektum mapy, ktere tu uz nejsou.
    chunk.shadow = None;
    for (k, cell) in cells.iter().enumerate() {
        let layer = (0..4)
            .max_by(|&a, &b| cell[a].total_cmp(&cell[b]))
            .unwrap_or(LAYER_SAND);
        chunk.pred_tex[k] = layer as u8;
        // Na skale nic neroste; v louzi take ne.
        let (row, col) = (k / 8, k % 8);
        let (x, y) = (
            nx - (row as f64 + 0.5) * size / 8.0,
            ny - (col as f64 + 0.5) * size / 8.0,
        );
        chunk.no_effect_doodad[k] = layer == LAYER_ROCK || hd_scenery::puddle_depth(x, y) > 0.0;
    }
    true
}

/// Okraj piskove plochy: pisek prekryje jednu puvodni vrstvu a smerem ven se vytraci.
fn paint_sand_edge(chunk: &mut ChunkMesh, nx: f64, ny: f64, step: f64) -> bool {
    let n = ALPHA_MAP_SIZE as usize;
    let mut sand = vec![0u8; n * n];
    let mut any = false;
    for ty in 0..n {
        for tx in 0..n {
            let x = nx - (ty as f64 + 0.5) * step;
            let y = ny - (tx as f64 + 0.5) * step;
            let w = to_alpha(hd_scenery::sand(x, y));
            any |= w > 0;
            sand[ty * n + tx] = w;
        }
    }
    if !any {
        return false;
    }

    let mut alpha = chunk.alpha_map.take().unwrap_or_else(|| {
        let mut map = vec![0u8; n * n * 4];
        for px in map.chunks_exact_mut(4) {
            px[3] = 255;
        }
        map
    });
    // Pisek do prvni volne vrstvy; plny chunk obetuje tu, ktere je v nem nejmin.
    let slot = if chunk.layer_textures.len() < 4 {
        chunk.layer_textures.len().max(1)
    } else {
        (1..4)
            .min_by_key(|&l| {
                alpha
                    .chunks_exact(4)
                    .map(|px| u32::from(px[l - 1]))
                    .sum::<u32>()
            })
            .unwrap_or(3)
    };
    set_layer(chunk, slot, SAND_TEXTURE);
    for (i, &w) in sand.iter().enumerate() {
        let px = &mut alpha[i * 4..][..4];
        px[slot - 1] = w;
        // Vrstvy nad piskem by ho prekryly: pod plnym piskem musi zmizet, at okraj navazuje.
        for later in px.iter_mut().take(3).skip(slot) {
            *later = (u32::from(*later) * u32::from(255 - w) / 255) as u8;
        }
    }
    chunk.alpha_map = Some(alpha);
    true
}

/// Nastavi vrstve `slot` texturu (vrstvu prida, kdyz chybi), bez travy a drobnosti.
fn set_layer(chunk: &mut ChunkMesh, slot: usize, texture: &str) {
    while chunk.layer_textures.len() <= slot {
        chunk.layer_textures.push(texture.to_string());
    }
    chunk.layer_textures[slot] = texture.to_string();
    chunk
        .layer_effect_ids
        .resize(chunk.layer_textures.len(), NO_EFFECT);
    chunk.layer_effect_ids[slot] = NO_EFFECT;
    if slot == 0 {
        chunk.base_texture = Some(texture.to_string());
    }
}

/// Hladina louze: kde teren chunku klesa pod hladinu, prida vodni plochu. Vrati, jestli nejakou
/// pridal.
///
/// Hladina lezi kousek pod rovinou nadvori, takze je videt jen v doliku a breh tvori teren sam.
/// Voda je melka: postava se v ni brodi, plavat zacina az v hloubce 0,75 sve vysky.
fn add_puddle_water(chunk: &mut ChunkMesh) -> bool {
    if chunk.positions.len() < 145 {
        return false;
    }
    let nw = chunk.positions[0];
    let size = f64::from(CHUNK_SIZE);
    let (nx, ny) = (f64::from(nw[0]), f64::from(nw[1]));
    let (cx, cy) = (f64::from(CENTER[0]), f64::from(CENTER[1]));
    let near = (cx.clamp(nx - size, nx) - cx).hypot(cy.clamp(ny - size, ny) - cy);
    if near > hd_terrain::COURT_R {
        return false; // louze jsou jen na nadvori
    }

    let level = FLAT_Z - hd_scenery::PUDDLE_WATER_BELOW as f32;
    let under = |i: usize| chunk.positions[i][2] < level - 0.02;
    let mut wet = vec![false; 64];
    let mut indices = Vec::new();
    for row in 0..8usize {
        for col in 0..8usize {
            // Rohy bunky a jeji stred (MCVT po 17: roh `r * 17 + c`, stred `r * 17 + 9 + c`).
            let verts = [
                row * 17 + col,
                row * 17 + col + 1,
                (row + 1) * 17 + col,
                (row + 1) * 17 + col + 1,
                row * 17 + 9 + col,
            ];
            if !verts.iter().any(|&i| under(i)) {
                continue;
            }
            wet[row * 8 + col] = true;
            let tl = (row * 9 + col) as u32;
            let bl = ((row + 1) * 9 + col) as u32;
            indices.extend_from_slice(&[tl, bl, bl + 1, tl, bl + 1, tl + 1]);
        }
    }
    if indices.is_empty() {
        return false;
    }

    let (mut positions, mut uvs, mut depths) = (Vec::new(), Vec::new(), Vec::new());
    for row in 0..9usize {
        for col in 0..9usize {
            let p = chunk.positions[row * 17 + col];
            positions.push([p[0], p[1], level]);
            uvs.push([col as f32 * 0.25, row as f32 * 0.25]);
            depths.push((PUDDLE_SHADE + (level - p[2]).max(0.0) * PUDDLE_SHADE_PER_YARD).min(1.0));
        }
    }
    chunk.liquids.push(LiquidMesh {
        grid: [9, 9],
        shared: vec![false; wet.len()],
        wet,
        positions,
        uvs,
        depths,
        indices,
        sound_nibble: 0, // jezero
        material_id: None,
        kind: LiquidKind::Still,
    });
    true
}

/// Rozsah dlazdice ve WoW souradnicich: (min x, max x, min y, max y).
fn tile_bounds(tile: &TileMesh) -> (f32, f32, f32, f32) {
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for p in tile.chunks.iter().flat_map(|c| c.positions.iter()) {
        min_x = min_x.min(p[0]);
        max_x = max_x.max(p[0]);
        min_y = min_y.min(p[1]);
        max_y = max_y.max(p[1]);
    }
    (min_x, max_x, min_y, max_y)
}

/// Prida modely skal, jejichz stred lezi v teto dlazdici. Vrati jejich pocet.
fn add_rocks(tile: &mut TileMesh) -> usize {
    let (min_x, max_x, min_y, max_y) = tile_bounds(tile);
    let mut added = 0;
    for (i, &(model, x, y, heading, tilt_x, tilt_z, scale, sink)) in ROCKS.iter().enumerate() {
        // Kazda skala patri prave jedne dlazdici (polootevrene hranice).
        if !(x >= min_x && x < max_x && y >= min_y && y < max_y) {
            continue;
        }
        let z = FLAT_Z + hd_terrain::height(f64::from(x), f64::from(y)) as f32 - sink;
        tile.doodads.push(Doodad {
            model: ROCK_MODELS[model as usize].to_string(),
            position: [x, y, z],
            rotation: [tilt_x, heading, tilt_z],
            scale,
            unique_id: ROCK_UNIQUE_BASE + i as u32,
            no_collision: false,
        });
        added += 1;
    }
    added
}

/// Prida objekty prostredi (stromy, podrost, kaminky), jejichz stred lezi v teto dlazdici, na
/// vysku vykresleneho terenu. Zadny nema kolizi. Vrati jejich pocet.
fn add_flora(tile: &mut TileMesh) -> usize {
    let (min_x, max_x, min_y, max_y) = tile_bounds(tile);
    let mut added = 0;
    for (i, f) in hd_scenery::flora().iter().enumerate() {
        if !(f.x >= min_x && f.x < max_x && f.y >= min_y && f.y < max_y) {
            continue;
        }
        let Some(z) = mesh_height(&tile.chunks, f.x, f.y) else {
            continue;
        };
        tile.doodads.push(Doodad {
            model: hd_scenery::MODELS[f.model as usize].to_string(),
            // Lekniny lezi na hladine louze, ostatni stoji na terenu.
            position: [
                f.x,
                f.y,
                if f.on_water {
                    FLAT_Z - hd_scenery::PUDDLE_WATER_BELOW as f32 + 0.03
                } else {
                    z - f.sink
                },
            ],
            rotation: [f.tilt_x, f.heading, f.tilt_z],
            scale: f.scale,
            unique_id: FLORA_UNIQUE_BASE + i as u32,
            no_collision: true,
        });
        added += 1;
    }
    added
}

/// Vyska vykresleneho terenu v bode: bunka je ctyri trojuhelniky kolem sveho stredu, stejne jako
/// ji klient kresli (a jako rozmistuje travu, `benilla_formats::ground_effects`).
fn mesh_height(chunks: &[ChunkMesh], x: f32, y: f32) -> Option<f32> {
    chunks.iter().find_map(|chunk| chunk_height(chunk, x, y))
}

/// `mesh_height` v jednom chunku; `None`, kdyz bod lezi mimo nej.
fn chunk_height(chunk: &ChunkMesh, x: f32, y: f32) -> Option<f32> {
    if chunk.positions.len() < 145 {
        return None;
    }
    let unit = CHUNK_SIZE / 8.0;
    let nw = chunk.positions[0];
    // Na jih (-X) a na vychod (-Y) od severozapadniho rohu chunku.
    let (south, east) = (nw[0] - x, nw[1] - y);
    if !(0.0..CHUNK_SIZE).contains(&south) || !(0.0..CHUNK_SIZE).contains(&east) {
        return None;
    }
    let (row, col) = (
        ((south / unit) as usize).min(7),
        ((east / unit) as usize).min(7),
    );
    let (fy, fx) = (south / unit - row as f32, east / unit - col as f32);
    let z = |i: usize| chunk.positions[i][2];
    let (tl, tr) = (z(row * 17 + col), z(row * 17 + col + 1));
    let (bl, br) = (z((row + 1) * 17 + col), z((row + 1) * 17 + col + 1));
    let ctr = z(row * 17 + 9 + col);
    // Vahy (stred, prvni roh, druhy roh) podle toho, ve kterem ze ctyr trojuhelniku bod je.
    let (u, v, w, va, vb) = if fx + fy <= 1.0 {
        if fy <= fx {
            (2.0 * fy, 1.0 - fx - fy, fx - fy, tl, tr)
        } else {
            (2.0 * fx, fy - fx, 1.0 - fx - fy, bl, tl)
        }
    } else if fy >= fx {
        (2.0 * (1.0 - fy), fx + fy - 1.0, fy - fx, br, bl)
    } else {
        (2.0 * (1.0 - fx), fx - fy, fx + fy - 1.0, tr, br)
    };
    Some(u * ctr + v * va + w * vb)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chunk 9 x 9 rohu a 8 x 8 stredu se severozapadnim rohem v (nx, ny) a vyskou `z(x, y)`.
    fn chunk(nx: f32, ny: f32, z: impl Fn(f32, f32) -> f32) -> ChunkMesh {
        let unit = CHUNK_SIZE / 8.0;
        let mut positions = Vec::new();
        for row in 0..9 {
            for col in 0..9 {
                let (x, y) = (nx - row as f32 * unit, ny - col as f32 * unit);
                positions.push([x, y, z(x, y)]);
            }
            if row < 8 {
                for col in 0..8 {
                    let (x, y) = (
                        nx - (row as f32 + 0.5) * unit,
                        ny - (col as f32 + 0.5) * unit,
                    );
                    positions.push([x, y, z(x, y)]);
                }
            }
        }
        ChunkMesh {
            normals: vec![[0.0, 0.0, 1.0]; positions.len()],
            positions,
            uvs: Vec::new(),
            indices: Vec::new(),
            holes: 0,
            base_texture: None,
            layer_textures: Vec::new(),
            layer_effect_ids: Vec::new(),
            alpha_map: None,
            shadow: None,
            pred_tex: [0; 64],
            no_effect_doodad: [false; 64],
            index_x: 0,
            index_y: 0,
            area_id: 0,
            impassable: false,
            liquids: Vec::new(),
        }
    }

    #[test]
    fn mesh_height_follows_a_tilted_plane() {
        // Rovina je na trojuhelnicich presna: vyska site se rovna vysce roviny.
        let plane = |x: f32, y: f32| 0.25 * x - 0.5 * y + 3.0;
        let c = [chunk(100.0, 200.0, plane)];
        for (x, y) in [(99.0, 199.5), (80.3, 170.2), (67.0, 199.9), (90.0, 180.0)] {
            let z = mesh_height(&c, x, y).unwrap();
            assert!((z - plane(x, y)).abs() < 1e-3, "({x}, {y}): {z}");
        }
        assert!(mesh_height(&c, 101.0, 199.0).is_none());
    }

    #[test]
    fn courtyard_chunk_gets_scenery_layers_and_puddle_water() {
        // Chunk na nadvori pod prvni louzi: rovina hradu, po uprave s dolikem a vodou.
        let p = hd_scenery::puddles()[0];
        let (nx, ny) = (p.x as f32 + CHUNK_SIZE / 2.0, p.y as f32 + CHUNK_SIZE / 2.0);
        let mut c = chunk(nx, ny, |_, _| FLAT_Z);
        for v in &mut c.positions {
            v[2] -= hd_scenery::puddle_depth(f64::from(v[0]), f64::from(v[1])) as f32;
        }
        assert!(paint_ground(&mut c));
        assert_eq!(c.layer_textures.len(), 4);
        assert_eq!(c.layer_textures[LAYER_SAND], SAND_TEXTURE);
        assert_eq!(c.layer_effect_ids[LAYER_ROCK], NO_EFFECT);
        assert!(add_puddle_water(&mut c));
        let water = &c.liquids[0];
        let wet = water.wet.iter().filter(|w| **w).count();
        assert!((1..=64).contains(&wet), "mokrych bunek {wet}");
        assert_eq!(water.indices.len(), wet * 6);
        // Hladina je pod rovinou nadvori a nejvys 0,45 yd nad dnem (brodi se, neplave).
        let level = water.positions[0][2];
        assert!(level < FLAT_Z);
        let bottom = c.positions.iter().map(|v| v[2]).fold(f32::MAX, f32::min);
        assert!(
            level - bottom > 0.2 && level - bottom < 0.45,
            "{}",
            level - bottom
        );
        // V louzi nerostou drobnosti.
        assert!(c.no_effect_doodad.iter().any(|b| *b));
    }

    #[test]
    fn far_chunk_is_left_alone_and_edge_chunk_keeps_its_layers() {
        let far = hd_scenery::SAND_END as f32 + 60.0;
        let mut c = chunk(CENTER[0] + far, CENTER[1], |_, _| FLAT_Z);
        c.layer_textures = vec!["a".into(), "b".into()];
        assert!(!paint_ground(&mut c));
        assert_eq!(c.layer_textures, vec!["a".to_string(), "b".to_string()]);

        // Chunk pres okraj plneho pisku: puvodni vrstvy zustanou, pisek pribude jako dalsi.
        let edge = hd_scenery::SAND_FULL as f32 + 20.0;
        let mut c = chunk(CENTER[0] + edge, CENTER[1] + 10.0, |_, _| FLAT_Z);
        c.layer_textures = vec!["a".into(), "b".into()];
        c.layer_effect_ids = vec![1, 2];
        assert!(paint_ground(&mut c));
        assert_eq!(c.layer_textures.len(), 3);
        assert_eq!(c.layer_textures[2], SAND_TEXTURE);
        assert_eq!(c.layer_effect_ids, vec![1, 2, NO_EFFECT]);
        let alpha = c.alpha_map.unwrap();
        let sand: Vec<u8> = alpha.chunks_exact(4).map(|px| px[1]).collect();
        assert!(sand.iter().any(|&a| a == 255) && sand.iter().any(|&a| a < 255));
    }
}
