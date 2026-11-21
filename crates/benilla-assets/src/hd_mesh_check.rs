//! Hero Defense: overeni terenu na siti, jakou klient kresli a po jake postava chodi.
//!
//! Vyskove pole `hd_terrain` se vzorkuje ve vrcholech terenu (rohy bunek po 4,17 yd a jejich
//! stredy) a mezi nimi je rovny trojuhelnik, takze uzky hreben nebo stena muze na siti vyjit nizsi
//! a mirnejsi nez v poli. Tady se proto pocita primo na trojuhelnicich, co postava zvladne:
//!
//! - vyjde sklon do 50 stupnu,
//! - po strmejsim sjede dolu,
//! - umi vyskocit (1,64 yd vysoko, necelych 6 yd daleko), a to i ze steny do `CLIMB_SLOPE`
//!   stupnu: po mirne strme stene se da vyskakat.
//!
//! Z areny se nesmi dat dostat do jine areny jinudy nez pres rampu a nadvori a na skaly se nesmi
//! dat dostat odnikud.
//!
//! Nahrazuje `core/tools/terrain/check.py`: ten pocital jen chuzi, bez skluzu a bez vyskoku,
//! a sedlo mezi arenami 1 a 2 (sklon 52 az 55 stupnu, 1,4 az 4 yd nad plosinou) mu proslo. Hrac ho
//! 21. 11. 2026 preskakal.

use crate::hd_terrain as t;

/// Rozestup vrcholu terenu (yd) a stred mapy.
const UNIT: f64 = 1600.0 / 3.0 / 128.0;
const MAP_CENTER: f64 = 32.0 * 1600.0 / 3.0;
/// Polomer overovane plochy kolem hradu (skaly konci do 252 yd).
const RADIUS: f64 = 262.0;
/// Vyskok postavy: pocatecni rychlost 7,96 yd/s a tize 19,29 yd/s2 davaji vrchol 1,64 yd a 0,83 s
/// ve vzduchu, pri behu 7 yd/s tedy necelych 6 yd. Pocita se s rezervou.
const JUMP_UP: f64 = 1.9;
const JUMP_REACH: f64 = 6.5;
/// Do jakeho sklonu (stupne) se test tvari, ze se da ze steny odrazit k dalsimu vyskoku. Sedlo, ktere
/// hrac preskakal, melo 52 az 55; steny u pochozich ploch maji mit pres 73.
const CLIMB_SLOPE: f64 = 65.0;

/// Vysky terenu na mrizce vrcholu kolem hradu, posunute o `shift` yardu (jina poloha mrizky vuci
/// hradu, at vysledek nestoji na nahode jednoho zarovnani).
struct Mesh {
    i0: i64,
    j0: i64,
    ni: usize,
    nj: usize,
    shift: f64,
    corners: Vec<f64>,
    centres: Vec<f64>,
}

impl Mesh {
    fn new(shift: f64) -> Mesh {
        let i0 = ((MAP_CENTER - (t::CX + RADIUS)) / UNIT).floor() as i64;
        let i1 = ((MAP_CENTER - (t::CX - RADIUS)) / UNIT).ceil() as i64;
        let j0 = ((MAP_CENTER - (t::CY + RADIUS)) / UNIT).floor() as i64;
        let j1 = ((MAP_CENTER - (t::CY - RADIUS)) / UNIT).ceil() as i64;
        let mut m = Mesh {
            i0,
            j0,
            ni: (i1 - i0 + 1) as usize,
            nj: (j1 - j0 + 1) as usize,
            shift,
            corners: Vec::new(),
            centres: Vec::new(),
        };
        for a in 0..m.ni {
            for b in 0..m.nj {
                m.corners.push(t::height(m.x(a as f64), m.y(b as f64)));
            }
        }
        for a in 0..m.ni - 1 {
            for b in 0..m.nj - 1 {
                m.centres
                    .push(t::height(m.x(a as f64 + 0.5), m.y(b as f64 + 0.5)));
            }
        }
        m
    }

    fn x(&self, a: f64) -> f64 {
        MAP_CENTER - (self.i0 as f64 + a) * UNIT + self.shift
    }

    fn y(&self, b: f64) -> f64 {
        MAP_CENTER - (self.j0 as f64 + b) * UNIT + self.shift
    }

    fn cells(&self) -> usize {
        (self.ni - 1) * (self.nj - 1)
    }

    /// Trojuhelnik `id % 4` bunky `id / 4`: stred bunky a dva jeji rohy.
    fn triangle(&self, id: usize) -> [[f64; 3]; 3] {
        let (cell, k) = (id / 4, id % 4);
        let (a, b) = (cell / (self.nj - 1), cell % (self.nj - 1));
        let corner = |da: usize, db: usize| {
            [
                self.x((a + da) as f64),
                self.y((b + db) as f64),
                self.corners[(a + da) * self.nj + b + db],
            ]
        };
        let centre = [
            self.x(a as f64 + 0.5),
            self.y(b as f64 + 0.5),
            self.centres[cell],
        ];
        let (tl, tr, bl, br) = (corner(0, 0), corner(0, 1), corner(1, 0), corner(1, 1));
        match k {
            0 => [centre, tl, bl],
            1 => [centre, bl, br],
            2 => [centre, br, tr],
            _ => [centre, tr, tl],
        }
    }

    /// Vyska site v bode (souradnice sveta); mimo sit 0.
    fn height_at(&self, x: f64, y: f64) -> f64 {
        let (fa, fb) = (
            (MAP_CENTER + self.shift - x) / UNIT - self.i0 as f64,
            (MAP_CENTER + self.shift - y) / UNIT - self.j0 as f64,
        );
        if fa < 0.0 || fb < 0.0 || fa >= (self.ni - 1) as f64 || fb >= (self.nj - 1) as f64 {
            return 0.0;
        }
        let (a, b) = (fa as usize, fb as usize);
        // Na jih (radky) a na vychod (sloupce) v bunce, stejne vahy jako `hd_zone::chunk_height`.
        let (fy, fx) = (fa - a as f64, fb - b as f64);
        let z = |da: usize, db: usize| self.corners[(a + da) * self.nj + b + db];
        let (tl, tr, bl, br) = (z(0, 0), z(0, 1), z(1, 0), z(1, 1));
        let ctr = self.centres[a * (self.nj - 1) + b];
        let (u, v, w, va, vb) = if fx + fy <= 1.0 {
            if fy <= fx {
                (2.0 * fy, 1.0 - fx - fy, fx - fy, tl, tr)
            } else {
                (2.0 * fx, fy - fx, 1.0 - fx - fy, bl, tl)
            }
        } else if fy >= fx {
            (2.0 * (1.0 - fy), fx + fy - 1.0, fy - fx, br, bl)
        } else {
            (2.0 * (1.0 - fx), fx - fy, fx + fy - 1.0, tr, br)
        };
        u * ctr + v * va + w * vb
    }

    /// Trojuhelniky bunek do `reach` yardu od bunky trojuhelniku `id`.
    fn around(&self, id: usize, reach: f64) -> Vec<usize> {
        let cell = id / 4;
        let (a, b) = ((cell / (self.nj - 1)) as i64, (cell % (self.nj - 1)) as i64);
        let n = (reach / UNIT).ceil() as i64;
        let mut out = Vec::new();
        for da in -n..=n {
            for db in -n..=n {
                let (ca, cb) = (a + da, b + db);
                if ca < 0 || cb < 0 || ca >= self.ni as i64 - 1 || cb >= self.nj as i64 - 1 {
                    continue;
                }
                let c = ca as usize * (self.nj - 1) + cb as usize;
                out.extend(c * 4..c * 4 + 4);
            }
        }
        out
    }

    /// Sousede trojuhelniku pres jeho tri hrany: (soused, vyska stredu spolecne hrany).
    fn neighbours(&self, id: usize) -> Vec<(usize, f64)> {
        let (cell, k) = (id / 4, id % 4);
        let (a, b) = (cell / (self.nj - 1), cell % (self.nj - 1));
        let tri = self.triangle(id);
        let mid = |p: [f64; 3], q: [f64; 3]| (p[2] + q[2]) / 2.0;
        let mut out = vec![
            (cell * 4 + (k + 1) % 4, mid(tri[0], tri[2])),
            (cell * 4 + (k + 3) % 4, mid(tri[0], tri[1])),
        ];
        let across = match k {
            0 if b > 0 => Some((cell - 1) * 4 + 2),
            1 if a + 2 < self.ni => Some((cell + self.nj - 1) * 4 + 3),
            2 if b + 2 < self.nj => Some((cell + 1) * 4),
            3 if a > 0 => Some((cell - (self.nj - 1)) * 4 + 1),
            _ => None,
        };
        if let Some(o) = across {
            out.push((o, mid(tri[1], tri[2])));
        }
        out
    }
}

/// Kam trojuhelnik patri: mrizka areny `k` (1 az 8), `PASS` = nadvori, cesta bosse a rampy (tudy
/// se mezi arenami chodit smi), `PLAIN` = plan za vnejsim okrajem skal, 0 = skaly a okraje plosin.
const PASS: i32 = -1;
const PLAIN: i32 = -2;

fn region(x: f64, y: f64) -> i32 {
    let r = t::distance(x, y);
    if r < t::COURT_R {
        return PASS;
    }
    let (f, l) = t::local(x, y, 0);
    if f > 0.0 && l.abs() <= t::LANE_HALF {
        return PASS;
    }
    for k in 1..t::SLOTS {
        let (f, l) = t::local(x, y, k);
        if (t::FINAL_PATH..=t::ARENA_FAR).contains(&f) && l.abs() <= t::ARENA_HALF {
            return k;
        }
        if f >= t::RAMP_BOT - 4.0 && f < t::FINAL_PATH && l.abs() <= t::RAMP_HALF {
            return PASS;
        }
    }
    if r > t::OUTER_MAX + 2.0 {
        return PLAIN;
    }
    0
}

struct Walk {
    mesh: Mesh,
    /// Sklon trojuhelniku ve stupnich; schudny je do 50.
    slope: Vec<f64>,
    region: Vec<i32>,
    centre: Vec<[f64; 3]>,
}

impl Walk {
    fn new(shift: f64) -> Walk {
        let mesh = Mesh::new(shift);
        let n = mesh.cells() * 4;
        let (mut slope, mut reg, mut centre) = (Vec::new(), Vec::new(), Vec::new());
        for id in 0..n {
            let p = mesh.triangle(id);
            let u = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
            let v = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
            let nrm = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            let len = (nrm[0] * nrm[0] + nrm[1] * nrm[1] + nrm[2] * nrm[2]).sqrt();
            slope.push((nrm[2].abs() / len).acos().to_degrees());
            let c = [
                (p[0][0] + p[1][0] + p[2][0]) / 3.0,
                (p[0][1] + p[1][1] + p[2][1]) / 3.0,
                (p[0][2] + p[1][2] + p[2][2]) / 3.0,
            ];
            reg.push(region(c[0], c[1]));
            centre.push(c);
        }
        Walk {
            mesh,
            slope,
            region: reg,
            centre,
        }
    }

    fn walkable(&self, id: usize) -> bool {
        self.slope[id] <= 50.0
    }

    /// Da se z trojuhelniku `at` doskocit na trojuhelnik `to`: je na dosah, neni vys nez vyskok
    /// a teren mezi nimi vyskok neprevysuje.
    fn can_jump(&self, at: usize, to: usize) -> bool {
        let (p, q) = (self.centre[at], self.centre[to]);
        let dist = (p[0] - q[0]).hypot(p[1] - q[1]);
        if dist > JUMP_REACH || q[2] > p[2] + JUMP_UP {
            return false;
        }
        let steps = (dist / 0.5).ceil().max(1.0) as usize;
        (1..steps).all(|i| {
            let s = i as f64 / steps as f64;
            let (x, y) = (p[0] + (q[0] - p[0]) * s, p[1] + (q[1] - p[1]) * s);
            self.mesh.height_at(x, y) <= p[2] + JUMP_UP
        })
    }

    /// Vsechny trojuhelniky dosazitelne z tech, pro ktere plati `start` (a odkud se na ne prislo).
    /// Po schudnych kamkoli, na strmy jen shora a z nej jen dolu (skluz), vyskokem z trojuhelniku
    /// do `CLIMB_SLOPE`. Pres `PASS` se nechodi, kdyz `through_pass` neplati.
    fn reach(
        &self,
        start: impl Fn(usize) -> bool,
        through_pass: bool,
    ) -> (Vec<bool>, Vec<Option<usize>>) {
        let n = self.slope.len();
        let mut prev: Vec<Option<usize>> = vec![None; n];
        let mut seen = vec![false; n];
        let mut queue = std::collections::VecDeque::new();
        for id in 0..n {
            if start(id) {
                seen[id] = true;
                queue.push_back(id);
            }
        }
        let open = |id: usize| through_pass || self.region[id] != PASS;
        while let Some(at) = queue.pop_front() {
            for (next, edge) in self.mesh.neighbours(at) {
                if seen[next] || !open(next) {
                    continue;
                }
                let leave = self.walkable(at) || edge <= self.centre[at][2] + 1e-9;
                let enter = self.walkable(next) || edge >= self.centre[next][2] - 1e-9;
                if leave && enter {
                    seen[next] = true;
                    prev[next] = Some(at);
                    queue.push_back(next);
                }
            }
            if self.slope[at] <= CLIMB_SLOPE {
                for next in self.mesh.around(at, JUMP_REACH) {
                    if !seen[next] && open(next) && self.can_jump(at, next) {
                        seen[next] = true;
                        prev[next] = Some(at);
                        queue.push_back(next);
                    }
                }
            }
        }
        (seen, prev)
    }

    /// Cesta k trojuhelniku `to` podle `prev`, jen jeji cast mimo areny a pochozi plochy.
    fn describe(&self, prev: &[Option<usize>], to: usize) -> String {
        let mut path = vec![to];
        let mut p = to;
        while let Some(q) = prev[p] {
            path.push(q);
            p = q;
        }
        path.reverse();
        let mut out = String::new();
        for &id in path.iter().filter(|&&id| self.region[id] == 0) {
            let c = self.centre[id];
            out += &format!(
                "\n   ({:8.1}, {:8.1}) {:5.1} yd od hradu, vyska {:5.2}, sklon {:4.1}",
                c[0],
                c[1],
                t::distance(c[0], c[1]),
                c[2],
                self.slope[id]
            );
        }
        out
    }
}

/// Polohy mrizky vuci hradu: skutecna a dve posunute.
const SHIFTS: [f64; 3] = [0.0, 1.4, 2.9];

#[test]
fn arenas_are_separated_on_the_terrain_mesh() {
    for shift in SHIFTS {
        let walk = Walk::new(shift);
        for from in 1..t::SLOTS {
            let (seen, prev) = walk.reach(|id| walk.region[id] == from, false);
            let hit = (0..seen.len())
                .find(|&id| seen[id] && walk.region[id] > 0 && walk.region[id] != from);
            if let Some(hit) = hit {
                panic!(
                    "posun {shift}: z areny {from} se da dostat do areny {}:{}",
                    walk.region[hit],
                    walk.describe(&prev, hit)
                );
            }
        }
    }
}

#[test]
fn rock_tops_are_out_of_reach() {
    // Odkudkoli z pochozich ploch a z plane za skalami: nikdo se nedostane vys, nez kam z plosiny
    // doskoci (stena pod jeji hranou se pocita, vrchol skaly ne).
    let limit = t::PLATEAU_H + JUMP_UP + 0.1;
    for shift in SHIFTS {
        let walk = Walk::new(shift);
        // Vychozi zem: trojuhelniky cele na pochozi plose (nadvori ma nepravidelny okraj, proto
        // podle vrcholu, ne podle `region`) a plan za skalami.
        let ground = |id: usize| {
            walk.region[id] == PLAIN
                || walk
                    .mesh
                    .triangle(id)
                    .iter()
                    .all(|p| t::on_flat_ground(p[0], p[1]))
        };
        let (seen, prev) = walk.reach(ground, true);
        let top = (0..seen.len())
            .filter(|&id| seen[id])
            .max_by(|&a, &b| walk.centre[a][2].total_cmp(&walk.centre[b][2]))
            .unwrap();
        assert!(
            walk.centre[top][2] <= limit,
            "posun {shift}: da se dostat do vysky {:.2} (nejvys {limit:.2}):{}",
            walk.centre[top][2],
            walk.describe(&prev, top)
        );
    }
}

#[test]
fn walls_beside_walkable_ground_are_steep() {
    // Trojuhelnik, ktery ma vrchol na pochozi plose a vrchol na skale, je stena: aspon 70 stupnu.
    let walk = Walk::new(0.0);
    let mut walls = 0;
    for id in 0..walk.slope.len() {
        let tri = walk.mesh.triangle(id);
        let on_ground = |p: &[f64; 3]| t::on_flat_ground(p[0], p[1]);
        let ground = tri.iter().filter(|p| on_ground(p)).count();
        // Jen uvnitr skal: za vnejsim okrajem je plan, ne pochozi plocha z `walk_height`.
        if ground == 0 || ground == 3 || tri.iter().any(|p| t::distance(p[0], p[1]) > 222.0) {
            continue;
        }
        walls += 1;
        assert!(
            walk.slope[id] >= 70.0,
            "stena u ({:.1}, {:.1}) ma sklon jen {:.1}",
            walk.centre[id][0],
            walk.centre[id][1],
            walk.slope[id]
        );
    }
    assert!(walls > 1000, "sten jen {walls}");
}

#[test]
fn arena_grids_are_flat_on_the_terrain_mesh() {
    // Cela plocha mrizky vcetne krajnich car, po pul yardu: sit je vsude presne ve vysce plosiny.
    // Kontrola jen podle stredu trojuhelniku nestaci: trojuhelnik se stredem tesne za mrizkou do ni
    // zasahuje, a kdyz ma vrchol na skale, zveda krajni pole (stalo se 21. 11. 2026).
    for shift in SHIFTS {
        let mesh = Mesh::new(shift);
        for k in 1..t::SLOTS {
            let o = t::CO + f64::from(k) * t::step();
            for i in 0..=200 {
                for j in -60..=60 {
                    let (f, l) = (t::FINAL_PATH + f64::from(i) * 0.5, f64::from(j) * 0.5);
                    let (x, y) = (
                        t::CX + f * o.cos() - l * o.sin(),
                        t::CY + f * o.sin() + l * o.cos(),
                    );
                    let h = mesh.height_at(x, y);
                    // U usti rampy navazuje prvni rada poli na rampu, ktera ke hrane plosiny
                    // dobiha plynule: tam smi byt sit o par setin yardu niz. Jinde presne.
                    let ramp_mouth = f < t::FINAL_PATH + UNIT && l.abs() < t::RAMP_HALF + UNIT;
                    let slack = if ramp_mouth { 0.02 } else { 1e-9 };
                    assert!(
                        (h - t::PLATEAU_H).abs() < slack,
                        "posun {shift}: mrizka areny {k} neni rovna ({f}, {l}): vyska {h}"
                    );
                }
            }
        }
    }
}
