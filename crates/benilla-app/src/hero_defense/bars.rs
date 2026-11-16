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
//! Nad ukazatelem jsou uprostred male ikony aur: na nepratelich debuffy, na spojencich buffy
//! (pasivni a skryte aury ne), u stackujicich se aur cislo stacku v pravem dolnim rohu ikony.
//!
//! Debuff na nepriteli ukazuje zbyvajici cas jako hodiny: ubehla cast ikony postupne ve smeru
//! hodinovych rucicek ztmavne. Dobu posila server ([`AuraTimes`], 1.12 ji u cizich jednotek
//! klientu neposila).
//!
//! Spojenecke jednotky (vlastni jednotky, prodejci) maji ukazatel jen zranene, v cili, pod mysi
//! nebo s buffem, at plne zive jednotky mezi vlnami nezaplni obrazovku.
//!
//! Nepritel ma vlevo od ukazatele ikonu role ([`EnemyRoles`], radek `ER` ze serveru): mec bezny
//! utocnik, stit obrance (stitonos, odolna vlna), plus lecitel, dyka assassin (jde po hrdinovi),
//! lebka boss.

use std::collections::HashMap;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use benilla_protocol::EntityKind;
use benilla_ui::script::{BlendMode, JustifyH, JustifyV, Outline};
use benilla_world::view::WorldCamera;

use crate::entities::{overhead_anchor, BoneAttach, OverheadFallback};
use crate::net::{Embodied, Guid, NetEntity, ObjectStore};
use crate::target::{Hovered, RelationWorld, Selection};
use crate::ui_pass::{overlay_z, UiQuad, UiQuadAppend, UiQuads};
use crate::ui_text::{layout_text_quads, FontSpec, Justify, TextSeat};

/// Do jake vzdalenosti od hrdiny (yd) maji jednotky ukazatel. Puvodni nameplates 20 yd.
const RANGE: f32 = 120.0;
/// Ukazatel nad hlavou jednotky (yd nad kotvou jmena).
const LIFT: f32 = 0.45;
/// Rozmery na obrazovce vysoke 1080 px (zmensuji se a zvetsuji s vyskou okna).
/// Prouzek je tak vysoky, aby se do nej vesel text zivotu ("920/1000").
const W_1080: f32 = 62.0;
const H_1080: f32 = 11.0;
/// Text zivotu: nejmensi vyska pisma (px), pod ni (daleke jednotky) se nekresli.
const HP_TEXT_MIN: f32 = 7.0;
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

/// Ikony aur nad ukazatelem: velikost (px pri 1080), mezera mezi nimi a nad ukazatelem, nejvic
/// ikon v rade.
const ICON_1080: f32 = 17.0;
const ICON_GAP_1080: f32 = 2.0;
const MAX_ICONS: usize = 6;
/// Ikony WoW maji v obrazku vlastni ramecek, orizne se (jako `SetTexCoord(.07, .93, .07, .93)`).
const ICON_CROP: f32 = 0.07;
/// Sloty aur 1.12: buffy 0 az 31, debuffy 32 az 47 (`UnitBuff`, `UnitDebuff`).
const FIRST_DEBUFF_SLOT: u8 = 32;
/// Aura, ktera ikonu nema: `SPELL_ATTR_PASSIVE` (0x40) a skryta (0x80).
const HIDDEN_AURA_ATTRS: u32 = 0x40 | 0x80;
/// Cislo stacku: pismo cisel WoW (`NumberFontNormal`, ARIALN s obrysem).
const STACK_FONT: &str = "Fonts\\ARIALN.TTF";

/// Ztmaveni ubehle casti ikony debuffu (hodiny).
const SWEEP: [f32; 4] = [0.0, 0.0, 0.0, 0.7];

/// Ikona role nepritele vlevo od ukazatele: velikost vuci vysce prouzku, mezera, a jak dlouho
/// role plati bez obnoveni (server ji posila kazde 4 s znovu, `HD_ROLE_RESEND_MS`).
const ROLE_SIZE: f32 = 1.4;
/// Hustá skupina nepřátel: nepřítel s plnými životy (ne v cíli, ne pod myší, bez debuffu) má jen nízký
/// proužek bez textu (podíl výšky), at shluk nezakryje obrazovku.
const COMPACT_H: f32 = 0.45;
/// Ukazatele se nepřekrývají: překrytý se posune nahoru (o svou výšku a mezeru), nejvýš o tolik pater.
/// Bližší jednotky a cíl zůstávají na místě. Pak se kreslí, i kdyby se překrýval.
const MAX_STACK: usize = 6;
const STACK_GAP_1080: f32 = 2.0;
const ROLE_GAP_1080: f32 = 3.0;
const ROLE_TTL: f64 = 12.0;
/// Role (`HdEnemyRole` v serveru): ikona a barva okraje. Lebka bosse je obrazek bez ramecku.
const ROLE_ICONS: [(char, &str, [f32; 4]); 5] = [
    (
        'S',
        "Interface\\Icons\\INV_Sword_04",
        [0.75, 0.75, 0.78, 1.0],
    ),
    (
        'D',
        "Interface\\Icons\\INV_Shield_06",
        [0.35, 0.6, 1.0, 1.0],
    ),
    (
        'H',
        "Interface\\Icons\\Spell_ChargePositive",
        [0.3, 1.0, 0.35, 1.0],
    ),
    (
        'A',
        "Interface\\Icons\\Ability_BackStab",
        [1.0, 0.15, 0.1, 1.0],
    ),
    (
        'B',
        "Interface\\TargetingFrame\\UI-TargetingFrame-Skull",
        [0.0, 0.0, 0.0, 0.0],
    ),
];

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<AuraTimes>()
        .init_resource::<EnemyRoles>()
        .add_systems(Update, draw_bars.in_set(UiQuadAppend));
}

/// Role nepratel ze serveru: radek `ER;guid,role;guid,role` v `SMSG_DBLOOKUP` (`core`
/// `HdGame::QueueRole`, guid hexadecimalne). Posila se pri vyvolani a kazde 4 s znovu.
#[derive(Resource, Default)]
pub(crate) struct EnemyRoles {
    /// guid -> (role, cas posledni zpravy v sekundach `Time::elapsed`).
    roles: HashMap<u64, (char, f64)>,
}

impl EnemyRoles {
    pub(crate) fn read(&mut self, rest: &str, now: f64) {
        for entry in rest.split(';') {
            let Some((guid, role)) = entry.split_once(',') else {
                continue;
            };
            let (Ok(guid), Some(role)) = (u64::from_str_radix(guid, 16), role.chars().next())
            else {
                continue;
            };
            self.roles.insert(guid, (role, now));
        }
    }

    fn get(&self, guid: u64) -> Option<char> {
        self.roles.get(&guid).map(|&(role, _)| role)
    }

    fn prune(&mut self, now: f64) {
        self.roles.retain(|_, &mut (_, at)| at + ROLE_TTL > now);
    }
}

/// Doba debuffu na nepratelich ze serveru: radek `AD;guid,slot,kouzlo,zbyva ms,celkem ms;...`
/// (guid hexadecimalne) v `SMSG_DBLOOKUP` (`core` `HdGame::QueueAuraTime`). Server posila jen
/// novou nebo obnovenou auru, zbytek se dopocita z hodin klienta.
#[derive(Resource, Default)]
pub(crate) struct AuraTimes {
    /// (guid, slot) -> (kouzlo, konec v sekundach `Time::elapsed`, celkova doba v sekundach).
    ends: HashMap<(u64, u8), (u32, f64, f32)>,
}

impl AuraTimes {
    pub(crate) fn read(&mut self, rest: &str, now: f64) {
        for entry in rest.split(';') {
            let f: Vec<&str> = entry.split(',').collect();
            let [guid, slot, spell, remain, max] = f[..] else {
                continue;
            };
            let (Ok(guid), Ok(slot), Ok(spell), Ok(remain), Ok(max)) = (
                u64::from_str_radix(guid, 16),
                slot.parse::<u8>(),
                spell.parse::<u32>(),
                remain.parse::<i32>(),
                max.parse::<i32>(),
            ) else {
                continue;
            };
            if remain <= 0 || max <= 0 {
                self.ends.remove(&(guid, slot));
                continue;
            }
            self.ends.insert(
                (guid, slot),
                (spell, now + f64::from(remain) / 1000.0, max as f32 / 1000.0),
            );
        }
    }

    /// Ubehla cast aury (0 cerstva, 1 vyprsela), kdyz ji server hlasil pro toto kouzlo.
    fn elapsed(&self, guid: u64, slot: u8, spell: u32, now: f64) -> Option<f32> {
        let &(s, end, max) = self.ends.get(&(guid, slot))?;
        if s != spell {
            return None;
        }
        let remain = (end - now) as f32;
        Some((1.0 - remain / max).clamp(0.0, 1.0))
    }

    fn prune(&mut self, now: f64) {
        // Vyprsele o chvili pozdeji (aura muze zmizet az s dalsim paketem).
        self.ends.retain(|_, &mut (_, end, _)| end + 2.0 > now);
    }
}

/// Ztmaveni ubehle casti ikony `icon` od 12 hodin ve smeru hodinovych rucicek, podil `elapsed`:
/// vejir z prostredku po obvodu ctverce, kazdy trojuhelnik jako ctyruhelnik s vlastnimi rohy
/// (posledni roh = predposledni).
fn push_sweep(quads: &mut UiQuads, icon: Rect, elapsed: f32) {
    if elapsed <= 0.0 {
        return;
    }
    let c = icon.center();
    let half = icon.half_size();
    let at = |angle: f32| {
        // Uhel od 12 hodin po smeru, obrazovka ma y dolu.
        let d = Vec2::new(angle.sin(), -angle.cos());
        let t = 1.0 / d.x.abs().max(d.y.abs());
        c + d * t * half
    };
    let end = elapsed.min(1.0) * std::f32::consts::TAU;
    // Body obvodu: zacatek, rohy ctverce (45, 135, 225, 315 stupnu) do `end`, konec.
    let mut points = vec![at(0.0)];
    for k in 0..4 {
        let corner = std::f32::consts::FRAC_PI_4 * (1.0 + 2.0 * k as f32);
        if corner < end {
            points.push(at(corner));
        }
    }
    points.push(at(end));
    for pair in points.windows(2) {
        quads.overlays.push(UiQuad {
            rect: icon,
            z_key: overlay_z::HD_BARS,
            color: SWEEP,
            blend: BlendMode::Blend,
            corners: Some([c, pair[0], pair[1], pair[1]]),
            ..default()
        });
    }
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
    spells: Option<Res<'w, crate::ui_action::Spells>>,
    aura_times: ResMut<'w, AuraTimes>,
    roles: ResMut<'w, EnemyRoles>,
}

/// Ikony kouzel a pismo pro cisla stacku.
#[derive(SystemParam)]
struct IconWorld<'w, 's> {
    assets: Option<ResMut<'w, benilla_assets::WorldAssets>>,
    images: ResMut<'w, Assets<Image>>,
    atlas: Option<ResMut<'w, crate::ui_text::UiFontAtlas>>,
    /// Ikona podle kouzla; `None` = kouzlo ikonu nema (nezkousi se znovu).
    cache: Local<'s, HashMap<u32, Option<Handle<Image>>>>,
    /// Ikony roli podle cesty.
    paths: Local<'s, HashMap<&'static str, Option<Handle<Image>>>>,
}

impl IconWorld<'_, '_> {
    fn icon(&mut self, spells: &crate::ui_action::Spells, spell: u32) -> Option<Handle<Image>> {
        if let Some(h) = self.cache.get(&spell) {
            return h.clone();
        }
        let assets = self.assets.as_deref_mut()?;
        let h = spells
            .catalog
            .get(spell)
            .and_then(|d| d.icon.clone())
            .filter(|p| !p.is_empty())
            .and_then(|path| assets.sprite_texture(&format!("{path}.blp"), &mut self.images));
        self.cache.insert(spell, h.clone());
        h
    }

    fn texture(&mut self, path: &'static str) -> Option<Handle<Image>> {
        if let Some(h) = self.paths.get(path) {
            return h.clone();
        }
        let assets = self.assets.as_deref_mut()?;
        let h = assets.sprite_texture(&format!("{path}.blp"), &mut self.images);
        self.paths.insert(path, h.clone());
        h
    }
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
    /// Životy a maximum pro text na proužku.
    hp: u32,
    max_hp: u32,
    ghost: f32,
    colour: ([f32; 3], [f32; 3]),
    lit: bool,
    /// Ikony aur (kouzlo, stacky), nejvic [`MAX_ICONS`].
    auras: Vec<(Handle<Image>, u8, Option<f32>)>,
    /// Role nepritele (ikona vlevo), [`EnemyRoles`].
    role: Option<char>,
    /// Nizky proužek bez textu ([`COMPACT_H`]).
    compact: bool,
}

/// Sirka a vyska prouzku na obrazovce.
fn bar_size(bar: &Bar, snap: &impl Fn(f32) -> f32) -> (f32, f32) {
    let w = snap((W_1080 * bar.scale).max(24.0));
    let k = if bar.compact { COMPACT_H } else { 1.0 };
    let h = snap((H_1080 * bar.scale * k).max(3.0));
    (w, h)
}

/// Obdelnik, ktery ukazatel zabira (prouzek, ikona role vlevo, rada aur nahore), pro rozestup.
fn bar_footprint(bar: &Bar, snap: &impl Fn(f32) -> f32) -> Rect {
    let (w, h) = bar_size(bar, snap);
    let left = bar.centre.x - w * 0.5;
    let top = bar.centre.y - h;
    let role = if bar.role.is_some() {
        (h * ROLE_SIZE * 1.25).max(10.0) + ROLE_GAP_1080 * bar.scale
    } else {
        0.0
    };
    let aura = if bar.auras.is_empty() {
        0.0
    } else {
        (ICON_1080 * bar.scale).max(11.0) + 2.0 * ICON_GAP_1080 * bar.scale
    };
    // Ikona role je vyssi nez prouzek: presahuje nahoru i dolu.
    let role_over = if bar.role.is_some() {
        (((h * ROLE_SIZE * 1.25).max(10.0)) - h).max(0.0) * 0.5
    } else {
        0.0
    };
    Rect::new(
        left - role,
        top - aura.max(role_over),
        left + w,
        top + h + role_over,
    )
}

/// Rozestup: od nejdulezitejsiho (cil, pak nejblizsi) kazdy ukazatel na prvni volne patro nad svym
/// mistem. `bars` je serazene od nejvzdalenejsiho (poradi kresleni).
fn spread_bars(bars: &mut [Bar], snap: &impl Fn(f32) -> f32, px: f32) {
    let gap = STACK_GAP_1080 * px;
    let mut order: Vec<usize> = (0..bars.len()).collect();
    order.sort_by(|&a, &b| {
        bars[b]
            .lit
            .cmp(&bars[a].lit)
            .then(bars[a].dist.total_cmp(&bars[b].dist))
    });
    let mut placed: Vec<Rect> = Vec::with_capacity(bars.len());
    for i in order {
        let base = bar_footprint(&bars[i], snap);
        let step = base.height() + gap;
        let mut chosen = 0.0;
        for level in 0..=MAX_STACK {
            let dy = -step * level as f32;
            let r = Rect::new(base.min.x, base.min.y + dy, base.max.x, base.max.y + dy);
            let free = placed.iter().all(|p| {
                r.min.x >= p.max.x + gap
                    || r.max.x + gap <= p.min.x
                    || r.min.y >= p.max.y + gap
                    || r.max.y + gap <= p.min.y
            });
            if free || level == MAX_STACK {
                chosen = if free { dy } else { 0.0 };
                break;
            }
        }
        bars[i].centre.y += chosen;
        placed.push(Rect::new(
            base.min.x,
            base.min.y + chosen,
            base.max.x,
            base.max.y + chosen,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_bars(
    mut world: BarWorld,
    anchors: Query<&BoneAttach>,
    poses: Query<&benilla_world::rig_anim::RigPose>,
    fallbacks: Query<&OverheadFallback>,
    globals: Query<&GlobalTransform>,
    mounts: Query<(), With<crate::entities::mount::MountChild>>,
    mut ghosts: Local<HashMap<Entity, Ghost>>,
    mut icons: IconWorld,
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
    let now = world.time.elapsed_secs_f64();
    world.aura_times.prune(now);
    world.roles.prune(now);
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
        // Aury: na nepriteli debuffy, na spojenci buffy, bez pasivnich a skrytych.
        let mut auras = Vec::new();
        if let Some(spells) = world.spells.as_deref() {
            for aura in store.0.unit_auras() {
                if (aura.slot >= FIRST_DEBUFF_SLOT) != attackable {
                    continue;
                }
                let shown = spells
                    .catalog
                    .get(aura.spell_id)
                    .is_some_and(|d| d.attributes & HIDDEN_AURA_ATTRS == 0);
                if !shown {
                    continue;
                }
                if let Some(h) = icons.icon(spells, aura.spell_id) {
                    let elapsed = if attackable {
                        world
                            .aura_times
                            .elapsed(guid.0, aura.slot, aura.spell_id, now)
                    } else {
                        None
                    };
                    auras.push((h, aura.stacks, elapsed));
                    if auras.len() == MAX_ICONS {
                        break;
                    }
                }
            }
        }
        // Spojenci jen zranene, v cili nebo s buffem.
        if !attackable && frac >= 1.0 && !lit && auras.is_empty() {
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
        let role = if attackable {
            world.roles.get(guid.0)
        } else {
            None
        };
        bars.push(Bar {
            compact: attackable && frac >= 1.0 && !lit && auras.is_empty() && role != Some('B'),
            dist,
            centre: screen,
            scale: px * (1.0 - t * (1.0 - FAR_SCALE)),
            health: frac,
            hp: health as u32,
            max_hp: max as u32,
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
            auras,
            role,
        });
    }
    ghosts.retain(|e, _| seen.contains(e));

    // Vzdalene prvni, blizsi se kresli pres ne.
    bars.sort_by(|a, b| b.dist.total_cmp(&a.dist));
    let snap = |v: f32| crate::vplates::device_snap(v, device);
    // Shluk nepratel: ukazatele od sebe (nahoru), at se neprekryvaji.
    spread_bars(&mut bars, &snap, px);
    for bar in bars {
        let (w, h) = bar_size(&bar, &snap);
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

        // Zivoty na prouzku: "920/1000", bile s cernym obrysem, uprostred.
        let text_h = snap(h * 0.95);
        if text_h >= HP_TEXT_MIN {
            if let Some(atlas) = icons.atlas.as_deref() {
                let mut e = atlas.lock();
                let glyphs = layout_text_quads(
                    &mut e,
                    &format!("{}/{}", bar.hp, bar.max_hp),
                    Rect::new(left - w, top - edge, left + 2.0 * w, top + h + edge),
                    [1.0, 1.0, 1.0, 1.0],
                    Justify {
                        h: JustifyH::Center,
                        v: JustifyV::Middle,
                    },
                    overlay_z::HD_BARS,
                    FontSpec {
                        path: Some(STACK_FONT),
                        height: Some(text_h),
                        outline: Outline::Normal,
                        monochrome: false,
                        spacing: 0.0,
                        shadow_x: 0.0,
                        alpha_gradient: None,
                        non_space_wrap: false,
                        max_lines: 0,
                    },
                    TextSeat::Exact,
                );
                drop(e);
                quads.overlays.extend(glyphs);
            }
        }

        // Role nepritele vlevo od ukazatele: ikona s okrajem v barve role, lebka bosse bez nej.
        if let Some(&(_, path, border)) = bar
            .role
            .and_then(|r| ROLE_ICONS.iter().find(|(c, _, _)| *c == r))
        {
            if let Some(texture) = icons.texture(path) {
                let skull = border[3] == 0.0;
                let size = snap((h * ROLE_SIZE * if skull { 1.25 } else { 1.0 }).max(10.0));
                let gap = snap((ROLE_GAP_1080 * bar.scale).max(1.0));
                let right = outer.min.x - gap;
                let mid = (outer.min.y + outer.max.y) * 0.5;
                let icon = Rect::new(
                    right - size,
                    snap(mid - size * 0.5),
                    right,
                    snap(mid - size * 0.5) + size,
                );
                let (c0, c1) = if skull {
                    (0.0, 1.0)
                } else {
                    push(
                        &mut quads,
                        Rect::new(
                            icon.min.x - edge,
                            icon.min.y - edge,
                            icon.max.x + edge,
                            icon.max.y + edge,
                        ),
                        border,
                        None,
                    );
                    (ICON_CROP, 1.0 - ICON_CROP)
                };
                quads.overlays.push(UiQuad {
                    rect: icon,
                    z_key: overlay_z::HD_BARS,
                    texture: Some(texture),
                    uv: crate::ui_pass::UvRect {
                        corners: [[c0, c0], [c1, c0], [c1, c1], [c0, c1]],
                    },
                    color: [1.0, 1.0, 1.0, 1.0],
                    blend: BlendMode::Blend,
                    ..default()
                });
            }
        }

        // Ikony aur v rade uprostred nad ukazatelem, kazda s tmavym okrajem.
        if bar.auras.is_empty() {
            continue;
        }
        let size = snap((ICON_1080 * bar.scale).max(11.0));
        let gap = snap((ICON_GAP_1080 * bar.scale).max(1.0));
        let n = bar.auras.len() as f32;
        let row = n * size + (n - 1.0) * (gap + 2.0 * edge);
        let mut x = snap(bar.centre.x - row * 0.5);
        let bottom = outer.min.y - gap - edge;
        let (c0, c1) = (ICON_CROP, 1.0 - ICON_CROP);
        for (texture, stacks, elapsed) in &bar.auras {
            let icon = Rect::new(x, bottom - size, x + size, bottom);
            push(
                &mut quads,
                Rect::new(
                    icon.min.x - edge,
                    icon.min.y - edge,
                    icon.max.x + edge,
                    icon.max.y + edge,
                ),
                EDGE,
                None,
            );
            quads.overlays.push(UiQuad {
                rect: icon,
                z_key: overlay_z::HD_BARS,
                texture: Some(texture.clone()),
                uv: crate::ui_pass::UvRect {
                    corners: [[c0, c0], [c1, c0], [c1, c1], [c0, c1]],
                },
                color: [1.0, 1.0, 1.0, 1.0],
                blend: BlendMode::Blend,
                ..default()
            });
            // Hodiny: ubehla cast debuffu ztmavne, cislo stacku zustane pres ne citelne.
            if let Some(elapsed) = elapsed {
                push_sweep(&mut quads, icon, *elapsed);
            }
            // Stacky: bile cislo s cernym obrysem v pravem dolnim rohu, pres okraj ikony.
            if *stacks > 1 {
                if let Some(atlas) = icons.atlas.as_deref() {
                    let height = snap((size * 0.62).max(10.0));
                    let mut e = atlas.lock();
                    let glyphs = layout_text_quads(
                        &mut e,
                        &stacks.to_string(),
                        Rect::new(
                            icon.min.x - size,
                            icon.min.y,
                            icon.max.x + 2.0 * edge,
                            icon.max.y + edge,
                        ),
                        [1.0, 1.0, 1.0, 1.0],
                        Justify {
                            h: JustifyH::Right,
                            v: JustifyV::Bottom,
                        },
                        overlay_z::HD_BARS,
                        FontSpec {
                            path: Some(STACK_FONT),
                            height: Some(height),
                            outline: Outline::Thick,
                            monochrome: false,
                            spacing: 0.0,
                            shadow_x: 0.0,
                            alpha_gradient: None,
                            non_space_wrap: false,
                            max_lines: 0,
                        },
                        TextSeat::Exact,
                    );
                    drop(e);
                    quads.overlays.extend(glyphs);
                }
            }
            x += size + gap + 2.0 * edge;
        }
    }
}
