//! Hero Defense, not in 1.12: the spellbook card picker's bridge between the addon and the app.
//!
//! The server sends an offer of up to three cards (`core/HERO_DEFENSE_SPELLBOOKS.md`); the addon
//! parses it and hands it here, and the app (`benilla-app` `hero_defense/cards.rs`) draws the cards
//! and records the pick, which the addon reads back and sends to the server:
//!
//! - `HeroDefense_CardsBegin(category)` starts a new offer (0 single target .. 3 defense);
//! - `HeroDefense_CardsAdd(spell, rarity, isNew, rank, maxRank, name, tag, text)` adds a card
//!   (rarity 0 Common .. 4 Legendary, `text` lines separated by `^`, `>>` drawn as an arrow);
//! - `HeroDefense_CardsShow()` shows the offer, `HeroDefense_CardsHide()` closes it;
//! - `HeroDefense_CardPick()` returns the picked card's index (0-based) once, else nil;
//! - `HeroDefense_SpellIcon(spellId)` returns the spell's icon texture path
//!   (`Interface\Icons\...`, from `SpellIcon.dbc`), or nil until the app has looked it up: the
//!   first call queues the id, the app answers it on its next frame ([`HD_SPELL_ICONS`]). The
//!   addon draws its own cards with it (the app's card renderer stays for other callers).

use std::sync::Mutex;

use mlua::{Lua, Value};

/// One card as the server describes it.
#[derive(Clone, Debug, Default)]
pub struct HdCard {
    pub spell: u32,
    pub rarity: u8,
    pub is_new: bool,
    pub rank: u8,
    pub max_rank: u8,
    pub name: String,
    pub tag: String,
    pub text: String,
}

/// The offer on screen and the pick waiting for the addon.
#[derive(Debug)]
pub struct HdCardsBridge {
    /// Bumped on every show and hide; the app rebuilds when it changes.
    pub serial: u32,
    pub open: bool,
    pub category: u8,
    pub cards: Vec<HdCard>,
    pub picked: Option<u32>,
    staging: Vec<HdCard>,
    staging_category: u8,
}

impl HdCardsBridge {
    const fn new() -> Self {
        Self {
            serial: 0,
            open: false,
            category: 0,
            cards: Vec::new(),
            picked: None,
            staging: Vec::new(),
            staging_category: 0,
        }
    }
}

/// Shared between the script host and the app.
pub static HD_CARDS: Mutex<HdCardsBridge> = Mutex::new(HdCardsBridge::new());

/// Spell icons for `HeroDefense_SpellIcon`: ids the addon asked for and the answers the app wrote.
#[derive(Debug, Default)]
pub struct HdSpellIcons {
    pub requests: Vec<u32>,
    pub icons: std::collections::HashMap<u32, String>,
}

pub static HD_SPELL_ICONS: std::sync::LazyLock<Mutex<HdSpellIcons>> =
    std::sync::LazyLock::new(|| Mutex::new(HdSpellIcons::default()));

fn num(lua: &Lua, v: Value) -> u32 {
    lua.coerce_number(v).ok().flatten().map_or(0, |n| n.max(0.0) as u32)
}

fn text(lua: &Lua, v: Value) -> String {
    lua.coerce_string(v)
        .ok()
        .flatten()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Register the five globals.
pub(super) fn install(lua: &Lua) -> mlua::Result<()> {
    let g = lua.globals();

    g.set(
        "HeroDefense_CardsBegin",
        lua.create_function(|lua, category: Value| {
            if let Ok(mut b) = HD_CARDS.lock() {
                b.staging.clear();
                b.staging_category = num(lua, category).min(255) as u8;
            }
            Ok(())
        })?,
    )?;

    g.set(
        "HeroDefense_CardsAdd",
        lua.create_function(
            |lua, args: (Value, Value, Value, Value, Value, Value, Value, Value)| {
                let (spell, rarity, is_new, rank, max_rank, name, tag, body) = args;
                let card = HdCard {
                    spell: num(lua, spell),
                    rarity: num(lua, rarity).min(4) as u8,
                    is_new: num(lua, is_new) != 0,
                    rank: num(lua, rank).min(255) as u8,
                    max_rank: num(lua, max_rank).min(20) as u8,
                    name: text(lua, name),
                    tag: text(lua, tag),
                    text: text(lua, body),
                };
                if let Ok(mut b) = HD_CARDS.lock() {
                    if b.staging.len() < 5 {
                        b.staging.push(card);
                    }
                }
                Ok(())
            },
        )?,
    )?;

    g.set(
        "HeroDefense_CardsShow",
        lua.create_function(|_, ()| {
            if let Ok(mut b) = HD_CARDS.lock() {
                b.cards = std::mem::take(&mut b.staging);
                b.category = b.staging_category;
                b.open = !b.cards.is_empty();
                b.picked = None;
                b.serial = b.serial.wrapping_add(1);
            }
            Ok(())
        })?,
    )?;

    g.set(
        "HeroDefense_CardsHide",
        lua.create_function(|_, ()| {
            if let Ok(mut b) = HD_CARDS.lock() {
                if b.open {
                    b.open = false;
                    b.cards.clear();
                    b.serial = b.serial.wrapping_add(1);
                }
            }
            Ok(())
        })?,
    )?;

    g.set(
        "HeroDefense_SpellIcon",
        lua.create_function(|lua, spell: Value| {
            let id = num(lua, spell);
            let Ok(mut s) = HD_SPELL_ICONS.lock() else {
                return Ok(None);
            };
            if let Some(path) = s.icons.get(&id) {
                return Ok(Some(path.clone()));
            }
            if id != 0 && !s.requests.contains(&id) {
                s.requests.push(id);
            }
            Ok(None)
        })?,
    )?;

    g.set(
        "HeroDefense_CardPick",
        lua.create_function(|_, ()| {
            let picked = HD_CARDS.lock().ok().and_then(|mut b| b.picked.take());
            Ok(picked.map(f64::from))
        })?,
    )?;

    Ok(())
}
