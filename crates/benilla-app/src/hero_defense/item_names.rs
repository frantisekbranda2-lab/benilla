//! Hero Defense: nazvy modifikatoru itemu v popisku (enchanty instance itemu ze serveru,
//! `core` `hd_items.cpp`). VYGENEROVANO: `node tools/items/gen_items.js` v `core`, needitovat rucne.
//!
//! Id enchantu: [`HD_ENCHANT_AFFIX`] + typ x 1000 + hodnota, uroven itemu [`HD_ENCHANT_TIER`] + tier.
//! Klient je nema v `SpellItemEnchantment.dbc`, proto je ukazatel popisku dostane odsud.

/// Zacatek id enchantu modifikatoru.
pub const HD_ENCHANT_AFFIX: u32 = 40000;
/// Zacatek id enchantu urovne itemu.
pub const HD_ENCHANT_TIER: u32 = 39000;

/// (typ, text s `%d` za hodnotu).
pub const HD_AFFIX_TEXTS: [(u32, &str); 14] = [
    (1, "+%d Stesti"),
    (2, "+%d Zivoty"),
    (3, "+%d Brneni"),
    (4, "+%d Zivoty za 5 s"),
    (5, "+%d % Uhyb"),
    (6, "+%d % Blok"),
    (7, "+%d Sila bloku"),
    (8, "+%d Sila kouzel"),
    (9, "+%d Sila utoku"),
    (10, "+%d % Kriticka sance"),
    (11, "+%d % Kriticke poskozeni"),
    (12, "-%d % Cooldown schopnosti"),
    (13, "+%d Mana"),
    (14, "+%d Maximalni rage"),
];

/// Nazvy vsech enchantu Hero Defense (hodnoty 1 az 999, urovne 1 az 3) pro tabulku nazvu enchantu.
pub fn hd_enchant_names() -> impl Iterator<Item = (u32, String)> {
    let tiers = (1..=3).map(|t| (HD_ENCHANT_TIER + t, format!("Uroven predmetu {t}")));
    let affixes = HD_AFFIX_TEXTS.iter().flat_map(|&(id, text)| {
        (1..1000).map(move |v| {
            (
                HD_ENCHANT_AFFIX + id * 1000 + v,
                text.replace("%d", &v.to_string()),
            )
        })
    });
    tiers.chain(affixes)
}
