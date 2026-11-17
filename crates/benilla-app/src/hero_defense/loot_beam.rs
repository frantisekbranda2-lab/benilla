//! Hero Defense: sloup svetla nad mrtvou jednotkou, ve ktere lezi korist.
//!
//! Server (`core` `HdGame::RollLoot`, `BossLoot`) da do mrtvoly predmet a nastavi
//! `UNIT_DYNFLAG_LOOTABLE`; kazdemu klientovi ho posila jen tehdy, kdyz smi lootovat on
//! (`Object::BuildValuesUpdate`, `IsAllowedToLoot`). Sloup tedy vidi jen hrac, pro ktereho korist je
//! (u bosse cela skupina). Zmizi sam: po vylootovani server priznak sunda a mrtvolu odvola
//! (`UpdateLootCorpses`), na zacatku dalsi vlny mrtvoly odvola vsechny (`DespawnLootCorpses`).
//!
//! Kresli se v pruhu efektu sveta ([`benilla_world::particles::buffer::WorldEffectDraw`]) aditivne:
//! svisly pruh natoceny ke kamere (siroka mekka zare a uzke jasne jadro), zare na zemi promitnuta
//! na teren (jako varovani bossu v [`super::telegraphs`]) a jiskry stoupajici vzhuru. Vse jemne
//! pulzuje, aby si toho hrac vsiml i z kamery shora.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use benilla_protocol::EntityKind;
use benilla_world::decal::{DecalFrame, WorldDecal};
use benilla_world::particles::buffer::{EffectVertex, WorldEffectDraw};
use benilla_world::view::WorldCamera;

use crate::net::{NetEntity, ObjectStore};

/// Do jake vzdalenosti od kamery (yd) se sloup kresli.
const RANGE: f32 = 250.0;
/// Barva svetla (sRGB, zlata) a jadra (skoro bila).
const GLOW: [f32; 3] = [1.0, 0.78, 0.3];
const CORE: [f32; 3] = [1.0, 0.95, 0.75];
/// Sloup: vyska (yd), polovina sirky zare a jadra (yd), pruhlednost.
const HEIGHT: f32 = 16.0;
const GLOW_HALF: f32 = 1.3;
const CORE_HALF: f32 = 0.3;
const GLOW_ALPHA: f32 = 0.55;
const CORE_ALPHA: f32 = 0.9;
/// Zare na zemi: polomer (yd), svisly rozsah promitani (yd) a pruhlednost.
const GROUND_RADIUS: f32 = 2.2;
const GROUND_VERT: f32 = 3.0;
const GROUND_ALPHA: f32 = 0.8;
/// Pulzovani (radianu za sekundu) a jak moc (podil jasu).
const PULSE_SPEED: f32 = 3.0;
const PULSE_DEPTH: f32 = 0.25;
/// Jiskry: pocet na sloup, doba vystupu nahoru (s), velikost (yd) a polomer kruhu kolem stredu.
const MOTES: usize = 7;
const MOTE_RISE: f32 = 2.6;
const MOTE_SIZE: f32 = 0.35;
const MOTE_RADIUS: f32 = 0.7;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_textures).add_systems(
        PostUpdate,
        draw_beams.after(benilla_world::particles::buffer::begin_effect_frame),
    );
}

#[derive(Resource)]
struct BeamTextures {
    /// Svisly pruh: jasny uprostred sirky, slabne nahoru.
    beam: Handle<Image>,
    /// Mekky kruh (jiskry).
    dot: Handle<Image>,
    /// Zare na zemi: mekky kruh a jasnejsi prstenec.
    ground: Handle<Image>,
}

/// Bila textura s pruhlednosti `alpha(u, v)`, `u` a `v` od -1 do 1 (v = -1 nahore).
fn make_texture(w: u32, h: u32, alpha: impl Fn(f32, f32) -> f32) -> Image {
    let mut data = vec![255u8; (w * h * 4) as usize];
    for j in 0..h {
        for i in 0..w {
            let u = (i as f32 + 0.5) / w as f32 * 2.0 - 1.0;
            let v = (j as f32 + 0.5) / h as f32 * 2.0 - 1.0;
            data[((j * w + i) * 4 + 3) as usize] = (alpha(u, v).clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn setup_textures(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let beam = make_texture(64, 256, |u, v| {
        // Napric: mekky stred; podel: dole plny, nahoru slabne, uplne dole kratky nabeh.
        let across = (1.0 - u.abs()).powf(2.2);
        let up = (1.0 - v) * 0.5; // 0 dole, 1 nahore
        let along = (1.0 - up).powf(1.4) * (up / 0.03).min(1.0);
        across * along
    });
    let dot = make_texture(32, 32, |u, v| {
        let r = (u * u + v * v).sqrt();
        (1.0 - r).max(0.0).powf(2.0)
    });
    let ground = make_texture(128, 128, |u, v| {
        let r = (u * u + v * v).sqrt();
        let soft = (1.0 - r).max(0.0).powf(1.6) * 0.7;
        let ring = (1.0 - ((r - 0.78) / 0.12).abs()).max(0.0) * 0.8;
        (soft + ring) * (1.0 - ((r - 0.97) / 0.03).clamp(0.0, 1.0))
    });
    commands.insert_resource(BeamTextures {
        beam: images.add(beam),
        dot: images.add(dot),
        ground: images.add(ground),
    });
}

/// Ctyri vrcholy svisleho pruhu od `base` nahoru: vlevo dole, vpravo dole, vpravo nahore, vlevo
/// nahore (poradi jako pruhy blesku v `entities::chain_beam`).
fn push_quad(out: &mut Vec<EffectVertex>, corners: [Vec3; 4], color: [f32; 3], alpha: f32) {
    let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    for (pos, uv) in corners.into_iter().zip(uvs) {
        out.push(EffectVertex {
            pos: pos.to_array(),
            uv,
            color: [color[0], color[1], color[2], alpha],
        });
    }
}

/// Nahodne, ale pro danou mrtvolu stale stejne cislo 0..1 (rozestaveni jisker).
fn hash01(seed: u64, i: usize) -> f32 {
    let mut x = seed ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 33;
    x = x.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    x ^= x >> 33;
    (x & 0xFFFF) as f32 / 65535.0
}

#[allow(clippy::too_many_arguments)]
fn draw_beams(
    textures: Option<Res<BeamTextures>>,
    units: Query<(Entity, &NetEntity, &Transform, Option<&ObjectStore>)>,
    cam: Query<(Entity, &GlobalTransform), With<WorldCamera>>,
    time: Res<Time>,
    decals: WorldDecal,
    mut draw: WorldEffectDraw,
    mut ground: Local<Vec<EffectVertex>>,
    mut corpses: Local<Vec<(Entity, Vec3)>>,
) {
    let Some(textures) = textures else { return };
    let Ok((cam, cam_xf)) = cam.single() else {
        return;
    };
    let cam_pos = cam_xf.translation();
    corpses.clear();
    for (entity, net, tf, store) in &units {
        if !matches!(net.kind, EntityKind::Unit) {
            continue;
        }
        let Some(store) = store else { continue };
        if store.0.unit_health().unwrap_or(1) != 0 || !store.0.unit_lootable() {
            continue;
        }
        if tf.translation.distance(cam_pos) > RANGE {
            continue;
        }
        corpses.push((entity, tf.translation));
    }
    if corpses.is_empty() {
        return;
    }

    let now = time.elapsed_secs();
    // Pruh natoceny ke kamere kolem svisle osy: bok je pravy smer kamery bez svisle slozky.
    let mut side = *cam_xf.right();
    side.y = 0.0;
    let side = side.try_normalize().unwrap_or(Vec3::X);
    let up = Vec3::Y;
    let cam_up = *cam_xf.up();
    let cam_right = *cam_xf.right();

    for &(entity, base) in corpses.iter() {
        let seed = entity.to_bits();
        let phase = hash01(seed, 99) * std::f32::consts::TAU;
        let pulse = 1.0 - PULSE_DEPTH * 0.5 * (1.0 + (now * PULSE_SPEED + phase).sin());

        // Zare na zemi, promitnuta na teren.
        let frame = DecalFrame {
            center: base,
            sin: 0.0,
            cos: 1.0,
            min_x: -GROUND_RADIUS,
            max_x: GROUND_RADIUS,
            min_z: -GROUND_RADIUS,
            max_z: GROUND_RADIUS,
            min_y: -GROUND_VERT,
            max_y: GROUND_VERT,
        };
        ground.clear();
        if decals.project(
            &mut ground,
            &frame,
            // Posledni yard svisleho rozsahu slabne (p je vuci stredu, jako u varovani bossu).
            |p| (GROUND_VERT - p.y.abs()).clamp(0.0, 1.0),
            |x, z| frame.rect_uv(x, z),
        ) {
            let mut batch = draw
                .batch(cam, textures.ground.id())
                .additive()
                .anchored(base)
                .rung(
                    benilla_world::sky_order::Rung::RETICLE,
                    benilla_world::sky_order::Rung::DECAL_RASTER,
                );
            batch.extend(ground.iter().map(|v| EffectVertex {
                pos: v.pos,
                uv: v.uv,
                color: [GLOW[0], GLOW[1], GLOW[2], v.color[3] * GROUND_ALPHA * pulse],
            }));
            batch.tris();
        }

        // Sloup: siroka zare a uzke jadro.
        let mid = base + up * (HEIGHT * 0.5);
        let mut batch = draw
            .batch(cam, textures.beam.id())
            .additive()
            .anchored(mid)
            .owner(entity);
        let column = |half: f32, height: f32| {
            [
                base - side * half,
                base + side * half,
                base + side * half + up * height,
                base - side * half + up * height,
            ]
        };
        push_quad(
            batch.verts_mut(),
            column(GLOW_HALF, HEIGHT),
            GLOW,
            GLOW_ALPHA * pulse,
        );
        push_quad(
            batch.verts_mut(),
            column(CORE_HALF, HEIGHT * 0.85),
            CORE,
            CORE_ALPHA * pulse,
        );
        batch.quads();

        // Jiskry: stoupaji od zeme po kruhu kolem stredu, nahore zhasnou.
        let mut batch = draw
            .batch(cam, textures.dot.id())
            .additive()
            .anchored(mid)
            .owner(entity);
        for i in 0..MOTES {
            let offset = hash01(seed, i);
            let t = (now / MOTE_RISE + offset).fract();
            let angle = hash01(seed, i + 31) * std::f32::consts::TAU + now * 0.8;
            let radius = MOTE_RADIUS * (0.4 + 0.6 * hash01(seed, i + 57));
            let at = base
                + Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius)
                + up * (0.3 + t * HEIGHT * 0.55);
            let fade = (t / 0.15).min(1.0) * (1.0 - t);
            let size = MOTE_SIZE * (0.7 + 0.6 * hash01(seed, i + 83));
            let (r, u) = (cam_right * size, cam_up * size);
            push_quad(
                batch.verts_mut(),
                [at - r - u, at + r - u, at + r + u, at - r + u],
                CORE,
                fade,
            );
        }
        batch.quads();
    }
}
