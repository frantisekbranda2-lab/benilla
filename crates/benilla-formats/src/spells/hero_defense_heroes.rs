//! Hero Defense: kouzla hrdinu z knizek (nazvy, popisy, cena, cas, dosah, hodnoty efektu, bit rodiny).
//!
//! VYGENEROVANO z `core/tools/heroes/spells.csv` (`node tools/heroes/gen_spells.js`), rucne neupravovat.
//! Server ma stejna cisla v `core/sql/hero_defense/hero_defense_hero_spells.sql`.

use std::collections::HashMap;

use super::SpellDisplay;

fn set_effect(s: &mut SpellDisplay, i: usize, base_points: i32, die_sides: i32) {
    s.effect_base_points[i] = base_points;
    s.effect_die_sides[i] = die_sides;
    s.effect_base_dice[i] = 1;
}

fn set_radius(s: &mut SpellDisplay, radius_index: u32) {
    for r in s.effect_radius_index.iter_mut() {
        if *r > 0 {
            *r = radius_index;
        }
    }
}

pub(super) fn apply(spells: &mut HashMap<u32, SpellDisplay>) {
    // Heroic Strike (single)
    if let Some(s) = spells.get_mut(&78) {
        s.name = "Heroic Strike".to_string();
        s.rank = None;
        s.description = Some("Uder zbrani a navic 8 poskozeni.".to_string());
        s.aura_description = Some("Uder zbrani a navic 8 poskozeni.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 4;
        s.spell_family_flags |= 1u64 << 32;
        s.mana_cost = 3;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 1000;
        s.category_recovery_ms = 0;
        s.category = 0;
        set_effect(s, 0, 7, 1);
        s.attributes = 327696;
    }
    // Rend (single)
    if let Some(s) = spells.get_mut(&772) {
        s.name = "Rend".to_string();
        s.rank = None;
        s.description = Some("Krvaceni: 4 poskozeni kazde 3 s po 15 s.".to_string());
        s.aura_description = Some("Krvaceni: 4 poskozeni kazde 3 s po 15 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 4;
        s.spell_family_flags |= 1u64 << 33;
        s.mana_cost = 6;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 3000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 8;
        set_effect(s, 0, 3, 1);
    }
    // Mortal Strike (single)
    if let Some(s) = spells.get_mut(&12294) {
        s.name = "Mortal Strike".to_string();
        s.rank = None;
        s.description = Some("Silny uder zbrani za navic 20 poskozeni.".to_string());
        s.aura_description = Some("Silny uder zbrani za navic 20 poskozeni.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 4;
        s.spell_family_flags |= 1u64 << 34;
        s.mana_cost = 12;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 6000;
        s.category_recovery_ms = 0;
        s.category = 0;
        set_effect(s, 1, 19, 1);
    }
    // Cleave (aoe)
    if let Some(s) = spells.get_mut(&845) {
        s.name = "Cleave".to_string();
        s.rank = None;
        s.description = Some("Uder zasahne az 3 nepratele pred tebou a da navic 15 poskozeni.".to_string());
        s.aura_description = Some("Uder zasahne az 3 nepratele pred tebou a da navic 15 poskozeni.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 4;
        s.spell_family_flags |= 1u64 << 35;
        s.mana_cost = 8;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 3000;
        s.category_recovery_ms = 0;
        s.category = 0;
        set_effect(s, 0, 14, 1);
        s.attributes = 327696;
    }
    // Whirlwind (aoe)
    if let Some(s) = spells.get_mut(&1680) {
        s.name = "Whirlwind".to_string();
        s.rank = None;
        s.description = Some("Vir: zasahne vsechny nepratele do 8 yd zbrani a navic 20 poskozeni.".to_string());
        s.aura_description = Some("Vir: zasahne vsechny nepratele do 8 yd zbrani a navic 20 poskozeni.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 4;
        s.spell_family_flags |= 1u64 << 36;
        s.mana_cost = 15;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 8000;
        s.category_recovery_ms = 0;
        s.category = 0;
        set_radius(s, 14);
        set_effect(s, 0, 19, 1);
    }
    // Thunder Clap (aoe)
    if let Some(s) = spells.get_mut(&6343) {
        s.name = "Thunder Clap".to_string();
        s.rank = None;
        s.description = Some("Uder do zeme: 10 poskozeni vsem do 8 yd a zpomali je o 30 % na 4 s.".to_string());
        s.aura_description = Some("Uder do zeme: 10 poskozeni vsem do 8 yd a zpomali je o 30 % na 4 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 4;
        s.spell_family_flags |= 1u64 << 37;
        s.mana_cost = 10;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 6000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 35;
        set_radius(s, 14);
        set_effect(s, 0, 9, 1);
    }
    // Battle Shout (support)
    if let Some(s) = spells.get_mut(&6673) {
        s.name = "Battle Shout".to_string();
        s.rank = None;
        s.description = Some("Tve jednotky do 20 yd davaji o 15 % vic poskozeni po 20 s.".to_string());
        s.aura_description = Some("Tve jednotky do 20 yd davaji o 15 % vic poskozeni po 20 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 4;
        s.spell_family_flags |= 1u64 << 38;
        s.mana_cost = 10;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 20000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 18;
        set_radius(s, 9);
    }
    // Demoralizing Shout (support)
    if let Some(s) = spells.get_mut(&1160) {
        s.name = "Demoralizing Shout".to_string();
        s.rank = None;
        s.description = Some("Nepratele do 10 yd davaji o 20 % mene poskozeni po 15 s.".to_string());
        s.aura_description = Some("Nepratele do 10 yd davaji o 20 % mene poskozeni po 15 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 4;
        s.spell_family_flags |= 1u64 << 39;
        s.mana_cost = 10;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 15000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 8;
        set_radius(s, 13);
    }
    // Challenging Shout (support)
    if let Some(s) = spells.get_mut(&1161) {
        s.name = "Challenging Shout".to_string();
        s.rank = None;
        s.description = Some("Nepratele do 10 yd utoci 6 s jen na tebe (chrani jednotky).".to_string());
        s.aura_description = Some("Nepratele do 10 yd utoci 6 s jen na tebe (chrani jednotky).".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 4;
        s.spell_family_flags |= 1u64 << 40;
        s.mana_cost = 8;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 20000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 32;
        set_radius(s, 13);
    }
    // Shield Wall (defense)
    if let Some(s) = spells.get_mut(&871) {
        s.name = "Shield Wall".to_string();
        s.rank = None;
        s.description = Some("Dostavas o 50 % mene poskozeni po 10 s.".to_string());
        s.aura_description = Some("Dostavas o 50 % mene poskozeni po 10 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 4;
        s.spell_family_flags |= 1u64 << 41;
        s.mana_cost = 8;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 30000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 1;
        set_effect(s, 0, -51, 1);
    }
    // Last Stand (defense)
    if let Some(s) = spells.get_mut(&12975) {
        s.name = "Last Stand".to_string();
        s.rank = None;
        s.description = Some("Okamzite ti vrati 30 % maximalniho zivota.".to_string());
        s.aura_description = Some("Okamzite ti vrati 30 % maximalniho zivota.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 4;
        s.spell_family_flags |= 1u64 << 42;
        s.mana_cost = 8;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 45000;
        s.category_recovery_ms = 0;
        s.category = 0;
    }
    // Retaliation (defense)
    if let Some(s) = spells.get_mut(&20230) {
        s.name = "Retaliation".to_string();
        s.rank = None;
        s.description = Some("Po 10 s vracis uder kazdemu, kdo te zasahne nablizko.".to_string());
        s.aura_description = Some("Po 10 s vracis uder kazdemu, kdo te zasahne nablizko.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 4;
        s.spell_family_flags |= 1u64 << 43;
        s.mana_cost = 8;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 30000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 1;
    }
    // Arcane Shot (single)
    if let Some(s) = spells.get_mut(&3044) {
        s.name = "Arcane Shot".to_string();
        s.rank = None;
        s.description = Some("Rychla strela za 12 poskozeni (arkanni).".to_string());
        s.aura_description = Some("Rychla strela za 12 poskozeni (arkanni).".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 9;
        s.spell_family_flags |= 1u64 << 32;
        s.mana_cost = 3;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 1000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 35;
        set_effect(s, 0, 11, 1);
    }
    // Serpent Sting (single)
    if let Some(s) = spells.get_mut(&1978) {
        s.name = "Serpent Sting".to_string();
        s.rank = None;
        s.description = Some("Jed: 4 poskozeni kazde 3 s po 15 s.".to_string());
        s.aura_description = Some("Jed: 4 poskozeni kazde 3 s po 15 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 9;
        s.spell_family_flags |= 1u64 << 33;
        s.mana_cost = 6;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 3000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 35;
        s.duration_index = 8;
        set_effect(s, 0, 3, 1);
    }
    // Aimed Shot (single)
    if let Some(s) = spells.get_mut(&19434) {
        s.name = "Aimed Shot".to_string();
        s.rank = None;
        s.description = Some("Mirena strela za 125 poskozeni.".to_string());
        s.aura_description = Some("Mirena strela za 125 poskozeni.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 9;
        s.spell_family_flags |= 1u64 << 34;
        s.mana_cost = 15;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 8000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 35;
        set_effect(s, 0, 124, 1);
    }
    // Multi-Shot (aoe)
    if let Some(s) = spells.get_mut(&2643) {
        s.name = "Multi-Shot".to_string();
        s.rank = None;
        s.description = Some("Strela zasahne 3 nepratele a da navic 20 poskozeni.".to_string());
        s.aura_description = Some("Strela zasahne 3 nepratele a da navic 20 poskozeni.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 9;
        s.spell_family_flags |= 1u64 << 35;
        s.mana_cost = 10;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 4000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 35;
        set_effect(s, 0, 19, 1);
    }
    // Volley (aoe)
    if let Some(s) = spells.get_mut(&1510) {
        s.name = "Volley".to_string();
        s.rank = None;
        s.description = Some("Dest sipu na zem: 10 poskozeni kazdou sekundu vsem do 8 yd po 4 s. Hrdina muze dal bojovat.".to_string());
        s.aura_description = Some("Dest sipu na zem: 10 poskozeni kazdou sekundu vsem do 8 yd po 4 s. Hrdina muze dal bojovat.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 9;
        s.spell_family_flags |= 1u64 << 36;
        s.mana_cost = 20;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 10000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 35;
        s.duration_index = 35;
        set_radius(s, 14);
        set_effect(s, 0, 9, 1);
        s.attributes_ex = 268435456;
        s.channel_interrupt_flags = 0;
    }
    // Frost Trap (aoe)
    if let Some(s) = spells.get_mut(&13809) {
        s.name = "Frost Trap".to_string();
        s.rank = None;
        s.description = Some("Polozi pod sebe mrazivou past. Kdo jde pres ni, zpomali se o 60 %. Past vydrzi 60 s.".to_string());
        s.aura_description = Some("Polozi pod sebe mrazivou past. Kdo jde pres ni, zpomali se o 60 %. Past vydrzi 60 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 9;
        s.spell_family_flags |= 1u64 << 37;
        s.mana_cost = 10;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 15000;
        s.category_recovery_ms = 0;
        s.category = 0;
    }
    // Hunter's Mark (support)
    if let Some(s) = spells.get_mut(&1130) {
        s.name = "Hunter's Mark".to_string();
        s.rank = None;
        s.description = Some("Oznaceny nepritel dostava o 20 % vic poskozeni od vseho (hrdinove i jednotky) po 20 s.".to_string());
        s.aura_description = Some("Oznaceny nepritel dostava o 20 % vic poskozeni od vseho (hrdinove i jednotky) po 20 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 9;
        s.spell_family_flags |= 1u64 << 38;
        s.mana_cost = 6;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 5000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 35;
        s.duration_index = 18;
    }
    // Trueshot Aura (support)
    if let Some(s) = spells.get_mut(&19506) {
        s.name = "Trueshot Aura".to_string();
        s.rank = None;
        s.description = Some("Tve jednotky do 25 yd davaji o 10 % vic poskozeni po 30 s.".to_string());
        s.aura_description = Some("Tve jednotky do 25 yd davaji o 10 % vic poskozeni po 30 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 9;
        s.spell_family_flags |= 1u64 << 39;
        s.mana_cost = 10;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 30000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 9;
        set_radius(s, 20);
    }
    // Mend Pet (support)
    if let Some(s) = spells.get_mut(&136) {
        s.name = "Mend Pet".to_string();
        s.rank = None;
        s.description = Some("Vyleci tve jednotky do 20 yd o 25 % jejich zivota.".to_string());
        s.aura_description = Some("Vyleci tve jednotky do 20 yd o 25 % jejich zivota.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 9;
        s.spell_family_flags |= 1u64 << 40;
        s.mana_cost = 10;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 15000;
        s.category_recovery_ms = 0;
        s.category = 0;
        set_radius(s, 9);
        s.attributes_ex = 0;
        s.channel_interrupt_flags = 0;
        s.effect_implicit_target_a[0] = 1;
        s.effect_apply_aura[0] = 4;
    }
    // Feign Death (defense)
    if let Some(s) = spells.get_mut(&5384) {
        s.name = "Feign Death".to_string();
        s.rank = None;
        s.description = Some("Predstiras smrt: nepratele te prestanou napadat.".to_string());
        s.aura_description = Some("Predstiras smrt: nepratele te prestanou napadat.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 9;
        s.spell_family_flags |= 1u64 << 41;
        s.mana_cost = 5;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 30000;
        s.category_recovery_ms = 0;
        s.category = 0;
    }
    // Aspect of the Monkey (defense)
    if let Some(s) = spells.get_mut(&13163) {
        s.name = "Aspect of the Monkey".to_string();
        s.rank = None;
        s.description = Some("Uhybas o 20 % casteji po 20 s.".to_string());
        s.aura_description = Some("Uhybas o 20 % casteji po 20 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 9;
        s.spell_family_flags |= 1u64 << 42;
        s.mana_cost = 6;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 20000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 18;
        set_effect(s, 0, 19, 1);
    }
    // Wing Clip (defense)
    if let Some(s) = spells.get_mut(&2974) {
        s.name = "Wing Clip".to_string();
        s.rank = None;
        s.description = Some("Uder nablizko za 3 poskozeni, zpomali nepritele o 50 % na 5 s.".to_string());
        s.aura_description = Some("Uder nablizko za 3 poskozeni, zpomali nepritele o 50 % na 5 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 9;
        s.spell_family_flags |= 1u64 << 43;
        s.mana_cost = 6;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 6000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 28;
        set_effect(s, 0, 2, 1);
    }
    // Fireball (single)
    if let Some(s) = spells.get_mut(&133) {
        s.name = "Fireball".to_string();
        s.rank = None;
        s.description = Some("Ohniva koule za 10 az 14 poskozeni.".to_string());
        s.aura_description = Some("Ohniva koule za 10 az 14 poskozeni.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 3;
        s.spell_family_flags |= 1u64 << 32;
        s.mana_cost = 3;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 1000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 35;
        set_effect(s, 0, 9, 5);
    }
    // Frostbolt (single)
    if let Some(s) = spells.get_mut(&116) {
        s.name = "Frostbolt".to_string();
        s.rank = None;
        s.description = Some("Ledova strela za 8 az 11 poskozeni, zpomali nepritele o 40 % na 4 s.".to_string());
        s.aura_description = Some("Ledova strela za 8 az 11 poskozeni, zpomali nepritele o 40 % na 4 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 3;
        s.spell_family_flags |= 1u64 << 33;
        s.mana_cost = 3;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 1000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 35;
        set_effect(s, 1, 7, 4);
        s.effect_apply_aura[0] = 4;
    }
    // Pyroblast (single)
    if let Some(s) = spells.get_mut(&11366) {
        s.name = "Pyroblast".to_string();
        s.rank = None;
        s.description = Some("Obri ohniva koule za 70 az 85 poskozeni.".to_string());
        s.aura_description = Some("Obri ohniva koule za 70 az 85 poskozeni.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 3;
        s.spell_family_flags |= 1u64 << 34;
        s.mana_cost = 15;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 8000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 35;
        set_effect(s, 0, 69, 16);
    }
    // Arcane Explosion (aoe)
    if let Some(s) = spells.get_mut(&1449) {
        s.name = "Arcane Explosion".to_string();
        s.rank = None;
        s.description = Some("Vybuch arkanni sily: 15 az 18 poskozeni vsem do 10 yd.".to_string());
        s.aura_description = Some("Vybuch arkanni sily: 15 az 18 poskozeni vsem do 10 yd.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 3;
        s.spell_family_flags |= 1u64 << 35;
        s.mana_cost = 10;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 3000;
        s.category_recovery_ms = 0;
        s.category = 0;
        set_radius(s, 13);
        set_effect(s, 0, 14, 4);
    }
    // Blizzard (aoe)
    if let Some(s) = spells.get_mut(&10) {
        s.name = "Blizzard".to_string();
        s.rank = None;
        s.description = Some("Ledova boure na zem: 15 poskozeni kazdou sekundu vsem do 8 yd po 6 s. Hrdina muze dal bojovat.".to_string());
        s.aura_description = Some("Ledova boure na zem: 15 poskozeni kazdou sekundu vsem do 8 yd po 6 s. Hrdina muze dal bojovat.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 3;
        s.spell_family_flags |= 1u64 << 36;
        s.mana_cost = 25;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 6000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 4;
        s.duration_index = 32;
        set_radius(s, 14);
        set_effect(s, 0, 14, 1);
        s.attributes_ex = 268435592;
        s.channel_interrupt_flags = 0;
    }
    // Flamestrike (aoe)
    if let Some(s) = spells.get_mut(&2120) {
        s.name = "Flamestrike".to_string();
        s.rank = None;
        s.description = Some("Ohen na zem: 18 az 23 poskozeni vsem do 8 yd a pak 3 za kazde 2 s po 8 s.".to_string());
        s.aura_description = Some("Ohen na zem: 18 az 23 poskozeni vsem do 8 yd a pak 3 za kazde 2 s po 8 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 3;
        s.spell_family_flags |= 1u64 << 37;
        s.mana_cost = 15;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 8000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 4;
        s.duration_index = 31;
        set_radius(s, 14);
        set_effect(s, 0, 17, 6);
        set_effect(s, 1, 2, 1);
    }
    // Arcane Brilliance (support)
    if let Some(s) = spells.get_mut(&23028) {
        s.name = "Arcane Brilliance".to_string();
        s.rank = None;
        s.description = Some("Tve jednotky do 20 yd davaji o 15 % vic poskozeni po 20 s.".to_string());
        s.aura_description = Some("Tve jednotky do 20 yd davaji o 15 % vic poskozeni po 20 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 3;
        s.spell_family_flags |= 1u64 << 38;
        s.mana_cost = 10;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 20000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 18;
        set_radius(s, 9);
    }
    // Dampen Magic (support)
    if let Some(s) = spells.get_mut(&604) {
        s.name = "Dampen Magic".to_string();
        s.rank = None;
        s.description = Some("Tve jednotky do 20 yd dostavaji o 20 % mene poskozeni po 20 s.".to_string());
        s.aura_description = Some("Tve jednotky do 20 yd dostavaji o 20 % mene poskozeni po 20 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 3;
        s.spell_family_flags |= 1u64 << 39;
        s.mana_cost = 10;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 20000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 18;
        set_radius(s, 9);
        s.effect_implicit_target_a[0] = 1;
        s.effect_implicit_target_a[1] = 1;
    }
    // Evocation (support)
    if let Some(s) = spells.get_mut(&12051) {
        s.name = "Evocation".to_string();
        s.rank = None;
        s.description = Some("Obnovi 15 % many kazde 2 s po 8 s. Okamzite, hrdina muze dal bojovat.".to_string());
        s.aura_description = Some("Obnovi 15 % many kazde 2 s po 8 s. Okamzite, hrdina muze dal bojovat.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 3;
        s.spell_family_flags |= 1u64 << 40;
        s.mana_cost = 0;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 60000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 31;
        s.attributes_ex = 0;
        s.channel_interrupt_flags = 0;
    }
    // Ice Barrier (defense)
    if let Some(s) = spells.get_mut(&11426) {
        s.name = "Ice Barrier".to_string();
        s.rank = None;
        s.description = Some("Ledovy stit pohlti 200 poskozeni. Vydrzi 30 s.".to_string());
        s.aura_description = Some("Ledovy stit pohlti 200 poskozeni. Vydrzi 30 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 3;
        s.spell_family_flags |= 1u64 << 41;
        s.mana_cost = 10;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 20000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 9;
        set_effect(s, 0, 199, 1);
    }
    // Mana Shield (defense)
    if let Some(s) = spells.get_mut(&1463) {
        s.name = "Mana Shield".to_string();
        s.rank = None;
        s.description = Some("Stit z many pohlti 150 poskozeni. Vydrzi 30 s.".to_string());
        s.aura_description = Some("Stit z many pohlti 150 poskozeni. Vydrzi 30 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 3;
        s.spell_family_flags |= 1u64 << 42;
        s.mana_cost = 8;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 10000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 9;
        set_effect(s, 0, 149, 1);
    }
    // Frost Nova (defense)
    if let Some(s) = spells.get_mut(&122) {
        s.name = "Frost Nova".to_string();
        s.rank = None;
        s.description = Some("Zmrazi nepratele do 10 yd na miste na 4 s a da jim 8 az 10 poskozeni.".to_string());
        s.aura_description = Some("Zmrazi nepratele do 10 yd na miste na 4 s a da jim 8 az 10 poskozeni.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 3;
        s.spell_family_flags |= 1u64 << 43;
        s.mana_cost = 10;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 15000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 35;
        set_radius(s, 13);
        set_effect(s, 0, 7, 3);
    }
    // Corruption (single)
    if let Some(s) = spells.get_mut(&172) {
        s.name = "Corruption".to_string();
        s.rank = None;
        s.description = Some("Okamzite: 4 poskozeni kazde 3 s po 18 s (24).".to_string());
        s.aura_description = Some("Okamzite: 4 poskozeni kazde 3 s po 18 s (24).".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 5;
        s.spell_family_flags |= 1u64 << 32;
        s.mana_cost = 5;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 3000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 4;
        s.duration_index = 85;
        set_effect(s, 0, 3, 1);
    }
    // Shadow Bolt (single)
    if let Some(s) = spells.get_mut(&686) {
        s.name = "Shadow Bolt".to_string();
        s.rank = None;
        s.description = Some("Stinova strela za 15 az 19 poskozeni.".to_string());
        s.aura_description = Some("Stinova strela za 15 az 19 poskozeni.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 5;
        s.spell_family_flags |= 1u64 << 33;
        s.mana_cost = 3;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 1000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 4;
        set_effect(s, 0, 14, 5);
    }
    // Immolate (single)
    if let Some(s) = spells.get_mut(&348) {
        s.name = "Immolate".to_string();
        s.rank = None;
        s.description = Some("Ohen: 8 az 10 poskozeni a pak 3 kazde 3 s po 15 s.".to_string());
        s.aura_description = Some("Ohen: 8 az 10 poskozeni a pak 3 kazde 3 s po 15 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 5;
        s.spell_family_flags |= 1u64 << 34;
        s.mana_cost = 6;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 3000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 4;
        s.duration_index = 8;
        set_effect(s, 0, 2, 1);
        set_effect(s, 1, 7, 3);
    }
    // Rain of Fire (aoe)
    if let Some(s) = spells.get_mut(&5740) {
        s.name = "Rain of Fire".to_string();
        s.rank = None;
        s.description = Some("Ohnivy dest na zem: 13 poskozeni kazde 2 s vsem do 8 yd po 6 s. Hrdina muze dal bojovat.".to_string());
        s.aura_description = Some("Ohnivy dest na zem: 13 poskozeni kazde 2 s vsem do 8 yd po 6 s. Hrdina muze dal bojovat.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 5;
        s.spell_family_flags |= 1u64 << 35;
        s.mana_cost = 20;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 6000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 4;
        s.duration_index = 32;
        set_radius(s, 14);
        set_effect(s, 0, 12, 1);
        s.attributes_ex = 268435592;
        s.channel_interrupt_flags = 0;
    }
    // Epidemic (aoe)
    if let Some(s) = spells.get_mut(&6789) {
        s.name = "Epidemic".to_string();
        s.rank = None;
        s.description = Some("15 az 20 poskozeni a tve DoTy a kletby z cile preskoci na az 6 nepratel do 10 yd.".to_string());
        s.aura_description = Some("15 az 20 poskozeni a tve DoTy a kletby z cile preskoci na az 6 nepratel do 10 yd.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 5;
        s.spell_family_flags |= 1u64 << 36;
        s.mana_cost = 15;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 10000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 4;
        set_effect(s, 0, 14, 6);
        s.effects[0] = 2;
        s.effect_apply_aura[0] = 0;
        s.effects[1] = 0;
        s.effect_apply_aura[1] = 0;
        s.effect_implicit_target_a[1] = 0;
        s.mechanic = 0;
        s.effect_mechanic[0] = 0;
        s.effect_mechanic[1] = 0;
    }
    // Curse of Weakness (support)
    if let Some(s) = spells.get_mut(&702) {
        s.name = "Curse of Weakness".to_string();
        s.rank = None;
        s.description = Some("Kletba: nepritel dava o 25 % mene poskozeni a u hradu ubere o 50 % mene. Trva 20 s.".to_string());
        s.aura_description = Some("Kletba: nepritel dava o 25 % mene poskozeni a u hradu ubere o 50 % mene. Trva 20 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 5;
        s.spell_family_flags |= 1u64 << 37;
        s.mana_cost = 6;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 5000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 4;
        s.duration_index = 18;
    }
    // Curse of the Elements (support)
    if let Some(s) = spells.get_mut(&1490) {
        s.name = "Curse of the Elements".to_string();
        s.rank = None;
        s.description = Some("Kletba: nepritel dostava o 15 % vic poskozeni od vseho po 20 s.".to_string());
        s.aura_description = Some("Kletba: nepritel dostava o 15 % vic poskozeni od vseho po 20 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 5;
        s.spell_family_flags |= 1u64 << 38;
        s.mana_cost = 6;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 5000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 4;
        s.duration_index = 18;
        s.effect_apply_aura[0] = 4;
        s.effect_apply_aura[1] = 4;
    }
    // Blood Pact (support)
    if let Some(s) = spells.get_mut(&6307) {
        s.name = "Blood Pact".to_string();
        s.rank = None;
        s.description = Some("Tve jednotky do 20 yd dostavaji o 20 % mene poskozeni po 20 s.".to_string());
        s.aura_description = Some("Tve jednotky do 20 yd dostavaji o 20 % mene poskozeni po 20 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 5;
        s.spell_family_flags |= 1u64 << 39;
        s.mana_cost = 10;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 20000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 18;
        set_radius(s, 9);
    }
    // Demon Skin (defense)
    if let Some(s) = spells.get_mut(&687) {
        s.name = "Demon Skin".to_string();
        s.rank = None;
        s.description = Some("Brneni +40 a obnova 3 zivota kazde 5 s po 30 s.".to_string());
        s.aura_description = Some("Brneni +40 a obnova 3 zivota kazde 5 s po 30 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 5;
        s.spell_family_flags |= 1u64 << 40;
        s.mana_cost = 8;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 30000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 9;
        set_effect(s, 0, 39, 1);
        set_effect(s, 1, 2, 1);
    }
    // Shadow Ward (defense)
    if let Some(s) = spells.get_mut(&6229) {
        s.name = "Shadow Ward".to_string();
        s.rank = None;
        s.description = Some("Stit pohlti 200 poskozeni. Vydrzi 30 s.".to_string());
        s.aura_description = Some("Stit pohlti 200 poskozeni. Vydrzi 30 s.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 5;
        s.spell_family_flags |= 1u64 << 41;
        s.mana_cost = 10;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 20000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.duration_index = 9;
        set_effect(s, 0, 199, 1);
    }
    // Drain Life (defense)
    if let Some(s) = spells.get_mut(&689) {
        s.name = "Drain Life".to_string();
        s.rank = None;
        s.description = Some("Vysava 5 zivota kazdou sekundu po 5 s a leci te o ne. Hrdina muze dal bojovat.".to_string());
        s.aura_description = Some("Vysava 5 zivota kazdou sekundu po 5 s a leci te o ne. Hrdina muze dal bojovat.".to_string());
        s.power_type = 0;
        s.stances = 0;
        s.stances_not = 0;
        s.base_level = 1;
        s.spell_level = 1;
        s.max_level = 0;
        s.effect_dice_per_level = [0; 3];
        s.reagents = [(0, 0); 8];
        s.totems = [0; 2];
        s.equipped_item_class = -1;
        s.equipped_item_subclass_mask = 0;
        s.equipped_item_inventory_type_mask = 0;
        s.spell_family = 5;
        s.spell_family_flags |= 1u64 << 42;
        s.mana_cost = 6;
        s.casting_time_index = 1;
        s.start_recovery_category = 0;
        s.start_recovery_ms = 0;
        s.recovery_ms = 5000;
        s.category_recovery_ms = 0;
        s.category = 0;
        s.range_index = 4;
        s.duration_index = 28;
        set_effect(s, 0, 4, 1);
        s.attributes_ex = 16384;
        s.channel_interrupt_flags = 0;
    }
    if let Some(s) = spells.get_mut(&13810) {
        s.effect_apply_aura[0] = 4;
    }
}
