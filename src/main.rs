//! Ski!  Mark Stevans' game (1983-1990), from Eric S. Raymond's Python port
//! (the `ski` script beside this crate), ported to Rust for the Asciiverse
//! catalog. Same rules, constants and messages; what changed is listed in
//! PATCHES.md.

use std::io::{self, BufRead, Write};

const VERSION: &str = "6.16";

const REP_SNOW: char = '.';
const REP_TREE: char = 'Y';
const REP_GROUND: char = ':';
const REP_ICE: char = '#';
const REP_PLAYER: char = 'I';
const REP_YETI: char = 'A';
const REP_ICBM: char = '*';
const REP_DEMON: char = 'D';

const HELP: &str = "\
This is ski %s, designed by Mark Stevans, Python port by Eric S. Raymond.
You are hurtling down a ski slope in reverse, trying to evade the Yeti.
Available commands are:

 l = turn left            r = turn right
 j = jump                 h = hop
 t = teleport             Enter = keep skiing
 i = launch ICBM          d = summon Fire Demon

 ! = interpret line as shell command and execute.
 ? = print this help message.

";

// Length of line
const LINE_LEN: i32 = 70;

// Minimum distance from the player at which a new Yeti may appear.
const MIN_YETI_APPEARANCE_DISTANCE: i32 = 3;

// Constant multiplied into the first element of the cellular growth
// automaton probability array with each passing level.
const LEVEL_MULTIPLIER: f64 = 1.01;

// Absolute value of maximum horizontal player speed.
const MAX_HORIZONTAL_PLAYER_SPEED: i32 = 5;

const ICBM_SPEED: i32 = 3; // Horizontal speed of an ICBM.
const ICBM_RANGE: i32 = 2; // Horizontal Yeti lethality range of the ICBM.
const DEMON_RANGE: i32 = 1; // Horizontal Yeti lethality range of the demon.
const DEMON_SPEED: i32 = 1; // Horizontal maximum speed of the Fire Demon.

// Per-turn probabilities
const PROB_YETI_MELT: f64 = 1.0; // Of Yeti spontaneously melting.
const PROB_SKIS_MELT_YETI: f64 = 20.0; // Of melting Yeti by jumping over it.
const PROB_BAD_SPELL: f64 = 10.0; // Of "Summon Fire Demon" spell going awry.
const PROB_BAD_TELEPORT: f64 = 10.0; // Of a teleport going wrong.
const PROB_BAD_ICBM: f64 = 30.0; // Of your ICBM exploding on launch.
const PROB_SLIP_ON_ICE: f64 = 2.0; // Of slipping when on ice.
const PROB_FALL_ON_GROUND: f64 = 10.0; // Of falling down on bare ground.
const PROB_HIT_TREE: f64 = 25.0; // Of hitting tree, each turn in trees
const PROB_BAD_LANDING: f64 = 3.0; // Of landing badly from jump or hop.

// Points for one completed jump; a hop counts as half a jump.
const POINTS_PER_JUMP: f64 = 20.0;
// Points for each meter of horizontal or vertical motion each turn.
const POINTS_PER_METER: f64 = 1.0;
// Points for each Yeti that melts, however it melted.
const POINTS_PER_MELTED_YETI: f64 = 100.0;
// Points docked for each degree of injury.
const POINTS_PER_INJURY_DEGREE: f64 = -40.0;

// The injury categories.
const SLIGHT_INJURY: usize = 0;
const MODERATE_INJURY: usize = 3;
const SEVERE_INJURY: usize = 6;

// The randomness of injury degrees.
const INJURY_RANDOMNESS: usize = 6;

const INJURIES: [&str; 12] = [
    "However, you escaped injury!",
    "But you weren't hurt at all!",
    "But you only got a few scratches.",
    "You received some cuts and bruises.",
    "You wind up with a concussion and some contusions.",
    "You now have a broken rib.",
    "Your left arm has been fractured.",
    "You suffered a broken ankle.",
    "You have a broken arm and a broken leg.",
    "You have four broken limbs and a cut!",
    "You broke every bone in your body!",
    "I'm sorry to tell you that you have been killed....",
];

const RESET: &str = "\x1b[0m";

/// The escape sequence for one map symbol: the colours of the script
/// (curses colour numbers sent as setaf), with the player in reverse
/// video, which is what the script's `A_STANDOUT` was meant to give.
fn color_of(c: char) -> Option<&'static str> {
    Some(match c {
        REP_SNOW => "\x1b[0m\x1b[37m",
        REP_TREE => "\x1b[0m\x1b[32m",
        REP_PLAYER => "\x1b[0m\x1b[7m",
        REP_GROUND => "\x1b[0m\x1b[33m",
        REP_ICE => "\x1b[0m\x1b[36m",
        REP_YETI => "\x1b[0m\x1b[34m",
        REP_ICBM => "\x1b[0m\x1b[35m",
        REP_DEMON => "\x1b[0m\x1b[31m",
        _ => return None,
    })
}

/// Colorize special characters in a display list: a colour change only
/// where a part differs from the last character before it.
fn colorize(parts: &[String], color: bool) -> String {
    let mut out = String::new();
    for (i, p) in parts.iter().enumerate() {
        let same = i > 0 && {
            let mut chars = p.chars();
            let one = chars.next();
            chars.next().is_none() && one == parts[i - 1].chars().last()
        };
        if !same && color {
            let single = {
                let mut chars = p.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => color_of(c),
                    _ => None,
                }
            };
            out.push_str(single.unwrap_or(RESET));
        }
        out.push_str(p);
    }
    if color {
        out.push_str(RESET);
    }
    out
}

fn terrain_key(color: bool) -> String {
    let parts: Vec<String> = [
        "Terrain types:   ",
        ".",
        " = snow      ",
        "Y",
        " = tree  ",
        ":",
        " = bare ground     ",
        "#",
        " = ice\nCreatures:       ",
        "I",
        " = player    ",
        "A",
        " = yeti  ",
        "*",
        " = ICBM            ",
        "D",
        " = fire demon",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    colorize(&parts, color)
}

fn percent(rng: &mut fastrand::Rng, x: f64) -> bool {
    rng.f64() * 100.0 < x
}

/// Python's list indexing: -1 is the last element. The script lets the Yeti
/// step to -1, and an ICBM can overshoot the right edge.
fn wrap(i: i32) -> usize {
    i.rem_euclid(LINE_LEN) as usize
}

/// How a run ends: the description of the accident and its severity.
struct Accident {
    msg: &'static str,
    severity: usize,
}

fn accident(msg: &'static str, severity: usize) -> Option<Accident> {
    Some(Accident { msg, severity })
}

/// The game state.
struct World {
    // Cellular growth automaton probabilities, one array per hazard:
    //    [0] appearance of new hazards in clear snow
    //    [1] hazards edge growth
    //    [2] hazards edge stability
    //    [3] appearance of holes in solid hazards (this allows the Yeti
    //        to work his way through forests)
    prob_tree: [f64; 4],
    prob_ice: [f64; 4],
    prob_ground: [f64; 4],
    level_num: i32,
    slope: Vec<char>,
    player_pos: i32,
    yeti: Option<i32>,
    icbm_pos: Option<i32>,
    demon_pos: Option<i32>,
    prob_yeti_appearance: f64,
}

impl World {
    fn new(rng: &mut fastrand::Rng) -> World {
        let mut w = World {
            prob_tree: [0.0, 30.0, 70.0, 90.0],
            prob_ice: [0.0, 30.0, 70.0, 90.0],
            prob_ground: [0.0, 30.0, 70.0, 90.0],
            level_num: 0,
            slope: vec![REP_SNOW; LINE_LEN as usize],
            player_pos: 0,
            yeti: None,
            icbm_pos: None,
            demon_pos: None,
            prob_yeti_appearance: 0.0,
        };
        w.player_pos = w.teleport(rng);
        // Randomize the appearance probabilities.
        w.prob_tree[0] = rng.u32(0..=99) as f64 / 500.0 + 0.05;
        w.prob_ice[0] = rng.u32(0..=99) as f64 / 500.0 + 0.05;
        w.prob_ground[0] = rng.u32(0..=99) as f64 / 500.0 + 0.05;
        w.prob_yeti_appearance = rng.u32(0..=99) as f64 / 25.0 + 1.0;
        w
    }

    /// What kind of terrain are we on?
    fn terrain(&self) -> char {
        self.slope[wrap(self.player_pos)]
    }

    /// Is the specified position near enough to the player?
    fn nearby(&self, pos: Option<i32>, minval: i32) -> bool {
        pos.is_some_and(|p| (p - self.player_pos).abs() <= minval)
    }

    /// A random location.
    fn teleport(&self, rng: &mut fastrand::Rng) -> i32 {
        rng.i32(0..LINE_LEN)
    }

    /// Generates the slope of the next level from the current one, the
    /// probabilities, and the position of the player.
    fn gen_next_slope(&mut self, rng: &mut fastrand::Rng) {
        let current = self.slope.clone();
        let n = current.len();
        for i in 0..n {
            let (mut trees, mut ice, mut ground) = (0, 0, 0);
            match current[i] {
                REP_TREE => trees += 1,
                REP_ICE => ice += 1,
                REP_GROUND => ground += 1,
                _ => {}
            }
            for j in [i.checked_sub(1), (i + 1 < n).then_some(i + 1)]
                .into_iter()
                .flatten()
            {
                match current[j] {
                    REP_TREE => trees += 1,
                    REP_ICE => ice += 1,
                    REP_GROUND => ground += 1,
                    _ => {}
                }
            }
            let here = i as i32 == self.player_pos;
            self.slope[i] = if percent(rng, self.prob_tree[trees]) && (!here || trees > 0) {
                REP_TREE
            } else if percent(rng, self.prob_ice[ice]) && (!here || ice > 0) {
                REP_ICE
            } else if percent(rng, self.prob_ground[ground]) && (!here || ground > 0) {
                REP_GROUND
            } else {
                REP_SNOW
            };
        }
    }

    /// Go to the next level, and move the player.
    fn update_level(&mut self, player: &Player, rng: &mut fastrand::Rng) {
        self.level_num += 1;
        self.player_pos = (self.player_pos + player.player_speed + LINE_LEN) % LINE_LEN;
        self.gen_next_slope(rng);

        // If there is no Yeti, one might be created, not too close to the player.
        if self.yeti.is_none() && percent(rng, self.prob_yeti_appearance) {
            loop {
                self.yeti = Some(self.teleport(rng));
                if !self.nearby(self.yeti, MIN_YETI_APPEARANCE_DISTANCE) {
                    break;
                }
            }
        }

        // Increase the initial appearance probabilities of all obstacles and the Yeti.
        self.prob_tree[0] *= LEVEL_MULTIPLIER;
        self.prob_ice[0] *= LEVEL_MULTIPLIER;
        self.prob_ground[0] *= LEVEL_MULTIPLIER;
        self.prob_yeti_appearance *= LEVEL_MULTIPLIER;
    }

    fn manipulate_objects(&mut self, player: &mut Player, rng: &mut fastrand::Rng) {
        // If there is a Yeti, the player's jet-powered skis may melt him, or
        // he may spontaneously melt. Otherwise move him towards the player.
        // If there is a tree in the way, the Yeti is blocked.
        if self.yeti.is_some()
            && ((self.nearby(self.yeti, 1) && percent(rng, PROB_SKIS_MELT_YETI))
                || percent(rng, PROB_YETI_MELT))
        {
            self.yeti = None;
            player.num_snomen_melted += 1;
        }

        if let Some(y) = self.yeti {
            if y < self.player_pos {
                if self.slope[wrap(y + 1)] != REP_TREE {
                    self.yeti = Some(y + 1);
                }
            } else if self.slope[wrap(y - 1)] != REP_TREE {
                self.yeti = Some(y - 1);
            }
        }

        // If there is an ICBM, move it towards the Yeti, or self-destruct it.
        if let Some(i) = self.icbm_pos {
            self.icbm_pos = self.yeti.map(|y| {
                if i < y {
                    i + ICBM_SPEED
                } else {
                    i - ICBM_SPEED
                }
            });
        }

        // If there is a fire demon, move it towards the Yeti, or it might leave.
        if let Some(d) = self.demon_pos {
            if let Some(y) = self.yeti {
                self.demon_pos = Some(if d < y {
                    d + DEMON_SPEED
                } else {
                    d - DEMON_SPEED
                });
            } else if percent(rng, 25.0) {
                self.demon_pos = None;
            }
        }

        // If there is a Yeti and an ICBM on the slope, the Yeti might get melted.
        if let (Some(y), Some(i)) = (self.yeti, self.icbm_pos) {
            if (y - i).abs() <= ICBM_RANGE {
                self.icbm_pos = None;
                self.yeti = None;
                player.num_snomen_melted += 1;
            }
        }

        // If there is a Yeti and a fire demon, he might get melted.
        if let (Some(y), Some(d)) = (self.yeti, self.demon_pos) {
            if (y - d).abs() <= DEMON_RANGE {
                self.yeti = None;
                player.num_snomen_melted += 1;
            }
        }
    }

    /// A picture of the current level.
    fn picture(&self, color: bool) -> String {
        let mut picture = self.slope.clone();
        picture[wrap(self.player_pos)] = REP_PLAYER;
        for (pos, rep) in [
            (self.yeti, REP_YETI),
            (self.demon_pos, REP_DEMON),
            (self.icbm_pos, REP_ICBM),
        ] {
            if let Some(p) = pos {
                picture[wrap(p)] = rep;
            }
        }
        let parts: Vec<String> = picture.iter().map(|c| c.to_string()).collect();
        format!("{:4} {}", self.level_num, colorize(&parts, color))
    }
}

/// The player in all his glory.
struct Player {
    jump_count: i32,
    num_snomen_melted: i32,
    num_jumps_attempted: f64,
    player_speed: i32,
    meters_travelled: i32,
}

impl Player {
    fn new() -> Player {
        Player {
            jump_count: -1,
            num_snomen_melted: 0,
            num_jumps_attempted: 0.0,
            player_speed: 0,
            meters_travelled: 0,
        }
    }

    /// The end of the run: the accident, the injuries, the statistics and
    /// the score.
    fn report(&self, a: Accident, rng: &mut fastrand::Rng) {
        let degree = a.severity + rng.usize(0..INJURY_RANDOMNESS);
        println!("!\n\n{}  {}\n", a.msg, INJURIES[degree]);
        println!(
            "You skiied {} meters with {} jumps and melted {} {}.",
            self.meters_travelled,
            self.num_jumps_attempted as i64,
            self.num_snomen_melted,
            if self.num_snomen_melted != 1 {
                "Yetis"
            } else {
                "Yeti"
            }
        );
        let score = self.meters_travelled as f64 * POINTS_PER_METER
            + self.num_jumps_attempted * POINTS_PER_JUMP
            + self.num_snomen_melted as f64 * POINTS_PER_MELTED_YETI
            + degree as f64 * POINTS_PER_INJURY_DEGREE;
        // Negative scores are just too silly.
        println!("Your score for this run is {}.", score.max(0.0) as i64);
    }

    fn check_obstacles(&self, world: &World, rng: &mut fastrand::Rng) -> Option<Accident> {
        if self.jump_count == 0 && percent(rng, PROB_BAD_LANDING) {
            return accident("Whoops!  A bad landing!", SLIGHT_INJURY);
        }
        if world.terrain() == REP_TREE && percent(rng, PROB_HIT_TREE) {
            return accident("Oh no!  You hit a tree!", SEVERE_INJURY);
        }
        if world.terrain() == REP_GROUND && percent(rng, PROB_FALL_ON_GROUND) {
            return accident("You fell on the ground!", MODERATE_INJURY);
        }
        if world.terrain() == REP_ICE && percent(rng, PROB_SLIP_ON_ICE) {
            return accident("Oops!  You slipped on the ice!", SLIGHT_INJURY);
        }
        if world.nearby(world.yeti, 1) {
            return accident("Yikes!  The Yeti's got you!", MODERATE_INJURY);
        }
        None
    }

    /// Update state of player for current move.
    fn update_player(&mut self) {
        self.meters_travelled += self.player_speed.abs() + 1;
        if self.jump_count >= 0 {
            self.jump_count -= 1;
        }
    }
}

enum Outcome {
    Advance,
    Stay,
    Crash(Accident),
}

fn do_command(
    player: &mut Player,
    world: &mut World,
    cmdline: &str,
    color: bool,
    rng: &mut fastrand::Rng,
) -> Outcome {
    let cmd = cmdline.chars().next().unwrap_or('\n').to_ascii_uppercase();
    let crash = |a: Option<Accident>| a.map(Outcome::Crash);
    match cmd {
        '?' => {
            print!("{}{}\n", HELP.replace("%s", VERSION), terrain_key(color));
            return Outcome::Stay;
        }
        '!' => {
            shell(&cmdline[1..]);
            return Outcome::Stay;
        }
        'R' => {
            if world.terrain() != REP_ICE && player.player_speed < MAX_HORIZONTAL_PLAYER_SPEED {
                player.player_speed += 1;
            }
        }
        'L' => {
            if world.terrain() != REP_ICE && player.player_speed > -MAX_HORIZONTAL_PLAYER_SPEED {
                player.player_speed -= 1;
            }
        }
        'J' => {
            player.jump_count = rng.i32(0..=5) + 4;
            player.num_jumps_attempted += 1.0;
        }
        'H' => {
            player.jump_count = rng.i32(0..=2) + 2;
            player.num_jumps_attempted += 0.5;
        }
        'T' => {
            if percent(rng, PROB_BAD_TELEPORT) {
                return crash(accident(
                    "You materialized 25 feet in the air!",
                    SLIGHT_INJURY,
                ))
                .unwrap();
            }
            world.player_pos = world.teleport(rng);
        }
        'I' => {
            if percent(rng, PROB_BAD_ICBM) {
                return crash(accident("Nuclear blast in your backpack!", SEVERE_INJURY)).unwrap();
            }
            world.icbm_pos = Some(world.player_pos);
        }
        'D' => {
            if percent(rng, PROB_BAD_SPELL) {
                return crash(accident(
                    "A bad spell -- the demon grabs you!",
                    MODERATE_INJURY,
                ))
                .unwrap();
            }
            world.demon_pos = Some(world.teleport(rng));
        }
        // Any other command just advances
        _ => {}
    }
    Outcome::Advance
}

fn shell(command: &str) {
    let _ = io::stdout().flush();
    #[cfg(windows)]
    let status = std::process::Command::new("cmd")
        .args(["/C", command])
        .status();
    #[cfg(not(windows))]
    let status = std::process::Command::new("sh")
        .args(["-c", command])
        .status();
    if let Err(e) = status {
        println!("{e}");
    }
}

/// Reads one line, without its line ending; None at end of input.
fn read_line() -> Option<String> {
    let _ = io::stdout().flush();
    let mut line = String::new();
    match io::stdin().lock().read_line(&mut line) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(line.trim_end_matches(['\r', '\n']).to_string()),
    }
}

/// A command line: a count in front repeats the command (or Enter alone)
/// that many times, as `5l` or `5`.
fn parse_repeat(line: &str) -> (usize, String) {
    let digits = line.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 {
        return (
            0,
            if line.is_empty() {
                "\n".to_string()
            } else {
                line.to_string()
            },
        );
    }
    let n: usize = line[..digits].parse().unwrap_or(usize::MAX);
    let rest = &line[digits..];
    (
        n,
        if rest.is_empty() {
            "\n".to_string()
        } else {
            rest.to_string()
        },
    )
}

/// One run down the slope. Returns false when the input ended.
fn run(color: bool, rng: &mut fastrand::Rng) -> bool {
    let mut world = World::new(rng);
    let mut player = Player::new();
    let mut out = io::stdout();
    let mut repeat = 1usize;
    let mut cmd = "\n".to_string();
    loop {
        let _ = write!(out, "{}", world.picture(color));
        // If we are jumping, just finish the line. Otherwise, check for
        // obstacles, and do a command.
        if player.jump_count >= 0 {
            let _ = writeln!(out);
        } else {
            if let Some(a) = player.check_obstacles(&world, rng) {
                player.report(a, rng);
                return true;
            }
            repeat = repeat.saturating_sub(1);
            if repeat > 0 {
                let _ = writeln!(out);
            } else {
                let _ = write!(out, "? ");
                let Some(line) = read_line() else {
                    return false;
                };
                let (n, c) = parse_repeat(&line);
                repeat = n;
                cmd = c;
            }
            match do_command(&mut player, &mut world, &cmd, color, rng) {
                Outcome::Advance => {}
                Outcome::Stay => continue,
                Outcome::Crash(a) => {
                    player.report(a, rng);
                    return true;
                }
            }
        }
        world.manipulate_objects(&mut player, rng);
        world.update_level(&player, rng);
        player.update_player();
    }
}

/// Whether colour is on, for `bye`, which the Ctrl+C handler also calls.
static COLOR: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn bye() -> ! {
    if COLOR.load(std::sync::atomic::Ordering::Relaxed) {
        print!("{RESET}");
    }
    print!("\nBye!\n");
    let _ = io::stdout().flush();
    std::process::exit(0);
}

fn usage() -> ! {
    println!(
        "usage: ski [--seed N]\n\nSki! {VERSION}: evade the deadly Yeti on your jet-powered skis.\nType ? in the game for help."
    );
    std::process::exit(0);
}

fn main() {
    #[cfg(windows)]
    let _ = enable_ansi_support::enable_ansi_support();

    let mut rng = fastrand::Rng::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--seed" => match args.next().and_then(|s| s.parse().ok()) {
                Some(seed) => rng.seed(seed),
                None => usage(),
            },
            "--version" | "-V" => {
                println!("ski {VERSION}");
                return;
            }
            _ => usage(),
        }
    }
    let color = std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty())
        && std::env::var("TERM").map_or(true, |t| t != "dumb");

    COLOR.store(color, std::sync::atomic::Ordering::Relaxed);
    let _ = ctrlc::set_handler(|| bye());

    println!("SKI!  Version {VERSION}.  Type ? for help.");
    while run(color, &mut rng) {
        print!("\nPlay again? (y/n) ");
        match read_line() {
            Some(l) if l.trim().to_ascii_lowercase().starts_with('y') => println!(),
            _ => break,
        }
    }
    bye();
}
