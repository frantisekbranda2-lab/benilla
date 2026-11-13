//! Hero Defense: ukazatele zivotu nad jednotkami misto nameplates WoW (klavesa V).
//!
//! Puvodni nameplates (`crate::vplates`) jsou velke, jen do 20 yd a zapinaji se klavesou V. Tady
//! je ma kazda jednotka do [`RANGE`] yd od hrdiny: maly zaobleny prouzek s tmavym podkladem,
//! barva podle vztahu (nepritel cerveny, spojenec zeleny, hrac modry), ubrane zivoty chvili
//! dobihaji svetlym pruhem. Cil a jednotka pod mysi maji zlaty okraj.
//!
//! Kresli se do spodni vrstvy rozhrani ([`crate::ui_pass::UiQuads::overlays`], stejne jako
//! bojovy text), takze jsou pod okny a lištami, ne pres ne. Nameplates WoW jsou v Hero Defense
//! vypnute ([`crate::vplates::PlateSteps`]).
//!
//! Spojenecke jednotky (vlastni jednotky, prodejci) maji ukazatel jen zranene, v cili nebo pod
//! mysi, at plne zive jednotky mezi vlnami nezaplni obrazovku.

use std::collections::HashMap;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use benilla_protocol::EntityKind;
use benilla_ui::script::BlendMode;
use benilla_world::view::WorldCamera;

use crate::entities::{overhead_anchor, BoneAttach, OverheadFallback};
use crate::net::{Embodied, Guid, NetEntity, ObjectStore};
use crate::target::{Hovered, RelationWorld, Selection};
use crate::ui_pass::{overlay_z, UiQuad, UiQuadAppend, UiQuads};

/// Do jake vzdalenosti od hrdiny (yd) maji jednotky ukazatel. Puvodni nameplates 20 yd.
const RANGE: f32 = 120.0;
/// Ukazatel nad hlavou jednotky (yd nad kotvou jmena).
const LIFT: f32 = 0.45;
/// Rozmery na obrazovce vysoke 1080 px (zmensuji se a zvetsuji s vyskou okna).
const W_1080: f32 = 46.0;
const H_1080: f32 = 6.0;
/// Daleke jednotky maji ukazatel mensi: od [`SHRINK_FROM`] yd az na [`FAR_SCALE`] v [`RANGE`].
const SHRINK_FROM: f32 = 25.0;
const FAR_SCALE: f32 = 0.7;
/// Ubrane zivoty: svetly pruh chvili stoji, pak dobehne (podil plnych zivotu za sekundu).
const GHOST_HOLD: f32 = 0.35;
const GHOST_SPEED: f32 = 0.9;

/// Barvy (sRGB jako v rozhrani WoW): horni a dolni okraj prechodu.
const ENEMY: ([f32; 3], [f32; 3]) = ([0.95, 0.28, 0.22], [0.62, 0.06, 0.05]);
const NEUTRAL: ([f32; 3], [f32; 3]) = ([1.0, 0.86, 0.3], [0.78, 0.55, 0.05]);
const FRIEND: ([f32; 3], [f32; 3]) = ([0.42, 0.95, 0.38], [0.1, 0.58, 0.12]);
const PLAYER: ([f32; 3], [f32; 3]) = ([0.45, 0.75, 1.0], [0.1, 0.38, 0.85]);
const BACK: [f32; 4] = [0.04, 0.04, 0.05, 0.78];
const EDGE: [f32; 4] = [0.0, 0.0, 0.0, 0.9];
const LIT_EDGE: [f32; 4] = [1.0, 0.82, 0.2, 1.0];
const GHOST: [f32; 4] = [1.0, 0.93, 0.75, 0.85];

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, draw_bars.in_set(UiQuadAppend));
}

#[derive(SystemParam)]
struct BarWorld<'w, 's> {
    units: Query<
        'w,
        's,
        (
            Entity,
            &'static NetEntity,
            &'static Guid,
            &'static Transform,
            Option<&'static ObjectStore>,
        ),
    >,
    centre: Query<'w, 's, (Entity, &'static Transform), With<Embodied>>,
    camera: Query<'w, 's, (&'static Camera, &'static Transform), With<WorldCamera>>,
    window: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    widescreen: Option<Res<'w, crate::video::Widescreen>>,
    relations: RelationWorld<'w, 's>,
    selection: Res<'w, Selection>,
    hovered: Res<'w, Hovered>,
    time: Res<'w, Time>,
}

/// Svetly pruh ubranych zivotu jedne jednotky.
#[derive(Clone, Copy)]
struct Ghost {
    /// Zobrazeny podil zivotu (0..1), dobiha k aktualnimu.
    shown: f32,
    /// Sekundy, nez se zacne hybat.
    hold: f32,
}

struct Bar {
    dist: f32,
    centre: Vec2,
    scale: f32,
    health: f32,
    ghost: f32,
    colour: ([f32; 3], [f32; 3]),
    lit: bool,
}

#[allow(clippy::too_many_arguments)]
fn draw_bars(
    world: BarWorld,
    anchors: Query<&BoneAttach>,
    poses: Query<&benilla_world::rig_anim::RigPose>,
    fallbacks: Query<&OverheadFallback>,
    globals: Query<&GlobalTransform>,
    mounts: Query<(), With<crate::entities::mount::MountChild>>,
    mut ghosts: Local<HashMap<Entity, Ghost>>,
    mut quads: ResMut<UiQuads>,
) {
    let Ok((cam, cam_pose)) = world.camera.single() else {
        ghosts.clear();
        return;
    };
    let Ok((me_entity, me_tf)) = world.centre.single() else {
        ghosts.clear();
        return;
    };
    let Some(cam_viewport) = cam.logical_viewport_size() else {
        return;
    };
    let cam_tf = GlobalTransform::from(*cam_pose);
    let ui = crate::ui_script::UiScreen::of(
        cam_viewport,
        world.widescreen.as_deref().is_none_or(|w| w.0),
    );
    let px = ui.size.y / 1080.0;
    let device = world.window.single().map_or(1.0, |w| w.scale_factor());
    let dt = world.time.delta_secs();
    let rel = world.relations.get();
    let me = world.relations.me();

    let mut bars = Vec::new();
    let mut seen = Vec::new();
    for (entity, net, guid, tf, store) in &world.units {
        if entity == me_entity || !matches!(net.kind, EntityKind::Unit | EntityKind::Player) {
            continue;
        }
        let Some(store) = store else { continue };
        let dist = tf.translation.distance(me_tf.translation);
        if dist > RANGE {
            continue;
        }
        let health = store.0.unit_health().unwrap_or(0) as f32;
        let max = store.0.unit_max_health().unwrap_or(0) as f32;
        // Mrtve, nevybiratelne (flags bit 25) a duchove ukazatel nemaji.
        if health <= 0.0
            || max <= 0.0
            || store.0.unit_flags() & (1 << 25) != 0
            || store.0.unit_is_ghost_visual()
        {
            continue;
        }
        let is_player = net.kind == EntityKind::Player;
        let unit = world.relations.unit_of(guid.0, store);
        let (attackable, neutral) = me.as_ref().map_or((false, false), |me| {
            let attackable = rel.can_attack(me, &unit);
            let neutral = !attackable
                && !is_player
                && rel.reaction(&unit, me) == benilla_formats::Reaction::Neutral as u8;
            (attackable, neutral)
        });
        if attackable && store.0.unit_reads_dead() {
            continue;
        }
        let lit = world.selection.target == Some(entity) || world.hovered.target == Some(entity);
        let frac = (health / max).clamp(0.0, 1.0);
        // Spojenci jen zranene nebo v cili.
        if !attackable && frac >= 1.0 && !lit {
            continue;
        }
        let anchor = overhead_anchor(entity, tf, &anchors, &poses, &fallbacks, &globals, &mounts);
        let Some(screen) =
            crate::ui_pass::project_overlay(cam, &cam_tf, anchor + Vec3::Y * LIFT, cam_viewport)
                .map(|p| ui.point(p))
        else {
            continue;
        };

        // Svetly pruh: pri ztrate stoji a dobiha, pri leceni hned dorovna.
        let g = ghosts.entry(entity).or_insert(Ghost {
            shown: frac,
            hold: 0.0,
        });
        if frac >= g.shown {
            *g = Ghost {
                shown: frac,
                hold: 0.0,
            };
        } else if g.hold < GHOST_HOLD {
            g.hold += dt;
        } else {
            g.shown = (g.shown - GHOST_SPEED * dt).max(frac);
        }
        seen.push(entity);

        let t = ((dist - SHRINK_FROM) / (RANGE - SHRINK_FROM)).clamp(0.0, 1.0);
        bars.push(Bar {
            dist,
            centre: screen,
            scale: px * (1.0 - t * (1.0 - FAR_SCALE)),
            health: frac,
            ghost: g.shown,
            colour: if attackable {
                ENEMY
            } else if neutral {
                NEUTRAL
            } else if is_player {
                PLAYER
            } else {
                FRIEND
            },
            lit,
        });
    }
    ghosts.retain(|e, _| seen.contains(e));

    // Vzdalene prvni, blizsi se kresli pres ne.
    bars.sort_by(|a, b| b.dist.total_cmp(&a.dist));
    let snap = |v: f32| crate::vplates::device_snap(v, device);
    for bar in bars {
        let w = snap((W_1080 * bar.scale).max(24.0));
        let h = snap((H_1080 * bar.scale).max(3.0));
        let edge = snap(bar.scale.max(1.0 / device));
        let left = snap(bar.centre.x - w * 0.5);
        let top = snap(bar.centre.y - h);
        let outer = Rect::new(left - edge, top - edge, left + w + edge, top + h + edge);
        let inner = Rect::new(left, top, left + w, top + h);
        let push =
            |quads: &mut UiQuads, rect: Rect, color: [f32; 4], corners: Option<[[f32; 4]; 4]>| {
                if rect.width() > 0.0 && rect.height() > 0.0 {
                    quads.overlays.push(UiQuad {
                        rect,
                        z_key: overlay_z::HD_BARS,
                        color,
                        corner_colors: corners,
                        blend: BlendMode::Blend,
                        ..default()
                    });
                }
            };
        // Okraj (u cile zlaty), podklad, svetly pruh, zivoty s prechodem a leskem nahore.
        push(
            &mut quads,
            outer,
            if bar.lit { LIT_EDGE } else { EDGE },
            None,
        );
        push(&mut quads, inner, BACK, None);
        if bar.ghost > bar.health {
            let r = Rect::new(left + w * bar.health, top, left + w * bar.ghost, top + h);
            push(&mut quads, r, GHOST, None);
        }
        let fill = Rect::new(left, top, left + snap(w * bar.health).max(1.0), top + h);
        let (hi, lo) = bar.colour;
        let boost = if bar.lit { 1.15 } else { 1.0 };
        let c = |rgb: [f32; 3]| {
            [
                (rgb[0] * boost).min(1.0),
                (rgb[1] * boost).min(1.0),
                (rgb[2] * boost).min(1.0),
                1.0,
            ]
        };
        // Rohy TL, TR, BR, BL.
        push(&mut quads, fill, c(hi), Some([c(hi), c(hi), c(lo), c(lo)]));
        let shine = Rect::new(fill.min.x, top, fill.max.x, top + (h * 0.35).max(1.0));
        push(&mut quads, shine, [1.0, 1.0, 1.0, 0.18], None);
    }
}
