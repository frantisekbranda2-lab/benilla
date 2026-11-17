//! Hero Defense: zastupne texty v popisu kouzla hrdiny (`core/tools/heroes/spells.csv`), ktere zavisi
//! na zbrani hrace: `{zbran}` = poskozeni zbrane na dalku ("12-18"), `{dps}` = o kolik % zbran sili
//! schopnosti, ktere poskozeni zbrane nedavaji ("+24 %"; server `HD_RANGED_WEAPON_PCT_PER_DPS`
//! za bod DPS zbrane, `HdItems` v `hd_items.cpp`).

use std::borrow::Cow;

use crate::net::ObjectStore;

/// % poskozeni za bod DPS zbrane na dalku (server `HD_RANGED_WEAPON_PCT_PER_DPS`).
const RANGED_WEAPON_PCT_PER_DPS: f32 = 8.0;

/// Popis kouzla s hodnotami zbrane hrace; bez hrace (nebo bez zastupnych textu) beze zmeny, `?`.
pub(crate) fn expand<'a>(text: &'a str, me: Option<&ObjectStore>) -> Cow<'a, str> {
    if !text.contains("{zbran}") && !text.contains("{dps}") {
        return Cow::Borrowed(text);
    }
    let (damage, pct) = match me {
        Some(store) => {
            let f = &store.0;
            let min = f.unit_min_ranged_damage().unwrap_or(0.0);
            let max = f.unit_max_ranged_damage().unwrap_or(0.0);
            // DPS samotne zbrane jako server (prumer zaznamu predmetu za sekundu): z poskozeni na dalku
            // bez sily utoku (AP / 14 za sekundu rychlosti).
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
    Cow::Owned(text.replace("{zbran}", &damage).replace("{dps}", &pct))
}
