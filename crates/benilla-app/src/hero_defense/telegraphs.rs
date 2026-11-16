//! Hero Defense: varovani na zemi (mechaniky bossu) a sipka nad oznacenou jednotkou.
//!
//! Server (`core` `HdGame::Telegraph`, `MarkUnit`) posila v `SMSG_DBLOOKUP` radky:
//! - `TG;id;tvar;x;y;z;tx;ty;polomer;sirka;uhel;ms;plni;barva[;efekt]`: obrazec na zemi. Tvar `C` kruh
//!   (stred x,y), `R` pruh (od x,y k tx,ty, delka polomer, sirka), `V` vysec (vrchol x,y, smer
//!   k tx,ty, polomer, uhel ve stupnich). `ms` doba (0 = dokud neprijde `TGX`), `plni` 1 =
//!   vyplnuje se od stredu nebo zacatku a az je plny, prijde dopad. Barva `O` varovani, `R` zasah.
//!   Stejne id nahradi obrazec a ponecha mu cas zacatku (pruh charge sleduje hrdinu). Efekt `B`:
//!   z nebe pada balvan a dopadne presne, kdyz je kruh plny; `S`: uder do zeme (prach, tlakova
//!   vlna, otres kamery) na konci plneni.
//! - `TGX;id`: zrusit obrazec (0 = vsechny).
//! - `TM;guid;ms`: velka cervena sipka nad jednotkou (guid hexadecimalne), 0 = pryc.
//! - `HFX;G;x;y;z;ms;polomer`: rotujici gleva jednotky Rotujici gleva (`hd_tower.cpp` `AddZone`):
//!   velka tocici se gleva naplocho na zemi po dobu plochy.
//!
//! Obrazce se promitaji na teren a budovy stejne jako zamerovac plosnych kouzel
//! ([`crate::target`] `reticle`, [`benilla_world::decal`]): obrys je vyrazny, vnitrek pruhledny,
//! plnici se cast sytejsi. Kdo dostal zasah, rozhoduje jen server.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use benilla_assets::coords::wow_to_bevy;
use benilla_ui::script::BlendMode;
use benilla_world::decal::{DecalFrame, WorldDecal};
use benilla_world::particles::buffer::EffectVertex;
use benilla_world::view::WorldCamera;

use crate::camera_shake::PresetShake;
use crate::entities::dest_fx::spawn_world_fx;
use crate::entities::{overhead_anchor, BoneAttach, OverheadFallback};
use crate::net::Guid;
use crate::ui_pass::{overlay_z, UiQuad, UiQuadAppend, UiQuads};

/// Svisly rozsah promitani (yd nad a pod bodem ze serveru); posledni 2 yd slabnou.
const VERT: f32 = 8.0;
const VERT_FADE: f32 = 2.0;
/// Velikost generovanych textur (px).
const TEX: u32 = 256;
/// Sirka obrysu pruhu (yd).
const LANE_EDGE: f32 = 0.35;
/// Pruhlednost: vnitrek, plnici se cast, obrys.
const BASE_ALPHA: f32 = 0.22;
const FILL_ALPHA: f32 = 0.5;
const EDGE_ALPHA: f32 = 0.95;
/// Barvy (sRGB): varovani a zasah.
const ORANGE: [f32; 3] = [1.0, 0.55, 0.1];
const RED: [f32; 3] = [1.0, 0.12, 0.08];
/// Obrazec bez doby (ms 0), ktery server nezrusil, zmizi nejpozdeji po teto dobe (s).
const MAX_AGE: f64 = 120.0;

/// Balvan (efekt `B`): model strely, odkud pada (yd primo nad stredem kruhu), velikost a otaceni
/// (radianu za sekundu). Pada rovnomerne po celou dobu plneni kruhu a dopadne, kdyz je kruh plny.
const BOULDER_MODEL: &str = "spells\\BoulderGiant_Missile.mdx";
const BOULDER_HEIGHT: f32 = 22.0;
const BOULDER_SCALE: f32 = 2.0;
const BOULDER_SPIN: f32 = 2.5;
/// Dopad balvanu: vybuch kamene a prach (velikost na yd polomeru kruhu), slaby otres.
const ROCK_IMPACT: &str = "spells\\Meteor_Impact_Base.mdx";
const ROCK_DUST: &str = "Spells\\DustCloud_Land.mdx";
const ROCK_SHAKE: u32 = 1;
/// Uder (efekt `S`): tlakova vlna prachu a zahrmeni ze stredu, prach po kruhu, silny otres.
const SLAM_NOVA: &str = "spells\\dustnova_cast_base.mdx";
const SLAM_CLAP: &str = "Spells\\ThunderClap_Cast_Base.mdx";
const SLAM_GROUND: &str = "spells\\SandWorm_AttackEffects.mdx";
/// Polomer, ktery modely tlakove vlny maji samy od sebe (yd): podle nej se zvetsi na kruh.
const SLAM_NATIVE_RADIUS: f32 = 8.0;
const SLAM_SHAKES: [u32; 2] = [2, 12];
const SLAM_DUST_RING: usize = 12;

/// Rotujici gleva (`HFX;G`): ostri vrhace glevy nocnich elfu lezi v modelu naplocho, stred
/// (WoW souradnice modelu) a polomer z ohranicujiciho kvadru vrcholu; zvetsi se na polomer plochy.
const GLAIVE_MODEL: &str =
    "World\\Generic\\NIGHTELF\\PASSIVE DOODADS\\GlaiveThrower\\NightElfGlaiveThrowerBlade_Doodad01.m2";
const GLAIVE_CENTER: [f32; 3] = [1.02, 0.16, 0.0];
const GLAIVE_NATIVE_RADIUS: f32 = 3.6;
/// Otaceni (radianu za sekundu) a zdvih nad zem (yd).
const GLAIVE_SPIN: f32 = 9.0;
const GLAIVE_LIFT: f32 = 0.6;
/// Vir pod glevou (Whirlwind) a jeho polomer od sebe (yd).
const GLAIVE_WHIRL: &str = "Spells\\Whirlwind_State_Base.mdx";
const GLAIVE_WHIRL_RADIUS: f32 = 3.0;

/// Sipka nad jednotkou: rozmery pri vysce okna 1080 px, odstup nad hlavou, pohupovani.
const ARROW_W_1080: f32 = 34.0;
const ARROW_H_1080: f32 = 44.0;
const ARROW_LIFT_1080: f32 = 46.0;
const ARROW_BOB_1080: f32 = 6.0;
const ARROW_FILL: [f32; 4] = [1.0, 0.15, 0.1, 1.0];
const ARROW_EDGE: [f32; 4] = [0.0, 0.0, 0.0, 0.9];

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Telegraphs>()
        .add_systems(Startup, setup_textures)
        .add_systems(Update, (draw_marks.in_set(UiQuadAppend), telegraph_fx))
        .add_systems(
            PostUpdate,
            draw_telegraphs.after(benilla_world::particles::buffer::begin_effect_frame),
        );
}

#[derive(Clone, Copy, PartialEq)]
enum Fx {
    None,
    Boulder,
    Slam,
}

#[derive(Clone, Copy, PartialEq)]
enum Shape {
    Circle,
    Lane,
    Cone,
}

struct Telegraph {
    shape: Shape,
    /// Stred kruhu, zacatek pruhu, vrchol vysece (Bevy).
    at: Vec3,
    /// Smer pruhu a vysece v rovine (Bevy x, z), jednotkovy.
    dir: Vec2,
    radius: f32,
    width: f32,
    arc: f32,
    /// Zacatek a doba v sekundach `Time::elapsed`; doba 0 = do zruseni.
    start: f64,
    duration: f32,
    fill: bool,
    color: [f32; 3],
    fx: Fx,
    /// Padajici balvan (efekt `B`), dokud nedopadne.
    boulder: Option<Entity>,
    /// Efekt dopadu uz probehl.
    fired: bool,
}

/// Rotujici gleva na zemi (`HFX;G`).
struct Spinner {
    at: Vec3,
    radius: f32,
    start: f64,
    end: f64,
    /// Model glevy a vir pod ni, az se vytvori.
    blade: Option<Entity>,
    whirl: Option<Entity>,
}

/// Obrazce a znacky od serveru.
#[derive(Resource, Default)]
pub(crate) struct Telegraphs {
    shapes: HashMap<u32, Telegraph>,
    spinners: Vec<Spinner>,
    /// Balvany zrusenych obrazcu (boss padl): zmizi v [`telegraph_fx`].
    orphans: Vec<Entity>,
    /// guid -> konec v sekundach `Time::elapsed`.
    marks: HashMap<u64, f64>,
}

impl Telegraphs {
    /// Radek `TG`, `TGX` nebo `TM` (bez prefixu a stredniku za nim); `false`, kdyz neni nas.
    pub(crate) fn read(&mut self, text: &str, now: f64) -> bool {
        if let Some(rest) = text.strip_prefix("TGX;") {
            match rest.trim().parse::<u32>() {
                Ok(0) => {
                    let gone: Vec<u32> = self.shapes.keys().copied().collect();
                    for id in gone {
                        self.remove(id);
                    }
                }
                Ok(id) => self.remove(id),
                Err(_) => {}
            }
            return true;
        }
        if let Some(rest) = text.strip_prefix("TG;") {
            self.read_shape(rest, now);
            return true;
        }
        if let Some(rest) = text.strip_prefix("HFX;G;") {
            let f: Vec<f32> = rest
                .split(';')
                .filter_map(|v| v.trim().parse::<f32>().ok())
                .collect();
            if let [x, y, z, ms, radius] = f[..] {
                self.spinners.push(Spinner {
                    at: wow_to_bevy([x, y, z]),
                    radius: radius.max(1.0),
                    start: now,
                    end: now + f64::from(ms) / 1000.0,
                    blade: None,
                    whirl: None,
                });
            }
            return true;
        }
        if let Some(rest) = text.strip_prefix("TM;") {
            let mut f = rest.split(';');
            if let (Some(Ok(guid)), Some(Ok(ms))) = (
                f.next().map(|g| u64::from_str_radix(g, 16)),
                f.next().map(|m| m.trim().parse::<u32>()),
            ) {
                if ms == 0 {
                    self.marks.remove(&guid);
                } else {
                    self.marks.insert(guid, now + f64::from(ms) / 1000.0);
                }
            }
            return true;
        }
        false
    }

    fn read_shape(&mut self, rest: &str, now: f64) {
        let f: Vec<&str> = rest.split(';').collect();
        let [id, shape, x, y, z, tx, ty, radius, width, arc, ms, fill, color, ref rest @ ..] =
            f[..]
        else {
            return;
        };
        let fx = match rest.first().map(|s| s.trim()) {
            Some("B") => Fx::Boulder,
            Some("S") => Fx::Slam,
            _ => Fx::None,
        };
        let num = |s: &str| s.trim().parse::<f32>().ok();
        let (
            Ok(id),
            Some(x),
            Some(y),
            Some(z),
            Some(tx),
            Some(ty),
            Some(radius),
            Some(width),
            Some(arc),
            Ok(ms),
        ) = (
            id.parse::<u32>(),
            num(x),
            num(y),
            num(z),
            num(tx),
            num(ty),
            num(radius),
            num(width),
            num(arc),
            ms.trim().parse::<u32>(),
        )
        else {
            return;
        };
        let shape = match shape {
            "R" => Shape::Lane,
            "V" => Shape::Cone,
            _ => Shape::Circle,
        };
        let at = wow_to_bevy([x, y, z]);
        let to = wow_to_bevy([tx, ty, z]);
        let dir = Vec2::new(to.x - at.x, to.z - at.z).normalize_or(Vec2::X);
        // Stejne id: novy tvar, puvodni zacatek (plneni pokracuje) i padajici balvan.
        let (start, boulder, fired) = self.shapes.get(&id).map_or((now, None, false), |old| {
            (old.start, old.boulder, old.fired)
        });
        self.shapes.insert(
            id,
            Telegraph {
                shape,
                at,
                dir,
                radius: radius.max(0.1),
                width: width.max(0.1),
                arc: arc.clamp(1.0, 360.0),
                start,
                duration: ms as f32 / 1000.0,
                fill: fill.trim() == "1",
                color: if color.trim() == "R" { RED } else { ORANGE },
                fx,
                boulder,
                fired,
            },
        );
    }

    fn remove(&mut self, id: u32) {
        if let Some(t) = self.shapes.remove(&id) {
            self.orphans.extend(t.boulder);
        }
    }

    fn prune(&mut self, now: f64) {
        let orphans = &mut self.orphans;
        self.shapes.retain(|_, t| {
            let age = (now - t.start) as f32;
            let keep = if t.duration > 0.0 {
                age <= t.duration + 0.15
            } else {
                f64::from(age) < MAX_AGE
            };
            if !keep {
                orphans.extend(t.boulder.take());
            }
            keep
        });
        self.marks.retain(|_, &mut end| end > now);
        self.spinners.retain_mut(|s| {
            let keep = s.end > now;
            if !keep {
                orphans.extend(s.blade.take());
                orphans.extend(s.whirl.take());
            }
            keep
        });
    }
}

/// Generovane textury: kruh s obrysem, plny kruh, plna plocha (pruh) a vysece podle uhlu.
#[derive(Resource)]
struct TelegraphTextures {
    disc: Handle<Image>,
    disc_fill: Handle<Image>,
    solid: Handle<Image>,
    /// (uhel ve stupnich, plny) -> textura vysece.
    cones: HashMap<(u32, bool), Handle<Image>>,
}

/// Textura `TEX` x `TEX`, bila, alfa z `alpha(u, v)` v rozsahu -1..1 (stred 0).
fn make_texture(alpha: impl Fn(f32, f32) -> f32) -> Image {
    let mut data = vec![255u8; (TEX * TEX * 4) as usize];
    for j in 0..TEX {
        for i in 0..TEX {
            let u = (i as f32 + 0.5) / TEX as f32 * 2.0 - 1.0;
            let v = (j as f32 + 0.5) / TEX as f32 * 2.0 - 1.0;
            data[((j * TEX + i) * 4 + 3) as usize] = (alpha(u, v).clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    Image::new(
        Extent3d {
            width: TEX,
            height: TEX,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// Mekky prechod 0 -> 1 mezi `a` a `b`.
fn ramp(a: f32, b: f32, x: f32) -> f32 {
    ((x - a) / (b - a)).clamp(0.0, 1.0)
}

/// Kruh: vnitrek `inside`, obrys od polomeru `edge` do 1 (vyhlazeny okraj).
fn disc_alpha(u: f32, v: f32, inside: f32, edge: f32) -> f32 {
    let r = (u * u + v * v).sqrt();
    let outer = 1.0 - ramp(0.985, 1.0, r);
    let ring = ramp(edge - 0.015, edge, r);
    (inside + (1.0 - inside) * ring) * outer
}

/// Vysec uhlu `arc` (stupne) smerem +u, vrchol uprostred: vnitrek `inside`, obrys na oblouku
/// a na bocich (sirka `edge` v jednotkach polomeru).
fn cone_alpha(u: f32, v: f32, arc: f32, inside: f32, edge: f32) -> f32 {
    let r = (u * u + v * v).sqrt();
    if r > 1.0 {
        return 0.0;
    }
    let half = (arc * 0.5).to_radians();
    let angle = v.atan2(u).abs();
    if angle > half {
        return 0.0;
    }
    // Vzdalenost od boku (kolmo) a od oblouku, obe v jednotkach polomeru.
    let side = r * (half - angle).min(std::f32::consts::FRAC_PI_2).sin();
    let rim = 1.0 - r;
    let near = side.min(rim);
    let ring = 1.0 - ramp(edge * 0.6, edge, near);
    inside + (1.0 - inside) * ring
}

fn setup_textures(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.insert_resource(TelegraphTextures {
        disc: images.add(make_texture(|u, v| disc_alpha(u, v, BASE_ALPHA, 0.95))),
        disc_fill: images.add(make_texture(|u, v| disc_alpha(u, v, 1.0, 0.95))),
        solid: images.add(make_texture(|_, _| 1.0)),
        cones: HashMap::new(),
    });
}

impl TelegraphTextures {
    fn cone(&mut self, images: &mut Assets<Image>, arc: f32, fill: bool) -> Handle<Image> {
        let key = (arc.round() as u32, fill);
        self.cones
            .entry(key)
            .or_insert_with(|| {
                let arc = key.0 as f32;
                let inside = if fill { 1.0 } else { BASE_ALPHA };
                images.add(make_texture(move |u, v| {
                    cone_alpha(u, v, arc, inside, 0.02)
                }))
            })
            .clone()
    }
}

/// Ramec promitani: stred `at`, osa x' ve smeru `dir`, obdelnik `[x0, x1] x [z0, z1]` (yd).
fn frame(at: Vec3, dir: Vec2, x0: f32, x1: f32, z0: f32, z1: f32) -> DecalFrame {
    // in_frame: x' = dx·cos − dz·sin, z' = dz·cos + dx·sin; x' ma ukazovat ve smeru dir.
    DecalFrame {
        center: at,
        sin: -dir.y,
        cos: dir.x,
        min_x: x0,
        max_x: x1,
        min_z: z0,
        max_z: z1,
        min_y: -VERT,
        max_y: VERT,
    }
}

/// Jedno promitnuti do davky efektu: textura, barva a pruhlednost.
fn push(
    draw: &mut benilla_world::particles::buffer::WorldEffectDraw,
    decals: &WorldDecal,
    cam: Entity,
    texture: &Handle<Image>,
    frame: &DecalFrame,
    uv: impl Fn(f32, f32) -> [f32; 2],
    color: [f32; 3],
    alpha: f32,
    verts: &mut Vec<EffectVertex>,
) {
    verts.clear();
    let fade = |p: Vec3| ((VERT - p.y.abs()) / VERT_FADE).clamp(0.0, 1.0);
    if !decals.project(verts, frame, fade, uv) {
        return;
    }
    let mut batch = draw.batch(cam, texture.id()).anchored(frame.center).rung(
        benilla_world::sky_order::Rung::RETICLE,
        benilla_world::sky_order::Rung::DECAL_RASTER,
    );
    batch.extend(verts.iter().map(|v| EffectVertex {
        pos: v.pos,
        uv: v.uv,
        color: [color[0], color[1], color[2], v.color[3] * alpha],
    }));
    batch.tris();
}

#[allow(clippy::too_many_arguments)]
fn draw_telegraphs(
    mut telegraphs: ResMut<Telegraphs>,
    textures: Option<ResMut<TelegraphTextures>>,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
    cam: Query<Entity, With<WorldCamera>>,
    decals: WorldDecal,
    mut draw: benilla_world::particles::buffer::WorldEffectDraw,
    mut verts: Local<Vec<EffectVertex>>,
) {
    let now = time.elapsed_secs_f64();
    telegraphs.prune(now);
    let Some(mut textures) = textures else { return };
    let Ok(cam) = cam.single() else { return };
    for t in telegraphs.shapes.values() {
        let progress = if t.duration > 0.0 {
            ((now - t.start) as f32 / t.duration).clamp(0.0, 1.0)
        } else {
            1.0
        };
        match t.shape {
            Shape::Circle => {
                let square = |r: f32| frame(t.at, Vec2::X, -r, r, -r, r);
                let full = square(t.radius);
                let uv = |f: &DecalFrame| {
                    let (x0, x1, z0, z1) = (f.min_x, f.max_x, f.min_z, f.max_z);
                    move |x: f32, z: f32| [(x - x0) / (x1 - x0), (z - z0) / (z1 - z0)]
                };
                push(
                    &mut draw,
                    &decals,
                    cam,
                    &textures.disc,
                    &full,
                    uv(&full),
                    t.color,
                    1.0,
                    &mut verts,
                );
                if t.fill && progress > 0.01 {
                    let inner = square(t.radius * progress);
                    let texture = textures.disc_fill.clone();
                    push(
                        &mut draw,
                        &decals,
                        cam,
                        &texture,
                        &inner,
                        uv(&inner),
                        t.color,
                        FILL_ALPHA,
                        &mut verts,
                    );
                }
            }
            Shape::Lane => {
                let half = t.width * 0.5;
                let rect = |x0: f32, x1: f32, z0: f32, z1: f32| frame(t.at, t.dir, x0, x1, z0, z1);
                let flat = |_: f32, _: f32| [0.5, 0.5];
                let solid = textures.solid.clone();
                // Vnitrek, plnici se cast od zacatku, obrys (boky a oba konce).
                let body = rect(0.0, t.radius, -half, half);
                push(
                    &mut draw, &decals, cam, &solid, &body, flat, t.color, BASE_ALPHA, &mut verts,
                );
                if t.fill && progress > 0.01 {
                    let filled = rect(0.0, t.radius * progress, -half, half);
                    push(
                        &mut draw, &decals, cam, &solid, &filled, flat, t.color, FILL_ALPHA,
                        &mut verts,
                    );
                }
                let e = LANE_EDGE.min(half);
                for edge in [
                    rect(0.0, t.radius, -half, -half + e),
                    rect(0.0, t.radius, half - e, half),
                    rect(0.0, e, -half, half),
                    rect(t.radius - e, t.radius, -half, half),
                ] {
                    push(
                        &mut draw, &decals, cam, &solid, &edge, flat, t.color, EDGE_ALPHA,
                        &mut verts,
                    );
                }
            }
            Shape::Cone => {
                let square = |r: f32| frame(t.at, t.dir, -r, r, -r, r);
                let uv = |f: &DecalFrame| {
                    let (x0, x1, z0, z1) = (f.min_x, f.max_x, f.min_z, f.max_z);
                    move |x: f32, z: f32| [(x - x0) / (x1 - x0), (z - z0) / (z1 - z0)]
                };
                let outline = textures.cone(&mut images, t.arc, false);
                let full = square(t.radius);
                push(
                    &mut draw,
                    &decals,
                    cam,
                    &outline,
                    &full,
                    uv(&full),
                    t.color,
                    1.0,
                    &mut verts,
                );
                if t.fill && progress > 0.01 {
                    let filled = textures.cone(&mut images, t.arc, true);
                    let inner = square(t.radius * progress);
                    push(
                        &mut draw,
                        &decals,
                        cam,
                        &filled,
                        &inner,
                        uv(&inner),
                        t.color,
                        FILL_ALPHA,
                        &mut verts,
                    );
                }
            }
        }
    }
}

/// Sipka dolu nad oznacenymi jednotkami (i nad vlastnim hrdinou), ve vrstve ukazatelu zivotu.
#[allow(clippy::too_many_arguments)]
fn draw_marks(
    telegraphs: Res<Telegraphs>,
    units: Query<(Entity, &Guid, &Transform)>,
    camera: Query<(&Camera, &Transform), With<WorldCamera>>,
    widescreen: Option<Res<crate::video::Widescreen>>,
    anchors: Query<&BoneAttach>,
    poses: Query<&benilla_world::rig_anim::RigPose>,
    fallbacks: Query<&OverheadFallback>,
    globals: Query<&GlobalTransform>,
    mounts: Query<(), With<crate::entities::mount::MountChild>>,
    time: Res<Time>,
    mut quads: ResMut<UiQuads>,
) {
    if telegraphs.marks.is_empty() {
        return;
    }
    let Ok((cam, cam_pose)) = camera.single() else {
        return;
    };
    let Some(viewport) = cam.logical_viewport_size() else {
        return;
    };
    let cam_tf = GlobalTransform::from(*cam_pose);
    let ui = crate::ui_script::UiScreen::of(viewport, widescreen.as_deref().is_none_or(|w| w.0));
    let px = ui.size.y / 1080.0;
    let bob = (time.elapsed_secs() * 6.0).sin() * ARROW_BOB_1080 * px;
    for (entity, guid, tf) in &units {
        if !telegraphs.marks.contains_key(&guid.0) {
            continue;
        }
        let anchor = overhead_anchor(entity, tf, &anchors, &poses, &fallbacks, &globals, &mounts);
        let Some(head) =
            crate::ui_pass::project_overlay(cam, &cam_tf, anchor, viewport).map(|p| ui.point(p))
        else {
            continue;
        };
        let (w, h) = (ARROW_W_1080 * px, ARROW_H_1080 * px);
        // Spicka sipky nad hlavou (nad ukazatelem zivotu a ikonami aur).
        let tip = Vec2::new(head.x, head.y - ARROW_LIFT_1080 * px + bob);
        for (grow, color) in [(2.5 * px, ARROW_EDGE), (0.0, ARROW_FILL)] {
            let head_h = h * 0.55;
            let shaft_w = w * 0.4;
            // Hrot: trojuhelnik spickou dolu.
            let a = Vec2::new(tip.x, tip.y + grow);
            let b = Vec2::new(tip.x - w * 0.5 - grow, tip.y - head_h - grow * 0.5);
            let c = Vec2::new(tip.x + w * 0.5 + grow, tip.y - head_h - grow * 0.5);
            // Driek nad hrotem.
            let top = tip.y - h - grow;
            let shaft = Rect::new(
                tip.x - shaft_w * 0.5 - grow,
                top,
                tip.x + shaft_w * 0.5 + grow,
                tip.y - head_h + 1.0,
            );
            quads.overlays.push(UiQuad {
                rect: Rect::from_corners(Vec2::new(b.x, b.y.min(top)), Vec2::new(c.x, a.y)),
                z_key: overlay_z::HD_BARS,
                color,
                blend: BlendMode::Blend,
                corners: Some([a, b, c, c]),
                ..default()
            });
            quads.overlays.push(UiQuad {
                rect: shaft,
                z_key: overlay_z::HD_BARS,
                color,
                blend: BlendMode::Blend,
                ..default()
            });
        }
    }
}

/// Efekty mechanik: balvan pada z nebe a dopadne presne s naplnenim kruhu (efekt `B`), uder do
/// zeme na konci plneni (efekt `S`). Modely kouzel z archivu klienta, prehrane jako jednorazove
/// efekty kouzel na miste ([`spawn_world_fx`]); otres kamery z `CameraShakes.dbc`.
fn telegraph_fx(
    mut commands: Commands,
    mut telegraphs: ResMut<Telegraphs>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
    mut shakes: MessageWriter<PresetShake>,
) {
    let now = time.elapsed_secs_f64();
    for entity in telegraphs.orphans.drain(..) {
        if let Ok(mut e) = commands.get_entity(entity) {
            e.despawn();
        }
    }
    for s in &mut telegraphs.spinners {
        if s.end <= now {
            continue;
        }
        let scale = s.radius / GLAIVE_NATIVE_RADIUS;
        // Otaceni kolem stredu ostri (pocatek modelu je mimo stred).
        let rot = Quat::from_rotation_y((now - s.start) as f32 * GLAIVE_SPIN);
        let pos = s.at + Vec3::Y * GLAIVE_LIFT - rot * (wow_to_bevy(GLAIVE_CENTER) * scale);
        match s.blade {
            Some(entity) => {
                if let Ok(mut tf) = transforms.get_mut(entity) {
                    tf.translation = pos;
                    tf.rotation = rot;
                }
            }
            None => {
                let tf = Transform::from_translation(pos)
                    .with_rotation(rot)
                    .with_scale(Vec3::splat(scale));
                s.blade = Some(spawn_world_fx(&mut commands, GLAIVE_MODEL, tf, true));
                let whirl = Transform::from_translation(s.at)
                    .with_scale(Vec3::splat(s.radius / GLAIVE_WHIRL_RADIUS));
                s.whirl = Some(spawn_world_fx(&mut commands, GLAIVE_WHIRL, whirl, true));
            }
        }
    }
    for t in telegraphs.shapes.values_mut() {
        if t.fx == Fx::None || t.fired || t.duration <= 0.0 {
            continue;
        }
        let left = (t.start + f64::from(t.duration) - now) as f32;
        if t.fx == Fx::Boulder {
            let sky = t.at + Vec3::Y * BOULDER_HEIGHT;
            if left > 0.0 {
                // Kazdy kruh svuj balvan: stejnym tempem jako plneni kruhu, dopad s naplnenim.
                let k = (1.0 - left / t.duration).clamp(0.0, 1.0);
                let pos = sky.lerp(t.at, k);
                let spin = Quat::from_axis_angle(
                    Vec3::new(1.0, 0.0, 0.6).normalize(),
                    (t.duration - left) * BOULDER_SPIN,
                );
                match t.boulder {
                    Some(entity) => {
                        if let Ok(mut tf) = transforms.get_mut(entity) {
                            tf.translation = pos;
                            tf.rotation = spin;
                        }
                    }
                    None => {
                        let tf = Transform::from_translation(pos)
                            .with_rotation(spin)
                            .with_scale(Vec3::splat(BOULDER_SCALE));
                        t.boulder = Some(spawn_world_fx(&mut commands, BOULDER_MODEL, tf, true));
                    }
                }
            }
            if left <= 0.0 {
                t.fired = true;
                if let Some(entity) = t.boulder.take() {
                    if let Ok(mut e) = commands.get_entity(entity) {
                        e.despawn();
                    }
                }
                let size = (t.radius / 5.0).max(0.6);
                spawn_world_fx(
                    &mut commands,
                    ROCK_IMPACT,
                    Transform::from_translation(t.at).with_scale(Vec3::splat(size)),
                    false,
                );
                spawn_world_fx(
                    &mut commands,
                    ROCK_DUST,
                    Transform::from_translation(t.at).with_scale(Vec3::splat(size * 1.3)),
                    false,
                );
                shakes.write(PresetShake {
                    id: ROCK_SHAKE,
                    pos: t.at,
                });
            }
        } else if t.fx == Fx::Slam && left <= 0.0 {
            t.fired = true;
            let size = (t.radius / SLAM_NATIVE_RADIUS).max(0.5);
            for (path, scale) in [(SLAM_NOVA, size), (SLAM_CLAP, size), (SLAM_GROUND, 1.5)] {
                spawn_world_fx(
                    &mut commands,
                    path,
                    Transform::from_translation(t.at).with_scale(Vec3::splat(scale)),
                    false,
                );
            }
            // Prach po celem kruhu (vnejsi a vnitrni prstenec), at je videt jeho dosah.
            for ring in [0.95_f32, 0.55] {
                let n = if ring > 0.9 {
                    SLAM_DUST_RING
                } else {
                    SLAM_DUST_RING / 2
                };
                for k in 0..n {
                    let a = k as f32 / n as f32 * std::f32::consts::TAU;
                    let at = t.at + Vec3::new(a.cos(), 0.0, a.sin()) * t.radius * ring;
                    spawn_world_fx(
                        &mut commands,
                        ROCK_DUST,
                        Transform::from_translation(at).with_scale(Vec3::splat(1.6)),
                        false,
                    );
                }
            }
            for id in SLAM_SHAKES {
                shakes.write(PresetShake { id, pos: t.at });
            }
        }
    }
}
