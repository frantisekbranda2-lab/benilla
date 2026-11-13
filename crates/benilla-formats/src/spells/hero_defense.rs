//! Hero Defense: nazvy, ikony a popisy kouzel, kterymi hrac stavi veze.
//!
//! Server stavi veze pres existujici NPC kouzla klienta (mireni na zem, bez ceny), ktera si
//! v `spell_template` prepise na vyvolani veze (`sql/hero_defense/hero_defense_towers.sql`).
//! Klient by u nich ukazal puvodni nazev a ikonu (Meteor, Blizzard...), proto je tu prepisuje.
//! Cisla kouzel musi sedet s `HD_LINES` v serveru (`src/scripts/custom/hd_defs.h`): 3 rasy po
//! 7 liniich vezi, kouzlo stavi zakladni vez linie (vylepseni jsou v panelu veze v addonu).

use std::collections::HashMap;

use super::SpellDisplay;

/// `SpellRange.dbc` radek 13: 0 az 50 000 yd (Eye of Kilrogg).
const HD_RANGE_ANYWHERE: u32 = 13;

/// (kouzlo, nazev zakladni veze, rasa, ikona); poradi jako `HD_LINES` v serveru.
const TOWER_SPELLS: [(u32, &str, &str, &str); 21] = [
    (13808, "Puskar", "Human", "INV_Weapon_Rifle_01"),
    (25823, "Knez", "Human", "Spell_Holy_Renew"),
    (8333, "Mechano-tank", "Human", "INV_Misc_Bomb_05"),
    (4071, "Kapitan", "Human", "Ability_Warrior_DefensiveStance"),
    (4072, "Mag boure", "Human", "Spell_Nature_LightningBolt"),
    (19805, "Mrazivy mag", "Human", "Spell_Frost_FrostBolt02"),
    (4100, "Sermir", "Human", "INV_Sword_04"),
    (26462, "Sentinel", "Night Elf", "Ability_EyeOfTheOwl"),
    (25019, "Wisp", "Night Elf", "Spell_Nature_HealingTouch"),
    (26540, "Lovkyne", "Night Elf", "Ability_UpgradeMoonGlaive"),
    (21688, "Moonkin", "Night Elf", "Spell_Nature_StarFall"),
    (22700, "Treant", "Night Elf", "Spell_Nature_ForceOfNature"),
    (27661, "Chimera", "Night Elf", "Ability_Hunter_Pet_WindSerpent"),
    (15252, "Medvedi druid", "Night Elf", "Ability_Racial_BearForm"),
    (19717, "Vrhac sekyr", "Orc", "INV_Axe_04"),
    (23105, "Saman", "Orc", "Spell_Nature_HealingWaveGreater"),
    (24340, "Golem", "Orc", "INV_Ammo_FireTar"),
    (3720, "Temny mag", "Orc", "Spell_Shadow_ShadowBolt"),
    (3823, "Grunt", "Orc", "Ability_Warrior_Cleave"),
    (23056, "Tauren", "Orc", "Ability_Warrior_Charge"),
    (897, "Wind Rider", "Orc", "Ability_Hunter_Pet_Vulture"),
];

pub(super) fn apply(spells: &mut HashMap<u32, SpellDisplay>) {
    for (id, name, race, icon) in TOWER_SPELLS {
        let Some(spell) = spells.get_mut(&id) else {
            continue;
        };
        spell.name = name.to_string();
        spell.rank = Some(race.to_string());
        spell.icon = Some(format!("Interface\\Icons\\{icon}"));
        spell.description = Some(format!(
            "Postavi jednotku {name} na pole mrizky areny. Ve vlne jde naproti nepratelum, po kole se vrati. Vylepseni a prodej: vyber ji jako cil."
        ));
        // Stavba odkudkoli, i z "ocicka" (spectate) daleko od hrdiny: SpellRange 13 "Anywhere".
        // Server ma stejny dosah v hero_defense_towers.sql.
        spell.range_index = HD_RANGE_ANYWHERE;
    }

    // Svitek zkusenosti (item 90200 z truhly po vlne): kouzlo-nosic 5202 (MHTest01), server
    // ho prepise v hero_defense_loot.sql. Text je radek "Use:" v tooltipu svitku.
    if let Some(spell) = spells.get_mut(&XP_SCROLL_SPELL) {
        spell.name = "Svitek zkusenosti".to_string();
        spell.rank = None;
        spell.icon = Some("Interface\\Icons\\INV_Scroll_03".to_string());
        spell.description = Some("Prida 25 % zkusenosti potrebnych do dalsiho levelu.".to_string());
    }

    // Strely vezi, ktere jsou v klientu moc rychle. Musi sedet se serverem
    // (hero_defense_towers.sql, cast "rychlost strel"): server podle ni pocita, kdy zasah prijde.
    for (id, speed) in HD_SPELL_SPEEDS {
        if let Some(spell) = spells.get_mut(&id) {
            spell.speed = speed;
        }
    }

    // Vizualni aury vezi (debuffy na neprateli, buffy vezi): server je prepisuje na neskodnou
    // auru (hero_defense_towers.sql, cast "vizualni aury"), tady dostanou jmeno, ikonu a text.
    for (id, name, icon, text) in HD_VISUAL_AURAS {
        if let Some(spell) = spells.get_mut(&id) {
            spell.name = name.to_string();
            spell.rank = None;
            spell.icon = Some(format!("Interface\\Icons\\{icon}"));
            spell.description = Some(text.to_string());
            spell.aura_description = Some(text.to_string());
        }
    }

    // Kouzla hrdinu (vygenerovana z core/tools/heroes/spells.csv).
    super::hero_defense_heroes::apply(spells);
}

/// Rychlost strel vezi (yd/s): Axe Flurry, sekery Watch Tower a glevy (puvodne 40), Boulder,
/// katapult, delo a Treant (puvodne 30).
const HD_SPELL_SPEEDS: [(u32, f32); 2] = [(24020, 20.0), (9483, 15.0)];

/// Vizualni aury vezi: (kouzlo, jmeno, ikona, text). Cisla jako `HdVisual` v serveru (`hd_defs.h`).
const HD_VISUAL_AURAS: [(u32, &str, &str, &str); 11] = [
    (12881, "Jed", "Spell_Nature_CorrosiveBreath", "Jed: poskozeni prirodou v case."),
    (28330, "Horeni", "Spell_Fire_Immolation", "Hori: poskozeni ohnem v case."),
    (23072, "Temna rana", "Spell_Shadow_ShadowWordPain", "Stinove poskozeni v case nebo vyssi prijate poskozeni."),
    (7742, "Mraz", "Spell_Frost_FrostBolt02", "Zpomaleny mrazem."),
    (6388, "Zpomaleni", "Spell_Nature_StrengthOfEarthTotem02", "Zpomaleny."),
    (28356, "Nemoc", "Spell_Shadow_CallofBone", "Dostava vic poskozeni."),
    (24152, "Korinky", "Spell_Nature_StrangleVines", "Nemuze se hnout."),
    (10017, "Zamrznuti", "Spell_Frost_FrostNova", "Zamrzly, nemuze se hnout."),
    (18970, "Omraceni", "Spell_Frost_Stun", "Omraceny."),
    (22751, "Bloodlust", "Spell_Nature_BloodLust", "Rychlejsi utok."),
    (22108, "Arkanove pretizeni", "Spell_Nature_WispSplode", "Kazdy stack: +10 % poskozeni a dosahu Arcane Explosion."),
];

/// Kouzlo svitku zkusenosti, musi sedet s hero_defense_loot.sql.
const XP_SCROLL_SPELL: u32 = 5202;
