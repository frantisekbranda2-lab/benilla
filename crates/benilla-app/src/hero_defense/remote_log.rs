//! Hero Defense: centralni log. Klient posila serveru, co se mu stalo, a server to zapise do
//! jednoho souboru `server\HeroDefense.log` s hracem a casem (`HdLog` v `core/src/scripts/custom`,
//! prikaz addonu `LOG;druh;text`). Hostitel pak pri testu po LAN vidi chyby a pady vsech hracu
//! na jednom miste.
//!
//! Co posila (jako zprava addonu `HD`, kanal PARTY, server ji zachyti i bez skupiny):
//! - `I` po vstupu do hry: verze klienta.
//! - `C` crash report z minuleho behu (`benilla-config/Diagnostics/crash-*.txt`, [`crate::crash`]):
//!   soubor, misto a zprava panicu. Odeslane soubory si pamatuje v `Diagnostics/odeslano.txt`.
//! - `E` radky logu klienta s urovni ERROR (z [`benilla_world::log_ring`]), nejvys
//!   [`MAX_PER_MINUTE`] za minutu, at chyba v kazdem snimku server nezahlti.
//!
//! Chyby Lua (addon, UI) posila addon sam (`seterrorhandler`, `HeroDefense.lua`).
//! Posila se jen ve hre na serveru Hero Defense: kdyz v poslednich 30 s prisel radek `HB;`
//! (bonus hrdiny, `HdGame::SendHeroBonus`, jen ve hre aspon kazdych 10 s). V menu nic.

use std::collections::{HashSet, VecDeque};

use bevy::prelude::*;

use crate::net::{ClientCommand, NetCommands};

/// Nejvys tolik radku `E` za minutu.
const MAX_PER_MINUTE: u32 = 10;
/// Delka textu jedne zpravy (zprava addonu ma strop 255 znaku i s predponou).
const MAX_TEXT: usize = 220;
/// Data ze serveru starsi nez tohle: hrac uz neni ve hre.
const ONLINE_SECS: f64 = 30.0;

#[derive(Resource, Default)]
pub(crate) struct RemoteLog {
    /// Cas posledniho radku `HB;` ze serveru ([`super::on_hd_data`]).
    pub(crate) last_data: Option<f64>,
    hello_sent: bool,
    crashes_sent: bool,
    /// Uz odeslane radky logu (radky v kruhu jsou diky casu jedinecne).
    sent: HashSet<String>,
    sent_order: VecDeque<String>,
    next_scan: f64,
    minute_start: f64,
    minute_count: u32,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<RemoteLog>().add_systems(Update, pump);
}

fn clean(text: &str) -> String {
    let mut t: String = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    if t.chars().count() > MAX_TEXT {
        t = t.chars().take(MAX_TEXT).collect();
    }
    t
}

fn send(net: &NetCommands, kind: char, text: &str) {
    let _ = net.0.send(ClientCommand::AddonMessage {
        distribution: benilla_ui::script::AddonDistribution::Party,
        text: format!("HD\tLOG;{kind};{}", clean(text)),
    });
}

/// Crash reporty, ktere jeste nesly na server: (soubor, misto, zprava).
fn unsent_crashes() -> Vec<(String, String, String)> {
    let Some(dir) = crate::local_state::diagnostics_dir() else {
        return Vec::new();
    };
    let done_path = dir.join("odeslano.txt");
    let done: HashSet<String> = std::fs::read_to_string(&done_path)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_string())
        .collect();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if !name.starts_with("crash-") || !name.ends_with(".txt") || done.contains(&name) {
            continue;
        }
        let text = std::fs::read_to_string(e.path()).unwrap_or_default();
        let field = |key: &str| {
            text.lines()
                .find_map(|l| l.strip_prefix(key))
                .map(|v| v.trim().to_string())
                .unwrap_or_default()
        };
        out.push((name, field("at:"), field("panic:")));
    }
    out.sort();
    out
}

fn mark_sent(names: &[String]) {
    let Some(dir) = crate::local_state::diagnostics_dir() else {
        return;
    };
    let path = dir.join("odeslano.txt");
    let mut text = std::fs::read_to_string(&path).unwrap_or_default();
    for n in names {
        text.push_str(n);
        text.push('\n');
    }
    let _ = std::fs::write(path, text);
}

fn pump(
    mut log: ResMut<RemoteLog>,
    net: Option<Res<NetCommands>>,
    build: Option<Res<crate::BuildId>>,
    time: Res<Time>,
) {
    let now = time.elapsed_secs_f64();
    if now < log.next_scan {
        return;
    }
    log.next_scan = now + 5.0;
    let Some(net) = net else {
        return;
    };
    let online = log.last_data.is_some_and(|t| now - t < ONLINE_SECS);
    if !online {
        return;
    }

    if !log.hello_sent {
        log.hello_sent = true;
        let version = build.map_or_else(|| "?".to_string(), |b| b.summary());
        send(&net, 'I', &format!("klient {version}"));
    }

    if !log.crashes_sent {
        log.crashes_sent = true;
        let crashes = unsent_crashes();
        for (name, at, panic) in &crashes {
            send(&net, 'C', &format!("{name}: {panic} (at {at})"));
        }
        mark_sent(&crashes.into_iter().map(|c| c.0).collect::<Vec<_>>());
    }

    if now - log.minute_start >= 60.0 {
        log.minute_start = now;
        log.minute_count = 0;
    }
    for line in benilla_world::log_ring::recent() {
        if !line.contains(" ERROR ") || log.sent.contains(&line) {
            continue;
        }
        log.sent.insert(line.clone());
        log.sent_order.push_back(line.clone());
        if log.sent_order.len() > 2 * benilla_world::log_ring::CAPACITY {
            if let Some(old) = log.sent_order.pop_front() {
                log.sent.remove(&old);
            }
        }
        if log.minute_count >= MAX_PER_MINUTE {
            continue;
        }
        log.minute_count += 1;
        // Bez casu od startu ("+   12.345s ERROR "): server pise vlastni cas.
        let text = line.split_once(" ERROR ").map_or(line.as_str(), |(_, rest)| rest);
        send(&net, 'E', text);
    }
}
