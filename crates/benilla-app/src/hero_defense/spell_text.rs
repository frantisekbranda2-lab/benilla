//! Hero Defense: zastupne texty v popisu kouzla hrdiny (`core/tools/heroes/spells.csv`), ktere zavisi
//! na hraci:
//! - `{zbran}` = poskozeni zbrane na dalku ("12-18"), `{dps}` = o kolik % zbran sili schopnosti, ktere
//!   poskozeni zbrane nedavaji ("+24 %"; server `HD_RANGED_WEAPON_PCT_PER_DPS` za bod DPS zbrane,
//!   `HdItems` v `hd_items.cpp`).
//! - `{dmg:A-B}` / `{dmg:A}` = zakladni poskozeni ("15-19").
//! - `{dmgw:N}` = poskozeni zbrane na blizko + N ("12-18 + 8").
//! - `{bonus}` = rozpis bonusu ("+56 % (level +32 %, zbran +24 %)"); popisy ho od changelogu 67 nemaji.
//!
//! Bonus posila server (radek `HB;celkem;level;zbran`, `HdGame::SendHeroBonus`): level, zbran Warriora,
//! talenty, aury vezi a trinkety, tedy totez co `HdGame::DamageBonusPct`.
//!
//! Pod popis kouzla hrdiny ([`damage_lines`], jen tooltip kouzla) pripoji rozpis poskozeni ze serveru
//! (radek `SD;kouzlo;radek~radek~...`, `HdGame::SendSpellDamage`): vysledek bile, pod nim zelene, z ceho
//! se sklada (zaklad, sila kouzel x koeficient, zbran, vylepseni, level, talenty...). Kriticky zasah,
//! zranitelnost a brneni cile v nem nejsou.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::net::ObjectStore;

/// % poskozeni za bod DPS zbrane na dalku (server `HD_RANGED_WEAPON_PCT_PER_DPS`).
const RANGED_WEAPON_PCT_PER_DPS: f32 = 8.0;

/// Bonus hrdiny ze serveru: (celkem, level, zbran) v %.
static HERO_BONUS: Mutex<Option<(i32, i32, i32)>> = Mutex::new(None);

/// Rozpis poskozeni kouzel ze serveru: kouzlo -> radky (`~` mezi nimi).
static SPELL_DAMAGE: Mutex<Option<HashMap<u32, String>>> = Mutex::new(None);

/// Barva vysledku a rozpisu (zelena jako bonusy na predmetech).
const RESULT_COLOR: &str = "|cffffffff";
const PART_COLOR: &str = "|cff1eff00";

/// Radek `SD;kouzlo;text` ze serveru (bez `SD;`).
pub(crate) fn read_damage(rest: &str) {
    let Some((spell, text)) = rest.split_once(';') else {
        return;
    };
    let Ok(spell) = spell.trim().parse::<u32>() else {
        return;
    };
    if let Ok(mut map) = SPELL_DAMAGE.lock() {
        map.get_or_insert_with(HashMap::new)
            .insert(spell, text.to_string());
    }
}

/// Rozpis poskozeni kouzla pod popis (s prazdnym radkem pred nim), kdyz ho server poslal.
pub(crate) fn damage_lines(spell: u32) -> Option<String> {
    let map = SPELL_DAMAGE.lock().ok()?;
    let text = map.as_ref()?.get(&spell)?;
    let mut out = String::from("\n");
    for line in text.split('~').filter(|l| !l.is_empty()) {
        // Radky rozpisu zacinaji "+ ", vysledek ne.
        let color = if line.starts_with('+') {
            PART_COLOR
        } else {
            RESULT_COLOR
        };
        out.push('\n');
        out.push_str(color);
        // "|" by v textu zacal kod barvy.
        out.push_str(&line.replace('|', "||"));
        out.push_str("|r");
    }
    Some(out)
}

/// Radek `HB;celkem;level;zbran` ze serveru (bez `HB;`).
pub(crate) fn read_bonus(rest: &str) {
    let mut f = rest.split(';').map(|s| s.trim().parse::<i32>());
    if let (Some(Ok(total)), Some(Ok(level)), Some(Ok(weapon))) = (f.next(), f.next(), f.next()) {
        if let Ok(mut b) = HERO_BONUS.lock() {
            *b = Some((total, level, weapon));
        }
    }
}

fn bonus() -> Option<(i32, i32, i32)> {
    HERO_BONUS.lock().ok().and_then(|b| *b)
}

fn range(a: i32, b: i32) -> String {
    if a == b {
        a.to_string()
    } else {
        format!("{a}-{b}")
    }
}

/// `{dmg:A-B}`: "A-B" (vysledek s bonusy je v rozpisu pod popisem).
fn dmg_text(arg: &str) -> String {
    let (a, b) = match arg.split_once('-') {
        Some((a, b)) => (a.trim().parse::<f32>().ok(), b.trim().parse::<f32>().ok()),
        None => {
            let v = arg.trim().parse::<f32>().ok();
            (v, v)
        }
    };
    let (Some(a), Some(b)) = (a, b) else {
        return arg.to_string();
    };
    range(a.round() as i32, b.round() as i32)
}

/// `{dmgw:N}`: poskozeni zbrane na blizko + N (vysledek s bonusy je v rozpisu pod popisem).
fn dmgw_text(arg: &str, me: Option<&ObjectStore>) -> String {
    let extra = arg.trim().parse::<f32>().unwrap_or(0.0);
    let Some(store) = me else {
        return format!("poskozeni zbrane + {}", extra.round() as i32);
    };
    let f = &store.0;
    let min = f.unit_min_damage().unwrap_or(0.0);
    let max = f.unit_max_damage().unwrap_or(0.0);
    let weapon = range(min.round() as i32, max.round() as i32);
    format!("{weapon} + {}", extra.round() as i32)
}

/// `{bonus}`: rozpis bonusu hrdiny.
fn bonus_text() -> String {
    let Some((total, level, weapon)) = bonus() else {
        return "?".to_string();
    };
    let mut parts = vec![format!("level +{level} %")];
    if weapon != 0 {
        parts.push(format!("zbran +{weapon} %"));
    }
    let other = total - level - weapon;
    if other != 0 {
        parts.push(format!("ostatni {other:+} %"));
    }
    format!("{total:+} % ({})", parts.join(", "))
}

/// Nahradi `{tag:arg}` funkci `f(arg)`.
fn replace_tagged(text: &str, tag: &str, mut f: impl FnMut(&str) -> String) -> String {
    let open = format!("{{{tag}:");
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find(&open) {
        out.push_str(&rest[..i]);
        let after = &rest[i + open.len()..];
        let Some(end) = after.find('}') else {
            out.push_str(&rest[i..]);
            return out;
        };
        out.push_str(&f(&after[..end]));
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Popis kouzla s hodnotami hrace; bez hrace (nebo bez zastupnych textu) beze zmeny, `?`.
pub(crate) fn expand<'a>(text: &'a str, me: Option<&ObjectStore>) -> Cow<'a, str> {
    if !text.contains('{') {
        return Cow::Borrowed(text);
    }
    let mut out = text.to_string();
    if out.contains("{zbran}") || out.contains("{dps}") {
        let (damage, pct) = match me {
            Some(store) => {
                let f = &store.0;
                let min = f.unit_min_ranged_damage().unwrap_or(0.0);
                let max = f.unit_max_ranged_damage().unwrap_or(0.0);
                // DPS samotne zbrane jako server (prumer zaznamu predmetu za sekundu): z poskozeni na
                // dalku bez sily utoku (AP / 14 za sekundu rychlosti).
                let speed = f.unit_ranged_attack_time().unwrap_or(0) as f32 / 1000.0;
                let (pos, neg) = f.unit_ranged_attack_power_mods();
                let ap = (f.unit_ranged_attack_power().unwrap_or(0) + i32::from(pos) + i32::from(neg))
                    .max(0) as f32;
                let dps = if speed > 0.0 && max > 0.0 {
                    ((min + max) * 0.5 / speed - ap / 14.0).max(0.0)
                } else {
                    0.0
                };
                (
                    format!("{}-{}", min.round() as i32, max.round() as i32),
                    format!("+{} %", (dps * RANGED_WEAPON_PCT_PER_DPS).round() as i32),
                )
            }
            None => ("?".to_string(), "?".to_string()),
        };
        out = out.replace("{zbran}", &damage).replace("{dps}", &pct);
    }
    if out.contains("{dmgw:") {
        out = replace_tagged(&out, "dmgw", |a| dmgw_text(a, me));
    }
    if out.contains("{dmg:") {
        out = replace_tagged(&out, "dmg", dmg_text);
    }
    if out.contains("{bonus}") {
        out = out.replace("{bonus}", &bonus_text());
    }
    Cow::Owned(out)
}
