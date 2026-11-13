//! Hero Defense: vyber karty z knizky kouzel (`core/HERO_DEFENSE_SPELLBOOKS.md`).
//!
//! Karty jsou prevzate z Azeroth Survivors (`OutThisLife/azeroth-survivors-re`,
//! `crates/benilla-app/src/survivors/cards.rs` a `fill.rs`, MIT OR Apache-2.0): rozdaji se jedna po
//! druhe, naklani se za mysi, pod mysi se zvednou a pri vyberu vybuchnou. Barva je rarita podle
//! kvality itemu WoW.
//!
//! Nabidku posila server, addon ji preda pres most `benilla_ui::script::hero_defense_cards`
//! (`HeroDefense_CardsShow`), tady se vykresli, a vyber se zapise zpet do mostu, odkud si ho addon
//! vezme (`HeroDefense_CardPick`) a posle serveru. Hra pri vyberu bezi dal (hraje se ve vic lidech).
//! Klavesy 1 az 3 nejsou: sesilaji kouzla z listy.

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_render::ui_material::UiMaterial;

use benilla_ui::script::hero_defense_cards::{HdCard, HD_CARDS};

/// Barvy kvality itemu WoW (`ITEM_QUALITY_COLORS`).
const COMMON: Color = Color::srgb(1.0, 1.0, 1.0);
const UNCOMMON: Color = Color::srgb(0.118, 1.0, 0.0);
const RARE: Color = Color::srgb(0.0, 0.439, 0.867);
const EPIC: Color = Color::srgb(0.639, 0.208, 0.933);
const LEGENDARY: Color = Color::srgb(1.0, 0.502, 0.0);
const GOLD: Color = Color::srgb(1.0, 0.82, 0.0);
const DIM: Color = Color::srgb(0.6, 0.6, 0.6);

const CARD_W: f32 = 236.0;
const CARD_H: f32 = 330.0;
/// Sekundy mezi dopadem jedne karty a dalsi.
const DEAL_STAGGER: f32 = 0.09;
const DEAL_SECS: f32 = 0.38;
const PICK_SECS: f32 = 0.28;

const RARITY_NAMES: [&str; 5] = ["Common", "Uncommon", "Rare", "Epic", "Legendary"];
const RARITY_COLORS: [Color; 5] = [COMMON, UNCOMMON, RARE, EPIC, LEGENDARY];
const TITLES: [&str; 4] = ["KNIHA UTOKU", "KNIHA ZKAZY", "KNIHA PODPORY", "KNIHA OBRANY"];

fn rarity_color(r: u8) -> Color {
    RARITY_COLORS[usize::from(r.min(4))]
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(bevy::ui_render::UiMaterialPlugin::<FillMaterial>::default())
        .init_resource::<CardsUi>()
        .add_systems(
            Update,
            (sync_offer, choose_card, animate_cards, fill_icons).chain(),
        );
}

/// Stav okna: ktera nabidka z mostu je na obrazovce a probihajici vyber.
#[derive(Resource, Default)]
struct CardsUi {
    serial: u32,
    root: Option<Entity>,
    font: Option<Handle<Font>>,
    /// Vybrana karta a cas od kliknuti (karta vybuchne, pak se vyber zapise).
    pending: Option<(usize, f32)>,
    count: usize,
}

/// Pohyb jedne karty: rozdani, naklon k mysi, zvednuti, vybuch pri vyberu.
#[derive(Component)]
struct Card {
    index: usize,
    /// Sekundy od zacatku rozdavani teto karty; zaporne, dokud ceka na svuj tah.
    t: f32,
    hover: f32,
    tilt: Vec2,
    picked: Option<f32>,
    rarity: u8,
}

/// Lesk, ktery pri dopadu jednou prejede pres kartu.
#[derive(Component)]
struct CardShine(usize);

/// Pomalu se tocici zare za kartami.
#[derive(Component)]
struct Rays;

/// Ikona kouzla, doplni se z `SpellIcon.dbc`, jakmile jsou data klienta nactena.
#[derive(Component)]
struct SpellIcon(u32);

#[derive(Component)]
struct IconDone;

// ---- material (survivors `fill.rs`) ------------------------------------------------------------

/// Velke mekke prechody (licni strana karty, ztmaveni) s ditheringem proti pruhum.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct FillMaterial {
    #[uniform(0)]
    fill: FillUniform,
}

#[derive(Clone, Copy, ShaderType)]
struct FillUniform {
    a: Vec4,
    b: Vec4,
    c: Vec4,
    glow: Vec4,
    glow_at: Vec4,
    shape: Vec4,
}

impl UiMaterial for FillMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://benilla_app/shaders/hd_cards_fill.wgsl".into()
    }
}

fn lin(c: Color) -> Vec4 {
    c.to_linear().to_vec4()
}

impl FillMaterial {
    fn vertical(top: Color, mid: Color, mid_at: f32, bottom: Color, radius: f32) -> Self {
        Self {
            fill: FillUniform {
                a: lin(top),
                b: lin(mid),
                c: lin(bottom),
                glow: Vec4::ZERO,
                glow_at: Vec4::ZERO,
                shape: Vec4::new(radius, 0.0, mid_at, 0.0),
            },
        }
    }

    fn radial(centre: Color, edge: Color) -> Self {
        Self {
            fill: FillUniform {
                a: lin(centre),
                b: Vec4::ZERO,
                c: lin(edge),
                glow: Vec4::ZERO,
                glow_at: Vec4::ZERO,
                shape: Vec4::new(0.0, 1.0, 0.5, 0.0),
            },
        }
    }

    fn with_glow(mut self, color: Color, at: Vec2, radii: Vec2) -> Self {
        self.fill.glow = lin(color);
        self.fill.glow_at = Vec4::new(at.x, at.y, radii.x, radii.y);
        self
    }
}

// ---- texty ---------------------------------------------------------------------------------------

fn text(font: &Handle<Font>, s: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(s),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(color),
        TextShadow {
            offset: Vec2::splat(1.5),
            color: Color::BLACK,
        },
    )
}

fn centered(font: &Handle<Font>, s: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        text(font, s, size, color),
        TextLayout::new_with_justify(Justify::Center),
    )
}

/// Text ze serveru: `^` je novy radek, `>>` sipka.
fn display(s: &str) -> String {
    s.replace('^', "\n").replace(">>", "\u{00bb}")
}

// ---- okno ----------------------------------------------------------------------------------------

/// Nabidka z mostu na obrazovku: pri kazde zmene (nova nabidka, zavreni) se okno postavi znovu.
fn sync_offer(
    mut commands: Commands,
    mut ui: ResMut<CardsUi>,
    asset_server: Res<AssetServer>,
    spells: Option<Res<crate::ui_action::Spells>>,
    mut fills: ResMut<Assets<FillMaterial>>,
) {
    let (serial, open, category, cards) = {
        let Ok(b) = HD_CARDS.lock() else { return };
        if b.serial == ui.serial {
            return;
        }
        (b.serial, b.open, b.category, b.cards.clone())
    };
    ui.serial = serial;
    ui.pending = None;
    if let Some(root) = ui.root.take() {
        commands.entity(root).despawn();
    }
    if !open || cards.is_empty() {
        return;
    }
    let font = ui
        .font
        .get_or_insert_with(|| asset_server.load("mpq://Fonts/FRIZQT__.ttf"))
        .clone();
    ui.count = cards.len();
    let root = spawn_root(
        &mut commands,
        &font,
        category,
        &cards,
        spells.as_deref(),
        &mut fills,
    );
    ui.root = Some(root);
}

fn spawn_root(
    commands: &mut Commands,
    f: &Handle<Font>,
    category: u8,
    cards: &[HdCard],
    spells: Option<&crate::ui_action::Spells>,
    fills: &mut Assets<FillMaterial>,
) -> Entity {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            // Lehke ztmaveni: hra bezi dal a je videt.
            MaterialNode(
                fills.add(
                    FillMaterial::radial(Color::srgba(0.04, 0.025, 0.0, 0.45), Color::srgba(0.0, 0.0, 0.0, 0.7))
                        .with_glow(
                            Color::srgba(1.0, 0.75, 0.25, 0.18),
                            Vec2::splat(0.5),
                            Vec2::splat(700.0 / 768.0),
                        ),
                ),
            ),
            GlobalZIndex(420),
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|root| {
            root.spawn((
                Rays,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(1400.0),
                    height: Val::Px(1400.0),
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    margin: UiRect::new(Val::Px(-700.0), Val::Px(0.0), Val::Px(-700.0), Val::Px(0.0)),
                    ..default()
                },
                BackgroundGradient(vec![Gradient::Conic(ConicGradient::new(
                    UiPosition::CENTER,
                    (0..24)
                        .map(|i| {
                            let a = i as f32 / 24.0 * std::f32::consts::TAU;
                            let on = i % 2 == 0;
                            AngularColorStop::new(
                                if on {
                                    Color::srgba(1.0, 0.8, 0.3, 0.07)
                                } else {
                                    Color::srgba(1.0, 0.8, 0.3, 0.0)
                                },
                                a,
                            )
                        })
                        .collect(),
                ))]),
                UiTransform::default(),
                bevy::ui::FocusPolicy::Pass,
            ));
            let title = TITLES.get(usize::from(category)).copied().unwrap_or("KNIHA KOUZEL");
            root.spawn(centered(f, title, 52.0, GOLD));
            root.spawn(centered(f, "Vyber si kartu", 16.0, Color::srgb(0.85, 0.78, 0.6)));
            root.spawn(Node {
                column_gap: Val::Px(22.0),
                margin: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(28.0), Val::Px(16.0)),
                ..default()
            })
            .with_children(|row| {
                for (i, card) in cards.iter().enumerate() {
                    spawn_card(row, f, i, card, spells, fills);
                }
            });
            root.spawn(centered(f, "Klikni na kartu", 14.0, DIM));
        })
        .id()
}

fn spawn_card(
    row: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
    f: &Handle<Font>,
    i: usize,
    card: &HdCard,
    spells: Option<&crate::ui_action::Spells>,
    fills: &mut Assets<FillMaterial>,
) {
    let rc = rarity_color(card.rarity);
    // Nove kouzlo: jeho popis z klienta nad bonusem ze serveru.
    let mut desc = String::new();
    if card.is_new {
        if let Some(d) = spells.and_then(|s| s.catalog.get(card.spell)).and_then(|d| d.description.clone()) {
            desc.push_str(&d);
            desc.push_str("\n\n");
        }
    }
    desc.push_str(&display(&card.text));
    let tag_color = if card.is_new {
        Color::srgb(0.3, 1.0, 0.3)
    } else {
        Color::WHITE
    };

    row.spawn((
        Card {
            index: i,
            t: -(i as f32) * DEAL_STAGGER,
            hover: 0.0,
            tilt: Vec2::ZERO,
            picked: None,
            rarity: card.rarity,
        },
        Button,
        Interaction::default(),
        RelativeCursorPosition::default(),
        Node {
            width: Val::Px(CARD_W),
            height: Val::Px(CARD_H),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(18.0), Val::Px(14.0)),
            row_gap: Val::Px(6.0),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Px(10.0)),
            ..default()
        },
        BorderColor::all(rc),
        BoxShadow::new(rc.with_alpha(0.0), Val::Px(0.0), Val::Px(0.0), Val::Px(0.0), Val::Px(26.0)),
        UiTransform {
            translation: Val2::px(0.0, 260.0),
            scale: Vec2::splat(0.6),
            rotation: Rot2::degrees(-14.0),
        },
        Visibility::Hidden,
    ))
    .with_children(|c| {
        // Tmava licni strana osvetlena shora barvou rarity.
        let inner_h = CARD_H - 4.0;
        c.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                ..default()
            },
            MaterialNode(
                fills.add(
                    FillMaterial::vertical(
                        rc.mix(&Color::BLACK, 0.55).with_alpha(0.98),
                        Color::srgba(0.05, 0.04, 0.035, 0.97),
                        0.42,
                        Color::srgba(0.02, 0.015, 0.01, 0.98),
                        8.0 / inner_h,
                    )
                    .with_glow(
                        rc.with_alpha(0.45),
                        Vec2::new(0.5, 60.0 / inner_h),
                        Vec2::new((CARD_W - 4.0) / 2.0 / inner_h, 90.0 / inner_h),
                    ),
                ),
            ),
            bevy::ui::FocusPolicy::Pass,
        ));
        // Cislo karty a rarita nahore.
        c.spawn(Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|top| {
            top.spawn(text(f, format!("{}", i + 1), 15.0, DIM));
            top.spawn(text(f, RARITY_NAMES[usize::from(card.rarity.min(4))], 13.0, rc));
        });
        // Ikona v pozlacenem luzku.
        c.spawn((
            Node {
                width: Val::Px(74.0),
                height: Val::Px(74.0),
                margin: UiRect::vertical(Val::Px(6.0)),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BorderColor::all(rc.mix(&GOLD, 0.4)),
            BoxShadow::new(rc.with_alpha(0.55), Val::Px(0.0), Val::Px(0.0), Val::Px(2.0), Val::Px(14.0)),
        ))
        .with_children(|s| {
            s.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                ImageNode {
                    color: Color::srgba(0.0, 0.0, 0.0, 0.6),
                    ..default()
                },
                SpellIcon(card.spell),
            ));
        });
        c.spawn(centered(f, card.name.clone(), 21.0, rc.mix(&Color::WHITE, 0.25)));
        c.spawn(centered(f, display(&card.tag), 14.0, tag_color));
        // Puntiky: rank ted a ten, ktery vyber prida.
        if card.max_rank > 0 {
            c.spawn(Node {
                column_gap: Val::Px(4.0),
                margin: UiRect::vertical(Val::Px(2.0)),
                ..default()
            })
            .with_children(|pips| {
                for r in 0..card.max_rank {
                    let color = if r < card.rank {
                        GOLD
                    } else if r == card.rank {
                        rc
                    } else {
                        Color::srgba(1.0, 1.0, 1.0, 0.12)
                    };
                    pips.spawn((
                        Node {
                            width: Val::Px(14.0),
                            height: Val::Px(6.0),
                            border_radius: BorderRadius::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(color),
                    ));
                }
            });
        }
        c.spawn((
            Node {
                width: Val::Percent(80.0),
                height: Val::Px(1.0),
                margin: UiRect::vertical(Val::Px(4.0)),
                ..default()
            },
            BackgroundGradient::from(LinearGradient::to_right(vec![
                ColorStop::percent(rc.with_alpha(0.0), 0.0),
                ColorStop::percent(rc.with_alpha(0.8), 50.0),
                ColorStop::percent(rc.with_alpha(0.0), 100.0),
            ])),
        ));
        c.spawn(centered(f, desc, 13.0, Color::srgb(0.92, 0.9, 0.84)));
        // Lesk pri dopadu.
        c.spawn((
            CardShine(i),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(-40.0),
                bottom: Val::Px(-40.0),
                width: Val::Px(70.0),
                left: Val::Px(-120.0),
                ..default()
            },
            BackgroundGradient::from(LinearGradient::to_right(vec![
                ColorStop::percent(Color::srgba(1.0, 1.0, 1.0, 0.0), 0.0),
                ColorStop::percent(Color::srgba(1.0, 0.97, 0.85, 0.28), 50.0),
                ColorStop::percent(Color::srgba(1.0, 1.0, 1.0, 0.0), 100.0),
            ])),
            UiTransform::from_rotation(Rot2::degrees(18.0)),
            bevy::ui::FocusPolicy::Pass,
        ));
    });
}

fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

/// Klik na kartu: zacne jeji vybuch, vyber se zapise az po nem.
fn choose_card(mut ui: ResMut<CardsUi>, cards: Query<(&Card, &Interaction), Changed<Interaction>>) {
    if ui.root.is_none() || ui.pending.is_some() {
        return;
    }
    let pick = cards
        .iter()
        .find(|(c, i)| **i == Interaction::Pressed && c.t >= DEAL_SECS)
        .map(|(c, _)| c.index);
    if let Some(i) = pick.filter(|&i| i < ui.count) {
        ui.pending = Some((i, 0.0));
    }
}

/// Rozdani, naklon, zvednuti a vybuch (survivors `animate_cards`), v realnem case.
#[allow(clippy::type_complexity)]
fn animate_cards(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut ui: ResMut<CardsUi>,
    mut cards: Query<(
        &mut Card,
        &Interaction,
        &RelativeCursorPosition,
        &mut UiTransform,
        &mut BoxShadow,
        &mut Visibility,
    )>,
    mut shines: Query<(&CardShine, &mut Node)>,
    mut rays: Query<&mut UiTransform, (With<Rays>, Without<Card>)>,
) {
    if ui.root.is_none() {
        return;
    }
    let dt = time.delta_secs().min(0.05);
    let now = time.elapsed_secs();
    for mut r in &mut rays {
        r.rotation = Rot2::radians(now * 0.12);
        r.scale = Vec2::splat(1.0 + 0.03 * (now * 1.7).sin());
    }
    let chosen = ui.pending.map(|(i, _)| i);
    let mut shine_at = [None::<f32>; 4];
    for (mut card, interaction, cursor, mut tf, mut shadow, mut vis) in &mut cards {
        let card = &mut *card;
        card.t += dt;
        if card.t < 0.0 {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;
        let deal = (card.t / DEAL_SECS).clamp(0.0, 1.0);
        let e = ease_out_back(deal);
        let hot = *interaction != Interaction::None && chosen.is_none() && deal >= 1.0;
        let k = 1.0 - (-dt * 14.0).exp();
        let hv = card.hover;
        card.hover = hv + ((if hot { 1.0 } else { 0.0 }) - hv) * k;
        let want_tilt = if hot {
            cursor
                .normalized
                .unwrap_or_default()
                .clamp(Vec2::splat(-0.5), Vec2::splat(0.5))
        } else {
            Vec2::ZERO
        };
        let tl = card.tilt;
        card.tilt = tl + (want_tilt - tl) * k;
        let h = card.hover;
        // Vyber: vybrana karta naroste a zvedne se, ostatni odpadnou.
        let (pick_scale, pick_y, pick_fade) = match chosen {
            Some(c) if c == card.index => {
                let p = card.picked.get_or_insert(0.0);
                *p += dt;
                let u = (*p / PICK_SECS).clamp(0.0, 1.0);
                (1.0 + 0.18 * u, -30.0 * u, 0.0)
            }
            Some(_) => {
                let p = card.picked.get_or_insert(0.0);
                *p += dt;
                let u = (*p / PICK_SECS).clamp(0.0, 1.0);
                (1.0 - 0.15 * u, 140.0 * u * u, u)
            }
            None => (1.0, 0.0, 0.0),
        };
        // Naoko perspektivni naklon: karta se toci k strane mysi a nakloni se s ni.
        let lean = card.tilt.x * 7.0;
        let squash = 1.0 - card.tilt.x.abs() * 0.08;
        let bob = (now * 2.2 + card.index as f32 * 1.3).sin() * 1.5 * (1.0 - h);
        tf.translation = Val2::px(
            0.0,
            (1.0 - e) * 260.0 - 18.0 * h + card.tilt.y * 8.0 + bob + pick_y,
        );
        let s = (0.6 + 0.4 * e) * (1.0 + 0.06 * h) * pick_scale;
        tf.scale = Vec2::new(s * squash, s * (1.0 + card.tilt.y.abs() * 0.02));
        tf.rotation = Rot2::degrees((1.0 - e) * -14.0 + lean);
        let rc = rarity_color(card.rarity);
        if let Some(sh) = shadow.0.first_mut() {
            sh.color = rc.with_alpha((0.25 + 0.6 * h) * (1.0 - pick_fade));
            sh.blur_radius = Val::Px(18.0 + 22.0 * h);
            sh.spread_radius = Val::Px(2.0 + 4.0 * h);
            sh.y_offset = Val::Px(6.0 + 10.0 * h);
        }
        if pick_fade >= 1.0 {
            *vis = Visibility::Hidden;
        }
        if card.index < shine_at.len() {
            shine_at[card.index] = Some(card.t);
        }
    }
    for (shine, mut node) in &mut shines {
        let Some(t) = shine_at.get(shine.0).copied().flatten() else {
            continue;
        };
        let u = ((t - DEAL_SECS * 0.6) / 0.5).clamp(0.0, 1.0);
        node.left = Val::Px(-120.0 + u * (CARD_W + 200.0));
    }
    // Po vybuchu: vyber do mostu pro addon, okno zavrit.
    if let Some((i, t)) = ui.pending.as_mut() {
        *t += dt;
        if *t >= PICK_SECS {
            let i = *i;
            ui.pending = None;
            if let Some(root) = ui.root.take() {
                commands.entity(root).despawn();
            }
            if let Ok(mut b) = HD_CARDS.lock() {
                b.picked = Some(i as u32);
                b.open = false;
                b.cards.clear();
                b.serial = b.serial.wrapping_add(1);
                ui.serial = b.serial;
            }
        }
    }
}

/// Ikony z `SpellIcon.dbc`, jakmile jsou data klienta a cteni assetu nahore (survivors `fill_icons`).
fn fill_icons(
    mut commands: Commands,
    mut slots: Query<(Entity, &mut ImageNode, &SpellIcon), Without<IconDone>>,
    spells: Option<Res<crate::ui_action::Spells>>,
    mut assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<std::collections::HashMap<u32, Handle<Image>>>,
) {
    let (Some(spells), Some(assets)) = (spells.as_deref(), assets.as_deref_mut()) else {
        return;
    };
    for (e, mut img, icon) in &mut slots {
        let handle = match cache.get(&icon.0) {
            Some(h) => Some(h.clone()),
            None => {
                let path = spells
                    .catalog
                    .get(icon.0)
                    .and_then(|d| d.icon.clone())
                    .unwrap_or_else(|| "Interface\\Icons\\INV_Misc_QuestionMark".into());
                let h = assets.sprite_texture(&format!("{path}.blp"), &mut images);
                if let Some(h) = &h {
                    cache.insert(icon.0, h.clone());
                }
                h
            }
        };
        if let Some(h) = handle {
            img.image = h;
            img.color = Color::WHITE;
        }
        commands.entity(e).insert(IconDone);
    }
}
