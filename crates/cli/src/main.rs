use rulatro_core::{
    BlindKind, BlindOutcome, Card, ConsumableKind, Edition, Enhancement, Event, EventBus, PackOpen,
    PackOption, Phase, Rank, RunError, RunState, ScoreBreakdown, ScoreTables, ScoreTraceStep, Seal,
    ShopOfferRef, Suit,
};
use rulatro_data::{load_content_with_mods_locale, load_game_config, normalize_locale};
use rulatro_modding::ModManager;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
#[cfg(unix)]
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UiLocale {
    EnUs,
    ZhCn,
}

impl UiLocale {
    fn code(self) -> &'static str {
        match self {
            Self::EnUs => "en_US",
            Self::ZhCn => "zh_CN",
        }
    }

    fn from_opt(value: Option<&str>) -> Self {
        let normalized = normalize_locale(value);
        if normalized == "zh_CN" {
            Self::ZhCn
        } else {
            Self::EnUs
        }
    }

    fn text<'a>(self, en: &'a str, zh: &'a str) -> &'a str {
        if matches!(self, Self::ZhCn) {
            zh
        } else {
            en
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CliOptions {
    auto: bool,
    cui: bool,
    menu: bool,
    seed: Option<u64>,
    locale: UiLocale,
}

const COMPLETION_COMMANDS: &[&str] = &[
    "?",
    "a",
    "actions",
    "board",
    "buy",
    "d",
    "data",
    "deal",
    "deck",
    "discard",
    "edit",
    "exit",
    "h",
    "hand",
    "help",
    "inv",
    "inventory",
    "leave",
    "levels",
    "load",
    "ls",
    "n",
    "next",
    "overview",
    "p",
    "pack",
    "peek",
    "pick",
    "play",
    "quit",
    "r",
    "ref",
    "reroll",
    "reward",
    "s",
    "sell",
    "sh",
    "shop",
    "skip",
    "skip_blind",
    "skip_pack",
    "sp",
    "save",
    "state",
    "status",
    "summary",
    "tags",
    "use",
    "x",
];

const BUY_COMPLETIONS: &[&str] = &["card", "pack", "voucher"];
const PEEK_COMPLETIONS: &[&str] = &["draw", "discard"];

const SAVE_SCHEMA_VERSION: u32 = 1;
const DEFAULT_RUN_SEED: u64 = 0xC0FFEE;

fn default_run_seed() -> u64 {
    DEFAULT_RUN_SEED
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedAction {
    action: String,
    #[serde(default)]
    indices: Vec<usize>,
    #[serde(default)]
    target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedRunState {
    version: u32,
    locale: String,
    #[serde(default = "default_run_seed")]
    seed: u64,
    #[serde(default)]
    content_signature: String,
    actions: Vec<SavedAction>,
}

#[derive(Default)]
struct LineEditor {
    history: Vec<String>,
    history_path: Option<PathBuf>,
    history_index: Option<usize>,
    draft_line: Option<String>,
}

impl LineEditor {
    fn new() -> Self {
        let mut editor = Self {
            history_path: default_history_path(),
            ..Self::default()
        };
        editor.load_history();
        editor
    }

    #[cfg(unix)]
    fn read_line(&mut self, prompt: &str) -> Option<String> {
        let stdin = io::stdin();
        let fd = stdin.as_raw_fd();
        if unsafe { libc::isatty(fd) } != 1 {
            return self.read_line_basic(prompt);
        }
        let _raw = match RawMode::new(fd) {
            Ok(raw) => raw,
            Err(_) => return self.read_line_basic(prompt),
        };
        let mut input = stdin.lock();
        let mut buffer = String::new();
        let mut cursor = 0usize;
        self.history_index = None;
        self.draft_line = None;
        redraw_prompt_line(prompt, &buffer, cursor);
        loop {
            let mut byte = [0u8; 1];
            if input.read_exact(&mut byte).is_err() {
                return None;
            }
            match byte[0] {
                b'\n' | b'\r' => {
                    println!();
                    self.push_history(&buffer);
                    return Some(buffer);
                }
                3 => {
                    println!();
                    return Some(String::new());
                }
                4 => {
                    if buffer.is_empty() {
                        println!();
                        return None;
                    }
                }
                9 => {
                    let completion = complete_line(&buffer, cursor);
                    if let Some(updated) = completion.updated_line {
                        buffer = updated;
                        cursor = buffer.len();
                    }
                    if !completion.suggestions.is_empty() {
                        println!();
                        println!("{}", completion.suggestions.join("  "));
                    }
                    redraw_prompt_line(prompt, &buffer, cursor);
                    continue;
                }
                127 | 8 => {
                    if cursor > 0 {
                        let previous = prev_char_boundary(&buffer, cursor);
                        buffer.drain(previous..cursor);
                        cursor = previous;
                        self.history_index = None;
                    }
                }
                b'\x1b' => match read_escape_key(&mut input) {
                    Some(EscapeKey::Up) => self.history_prev(&mut buffer, &mut cursor),
                    Some(EscapeKey::Down) => self.history_next(&mut buffer, &mut cursor),
                    Some(EscapeKey::Left) => {
                        cursor = prev_char_boundary(&buffer, cursor);
                    }
                    Some(EscapeKey::Right) => {
                        cursor = next_char_boundary(&buffer, cursor);
                    }
                    Some(EscapeKey::Home) => cursor = 0,
                    Some(EscapeKey::End) => cursor = buffer.len(),
                    Some(EscapeKey::Delete) => {
                        if cursor < buffer.len() {
                            let next = next_char_boundary(&buffer, cursor);
                            buffer.drain(cursor..next);
                            self.history_index = None;
                        }
                    }
                    None => {}
                },
                byte if byte.is_ascii_control() => {}
                byte => {
                    let ch = byte as char;
                    buffer.insert(cursor, ch);
                    cursor += ch.len_utf8();
                    self.history_index = None;
                }
            }
            redraw_prompt_line(prompt, &buffer, cursor);
        }
    }

    #[cfg(not(unix))]
    fn read_line(&mut self, prompt: &str) -> Option<String> {
        self.read_line_basic(prompt)
    }

    fn read_line_basic(&mut self, prompt: &str) -> Option<String> {
        print!("{prompt}");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end_matches(&['\n', '\r'][..]).to_string();
        self.push_history(&line);
        Some(line)
    }

    fn history_prev(&mut self, buffer: &mut String, cursor: &mut usize) {
        if self.history.is_empty() {
            return;
        }
        match self.history_index {
            Some(0) => {}
            Some(index) => {
                self.history_index = Some(index.saturating_sub(1));
            }
            None => {
                self.draft_line = Some(buffer.clone());
                self.history_index = Some(self.history.len() - 1);
            }
        }
        if let Some(index) = self.history_index {
            *buffer = self.history[index].clone();
            *cursor = buffer.len();
        }
    }

    fn history_next(&mut self, buffer: &mut String, cursor: &mut usize) {
        let Some(index) = self.history_index else {
            return;
        };
        if index + 1 < self.history.len() {
            self.history_index = Some(index + 1);
            *buffer = self.history[index + 1].clone();
        } else {
            self.history_index = None;
            *buffer = self.draft_line.take().unwrap_or_default();
        }
        *cursor = buffer.len();
    }

    fn push_history(&mut self, line: &str) {
        let line = line.trim();
        if line.is_empty() {
            return;
        }
        if self.history.last().is_some_and(|last| last == line) {
            return;
        }
        self.history.push(line.to_string());
        if self.history.len() > 500 {
            let drop = self.history.len() - 500;
            self.history.drain(0..drop);
        }
    }

    fn load_history(&mut self) {
        let Some(path) = self.history_path.as_ref() else {
            return;
        };
        let Ok(contents) = fs::read_to_string(path) else {
            return;
        };
        self.history = contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect();
    }

    fn save_history(&self) {
        let Some(path) = self.history_path.as_ref() else {
            return;
        };
        let mut contents = self.history.join("\n");
        if !contents.is_empty() {
            contents.push('\n');
        }
        if let Err(err) = fs::write(path, contents) {
            eprintln!("history warning: {err}");
        }
    }
}

fn default_history_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("RULATRO_HISTORY") {
        return Some(PathBuf::from(path));
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".rulatro_cli_history"))
}

fn default_state_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("RULATRO_SAVE") {
        return Some(PathBuf::from(path));
    }
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".rulatro_cli_state.json"))
}

fn parse_optional_path(args: &[&str]) -> Option<PathBuf> {
    args.first().map(PathBuf::from).or_else(default_state_path)
}

#[derive(Clone, Copy)]
struct Fnv64(u64);

impl Fnv64 {
    fn new() -> Self {
        Self(0xcbf29ce484222325)
    }

    fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }

    fn finish(self) -> u64 {
        self.0
    }
}

fn hash_dir_tree(base: &Path, rel: &Path, hasher: &mut Fnv64) -> Result<(), String> {
    let path = base.join(rel);
    if !path.exists() {
        return Ok(());
    }
    let mut entries: Vec<_> = fs::read_dir(&path)
        .map_err(|err| err.to_string())?
        .filter_map(Result::ok)
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_name = entry.file_name();
        let rel_path = if rel.as_os_str().is_empty() {
            PathBuf::from(&file_name)
        } else {
            rel.join(&file_name)
        };
        let entry_path = entry.path();
        if entry_path.is_dir() {
            hasher.update(b"D");
            hasher.update(rel_path.to_string_lossy().as_bytes());
            hasher.update(&[0]);
            hash_dir_tree(base, &rel_path, hasher)?;
        } else if entry_path.is_file() {
            hasher.update(b"F");
            hasher.update(rel_path.to_string_lossy().as_bytes());
            hasher.update(&[0]);
            let bytes = fs::read(&entry_path).map_err(|err| err.to_string())?;
            hasher.update(&(bytes.len() as u64).to_le_bytes());
            hasher.update(&bytes);
        }
    }
    Ok(())
}

fn compute_content_signature(locale: UiLocale) -> Result<String, String> {
    let mut hasher = Fnv64::new();
    hasher.update(b"rulatro-save-signature-v1");
    hasher.update(locale.code().as_bytes());
    hash_dir_tree(Path::new("assets"), Path::new(""), &mut hasher)?;
    hash_dir_tree(Path::new("mods"), Path::new(""), &mut hasher)?;
    Ok(format!("{:016x}", hasher.finish()))
}

fn push_recorded_action(
    actions: &mut Vec<SavedAction>,
    action: &str,
    indices: Vec<usize>,
    target: Option<String>,
) {
    actions.push(SavedAction {
        action: action.to_string(),
        indices,
        target,
    });
}

fn save_state_file(
    locale: UiLocale,
    seed: u64,
    content_signature: &str,
    actions: &[SavedAction],
    path: &Path,
) -> Result<(), String> {
    let payload = SavedRunState {
        version: SAVE_SCHEMA_VERSION,
        locale: locale.code().to_string(),
        seed,
        content_signature: content_signature.to_string(),
        actions: actions.to_vec(),
    };
    let body = serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?;
    fs::write(path, body).map_err(|err| err.to_string())
}

fn load_state_file(path: &Path) -> Result<SavedRunState, String> {
    let body = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let payload: SavedRunState = serde_json::from_str(&body).map_err(|err| err.to_string())?;
    if payload.version != SAVE_SCHEMA_VERSION {
        return Err(format!(
            "unsupported save version {} (expected {})",
            payload.version, SAVE_SCHEMA_VERSION
        ));
    }
    Ok(payload)
}

fn apply_saved_action(
    run: &mut RunState,
    events: &mut EventBus,
    open_pack: &mut Option<PackOpen>,
    action: &SavedAction,
) -> Result<(), String> {
    match action.action.as_str() {
        "deal" => run.prepare_hand(events).map_err(|err| err.to_string())?,
        "play" => {
            run.play_hand(&action.indices, events)
                .map_err(|err| err.to_string())?;
        }
        "discard" => run
            .discard(&action.indices, events)
            .map_err(|err| err.to_string())?,
        "skip_blind" => run.skip_blind(events).map_err(|err| err.to_string())?,
        "enter_shop" => run.enter_shop(events).map_err(|err| err.to_string())?,
        "leave_shop" => {
            run.leave_shop();
            *open_pack = None;
        }
        "reroll" => run.reroll_shop(events).map_err(|err| err.to_string())?,
        "buy_card" => {
            let idx = action
                .target
                .as_deref()
                .ok_or_else(|| "missing target index".to_string())?
                .parse::<usize>()
                .map_err(|_| "invalid index".to_string())?;
            let purchase = run
                .buy_shop_offer(ShopOfferRef::Card(idx), events)
                .map_err(|err| err.to_string())?;
            run.apply_purchase(&purchase)
                .map_err(|err| err.to_string())?;
        }
        "buy_pack" => {
            let idx = action
                .target
                .as_deref()
                .ok_or_else(|| "missing target index".to_string())?
                .parse::<usize>()
                .map_err(|_| "invalid index".to_string())?;
            let purchase = run
                .buy_shop_offer(ShopOfferRef::Pack(idx), events)
                .map_err(|err| err.to_string())?;
            let open = run
                .open_pack_purchase(&purchase, events)
                .map_err(|err| err.to_string())?;
            *open_pack = Some(open);
        }
        "buy_voucher" => {
            let idx = action
                .target
                .as_deref()
                .ok_or_else(|| "missing target index".to_string())?
                .parse::<usize>()
                .map_err(|_| "invalid index".to_string())?;
            let purchase = run
                .buy_shop_offer(ShopOfferRef::Voucher(idx), events)
                .map_err(|err| err.to_string())?;
            run.apply_purchase(&purchase)
                .map_err(|err| err.to_string())?;
        }
        "pick_pack" => {
            let open = open_pack
                .clone()
                .ok_or_else(|| "no open pack".to_string())?;
            run.choose_pack_options(&open, &action.indices, events)
                .map_err(|err| err.to_string())?;
            *open_pack = None;
        }
        "skip_pack" => {
            let open = open_pack
                .clone()
                .ok_or_else(|| "no open pack".to_string())?;
            run.skip_pack(&open, events)
                .map_err(|err| err.to_string())?;
            *open_pack = None;
        }
        "use_consumable" => {
            let idx = action
                .target
                .as_deref()
                .ok_or_else(|| "missing target index".to_string())?
                .parse::<usize>()
                .map_err(|_| "invalid index".to_string())?;
            run.use_consumable(idx, &action.indices, events)
                .map_err(|err| err.to_string())?;
        }
        "sell_joker" => {
            let idx = action
                .target
                .as_deref()
                .ok_or_else(|| "missing target index".to_string())?
                .parse::<usize>()
                .map_err(|_| "invalid index".to_string())?;
            run.sell_joker(idx, events).map_err(|err| err.to_string())?;
        }
        "next_blind" => {
            run.start_next_blind(events)
                .map_err(|err| err.to_string())?;
            *open_pack = None;
        }
        _ => return Err(format!("unknown saved action '{}'", action.action)),
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum EscapeKey {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Delete,
}

struct CompletionResult {
    updated_line: Option<String>,
    suggestions: Vec<String>,
}

fn complete_line(line: &str, cursor: usize) -> CompletionResult {
    if cursor != line.len() {
        return CompletionResult {
            updated_line: None,
            suggestions: Vec::new(),
        };
    }
    let trimmed = line.trim_end();
    if trimmed.is_empty() {
        let suggestions = COMPLETION_COMMANDS
            .iter()
            .map(|item| item.to_string())
            .collect();
        return CompletionResult {
            updated_line: None,
            suggestions,
        };
    }
    let mut parts = trimmed.split_whitespace();
    let Some(first) = parts.next() else {
        return CompletionResult {
            updated_line: None,
            suggestions: Vec::new(),
        };
    };
    let second = parts.next();
    let more_args = parts.next().is_some();
    let is_first_token = !line.contains(' ');
    if is_first_token {
        return complete_token(first, COMPLETION_COMMANDS, true);
    }
    if more_args {
        return CompletionResult {
            updated_line: None,
            suggestions: Vec::new(),
        };
    }
    if line.ends_with(' ') {
        if second.is_none() {
            let suggestions = completion_table_for(first)
                .iter()
                .map(|item| item.to_string())
                .collect();
            return CompletionResult {
                updated_line: None,
                suggestions,
            };
        }
        return CompletionResult {
            updated_line: None,
            suggestions: Vec::new(),
        };
    }
    if let Some(current_second) = second {
        let base_len = line.len() - current_second.len();
        let mut result = complete_token(current_second, completion_table_for(first), true);
        if let Some(updated) = result.updated_line.take() {
            result.updated_line = Some(format!("{}{}", &line[..base_len], updated));
        }
        return result;
    }
    CompletionResult {
        updated_line: None,
        suggestions: Vec::new(),
    }
}

fn complete_token(token: &str, table: &[&str], append_space: bool) -> CompletionResult {
    let matches: Vec<&str> = table
        .iter()
        .copied()
        .filter(|item| item.starts_with(token))
        .collect();
    if matches.is_empty() {
        return CompletionResult {
            updated_line: None,
            suggestions: Vec::new(),
        };
    }
    if matches.len() == 1 {
        let mut value = matches[0].to_string();
        if append_space {
            value.push(' ');
        }
        return CompletionResult {
            updated_line: Some(value),
            suggestions: Vec::new(),
        };
    }
    let common = longest_common_prefix(&matches);
    let updated = if common.len() > token.len() {
        Some(common)
    } else {
        None
    };
    let suggestions = matches.iter().map(|item| item.to_string()).collect();
    CompletionResult {
        updated_line: updated,
        suggestions,
    }
}

fn completion_table_for(command: &str) -> &'static [&'static str] {
    match command {
        "buy" => BUY_COMPLETIONS,
        "peek" => PEEK_COMPLETIONS,
        _ => &[],
    }
}

fn longest_common_prefix(matches: &[&str]) -> String {
    let Some(first) = matches.first() else {
        return String::new();
    };
    let mut prefix = (*first).to_string();
    for entry in &matches[1..] {
        while !entry.starts_with(&prefix) {
            if prefix.is_empty() {
                return prefix;
            }
            prefix.pop();
        }
    }
    prefix
}

fn prev_char_boundary(text: &str, index: usize) -> usize {
    if index == 0 {
        return 0;
    }
    text[..index]
        .char_indices()
        .last()
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

fn next_char_boundary(text: &str, index: usize) -> usize {
    if index >= text.len() {
        return text.len();
    }
    index
        + text[index..]
            .chars()
            .next()
            .map(char::len_utf8)
            .unwrap_or(0)
}

fn redraw_prompt_line(prompt: &str, line: &str, cursor: usize) {
    print!("\r\x1b[2K{prompt}{line}");
    let line_chars = line.chars().count();
    let cursor_chars = line[..cursor].chars().count();
    let move_left = line_chars.saturating_sub(cursor_chars);
    if move_left > 0 {
        print!("\x1b[{move_left}D");
    }
    let _ = io::stdout().flush();
}

#[cfg(unix)]
fn read_escape_key(input: &mut impl Read) -> Option<EscapeKey> {
    let mut first = [0u8; 1];
    input.read_exact(&mut first).ok()?;
    match first[0] {
        b'[' => {
            let mut second = [0u8; 1];
            input.read_exact(&mut second).ok()?;
            match second[0] {
                b'A' => Some(EscapeKey::Up),
                b'B' => Some(EscapeKey::Down),
                b'C' => Some(EscapeKey::Right),
                b'D' => Some(EscapeKey::Left),
                b'H' => Some(EscapeKey::Home),
                b'F' => Some(EscapeKey::End),
                b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' => {
                    let mut code = vec![second[0]];
                    loop {
                        let mut next = [0u8; 1];
                        input.read_exact(&mut next).ok()?;
                        if next[0] == b'~' {
                            break;
                        }
                        code.push(next[0]);
                    }
                    match code.as_slice() {
                        b"1" | b"7" => Some(EscapeKey::Home),
                        b"3" => Some(EscapeKey::Delete),
                        b"4" | b"8" => Some(EscapeKey::End),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        b'O' => {
            let mut second = [0u8; 1];
            input.read_exact(&mut second).ok()?;
            match second[0] {
                b'H' => Some(EscapeKey::Home),
                b'F' => Some(EscapeKey::End),
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(unix)]
struct RawMode {
    fd: i32,
    original: libc::termios,
}

#[cfg(unix)]
impl RawMode {
    fn new(fd: i32) -> io::Result<Self> {
        let mut original = unsafe { std::mem::zeroed::<libc::termios>() };
        if unsafe { libc::tcgetattr(fd, &mut original) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let mut raw = original;
        raw.c_lflag &= !(libc::ICANON | libc::ECHO);
        raw.c_iflag &= !(libc::IXON | libc::ICRNL);
        raw.c_cc[libc::VMIN] = 1;
        raw.c_cc[libc::VTIME] = 0;
        if unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, &raw) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { fd, original })
    }
}

#[cfg(unix)]
impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = unsafe { libc::tcsetattr(self.fd, libc::TCSAFLUSH, &self.original) };
    }
}

fn parse_cli_options(args: &[String]) -> CliOptions {
    let mut auto = false;
    let mut cui = false;
    let mut menu = false;
    let mut seed = None;
    let mut locale_arg: Option<String> = std::env::var("RULATRO_LANG").ok();
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--auto" => auto = true,
            "--cui" => cui = true,
            "--menu" => menu = true,
            "--lang" | "-l" => {
                if let Some(value) = args.get(idx + 1) {
                    locale_arg = Some(value.clone());
                    idx += 1;
                }
            }
            "--seed" => {
                if let Some(value) = args.get(idx + 1) {
                    seed = value.parse::<u64>().ok();
                    idx += 1;
                }
            }
            _ => {}
        }
        idx += 1;
    }
    CliOptions {
        auto,
        cui,
        menu,
        seed,
        locale: UiLocale::from_opt(locale_arg.as_deref()),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let options = parse_cli_options(&args);
    if options.cui {
        let launch = rulatro_cui::LaunchOptions {
            locale: Some(options.locale.code().to_string()),
            seed: options.seed,
        };
        if let Err(err) = rulatro_cui::run(launch) {
            eprintln!("cui launch error: {err}");
            std::process::exit(1);
        }
        return;
    }
    if options.auto {
        run_auto(options.locale);
        return;
    }
    run_cui(options.locale, options.menu);
}

fn build_run_with_seed(
    locale: UiLocale,
    seed: u64,
) -> Result<(RunState, Vec<String>, Vec<String>, String), String> {
    let config = load_game_config(Path::new("assets")).map_err(|err| err.to_string())?;
    let modded =
        load_content_with_mods_locale(Path::new("assets"), Path::new("mods"), Some(locale.code()))
            .map_err(|err| err.to_string())?;
    let mod_ids: Vec<String> = modded
        .mods
        .iter()
        .map(|item| item.manifest.meta.id.clone())
        .collect();
    let warnings = modded.warnings.clone();
    let mut runtime = ModManager::new();
    runtime
        .load_mods(&modded.mods)
        .map_err(|err| err.to_string())?;
    let mut run = RunState::new(config, modded.content, seed);
    run.set_mod_runtime(Some(Box::new(runtime)));
    let content_signature = compute_content_signature(locale)?;
    Ok((run, mod_ids, warnings, content_signature))
}

fn build_run(locale: UiLocale) -> Result<(RunState, Vec<String>, Vec<String>, String), String> {
    build_run_with_seed(locale, DEFAULT_RUN_SEED)
}

fn run_auto(locale: UiLocale) {
    let mut events = EventBus::default();
    println!("locale: {}", locale.code());
    let (mut run, mod_ids, warnings, _content_signature) = build_run(locale).expect("load run");
    if !mod_ids.is_empty() {
        println!("mods loaded: {}", mod_ids.len());
        for mod_id in &mod_ids {
            println!("mod: {}", mod_id);
        }
    }
    for warning in &warnings {
        eprintln!("mod warning: {}", warning);
    }
    run.start_blind(1, BlindKind::Small, &mut events)
        .expect("start blind");

    let mut blinds_completed = 0;
    loop {
        run.prepare_hand(&mut events).expect("prepare hand");

        let play_count = run.hand.len().min(5);
        let indices: Vec<usize> = (0..play_count).collect();
        let breakdown = run.play_hand(&indices, &mut events).expect("play hand");

        println!(
            "hand: {:?}, chips: {}, mult: {:.2}, total: {}",
            breakdown.hand,
            breakdown.total.chips,
            breakdown.total.mult,
            breakdown.total.total()
        );
        println!(
            "blind score: {} / target: {}",
            run.state.blind_score, run.state.target
        );

        if let Some(outcome) = run.blind_outcome() {
            println!("blind outcome: {:?}", outcome);
            match outcome {
                BlindOutcome::Cleared => {
                    blinds_completed += 1;
                    run.enter_shop(&mut events).expect("enter shop");
                    if run.reroll_shop(&mut events).is_ok() {
                        println!("shop rerolled");
                    }
                    if let Ok(purchase) = run.buy_shop_offer(ShopOfferRef::Pack(0), &mut events) {
                        if let Ok(open) = run.open_pack_purchase(&purchase, &mut events) {
                            let _ = run.choose_pack_options(&open, &[0], &mut events);
                            println!("opened pack with {} options", open.options.len());
                        }
                    } else if let Ok(purchase) =
                        run.buy_shop_offer(ShopOfferRef::Card(0), &mut events)
                    {
                        let _ = run.apply_purchase(&purchase);
                        println!("bought card 0");
                    }
                    run.leave_shop();
                    if run.start_next_blind(&mut events).is_err() {
                        break;
                    }
                }
                BlindOutcome::Failed => {
                    break;
                }
            }
        }

        if blinds_completed >= 2 {
            break;
        }
    }

    for event in events.drain() {
        println!("event: {:?}", event);
    }
}

#[derive(Debug, Clone, Copy)]
enum MenuCommand {
    Static(&'static str),
    Play,
    Discard,
    Buy,
    Pick,
    Use,
    Sell,
    Peek,
    Save,
    Load,
    Custom,
    Quit,
}

#[derive(Debug, Clone)]
struct MenuEntry {
    label: String,
    command: MenuCommand,
}

fn menu_entry(locale: UiLocale, en: &str, zh: &str, command: MenuCommand) -> MenuEntry {
    MenuEntry {
        label: locale.text(en, zh).to_string(),
        command,
    }
}

fn menu_static(locale: UiLocale, en: &str, zh: &str, command: &'static str) -> MenuEntry {
    menu_entry(locale, en, zh, MenuCommand::Static(command))
}

fn read_next_command(
    locale: UiLocale,
    line_editor: &mut LineEditor,
    run: &RunState,
    open_pack: Option<&PackOpen>,
    menu_mode: bool,
) -> Option<String> {
    if menu_mode {
        prompt_menu_command(locale, line_editor, run, open_pack)
    } else {
        let prompt = prompt_text(locale, run, open_pack);
        line_editor.read_line(&prompt)
    }
}

fn build_menu_entries(
    locale: UiLocale,
    run: &RunState,
    open_pack: Option<&PackOpen>,
) -> Vec<MenuEntry> {
    let mut entries = vec![
        menu_static(locale, "Show overview", "查看总览", "board"),
        menu_static(locale, "Show hand", "查看手牌", "hand"),
        menu_static(locale, "Show inventory", "查看背包", "inv"),
        menu_static(locale, "Show action guide", "查看推荐操作", "actions"),
        menu_static(locale, "Show summary", "查看摘要", "summary"),
    ];

    if open_pack.is_some() {
        entries.push(menu_entry(
            locale,
            "Pick pack options",
            "选择卡包选项",
            MenuCommand::Pick,
        ));
        entries.push(menu_static(locale, "Skip pack", "跳过卡包", "skip_pack"));
        entries.push(menu_static(locale, "Show pack", "查看卡包", "pack"));
    } else {
        match run.state.phase {
            Phase::Deal => {
                entries.push(menu_static(locale, "Deal hand", "发牌", "deal"));
                if run.state.blind != BlindKind::Boss {
                    entries.push(menu_static(locale, "Skip blind", "跳过盲注", "skip"));
                }
            }
            Phase::Play => {
                entries.push(menu_entry(locale, "Play cards", "出牌", MenuCommand::Play));
                if run.state.discards_left > 0 {
                    entries.push(menu_entry(
                        locale,
                        "Discard cards",
                        "弃牌",
                        MenuCommand::Discard,
                    ));
                }
            }
            Phase::Shop => {
                entries.push(menu_entry(
                    locale,
                    "Buy from shop",
                    "购买商店商品",
                    MenuCommand::Buy,
                ));
                entries.push(menu_static(locale, "Reroll shop", "刷新商店", "reroll"));
                entries.push(menu_static(locale, "Leave shop", "离开商店", "leave"));
            }
            Phase::Setup => entries.push(menu_static(
                locale,
                "Start next blind",
                "开始下一盲注",
                "next",
            )),
            Phase::Score | Phase::Cleanup => {}
        }

        if matches!(run.blind_outcome(), Some(BlindOutcome::Cleared))
            && run.state.phase != Phase::Shop
        {
            entries.push(menu_static(locale, "Enter shop", "进入商店", "shop"));
        }
        if !run.inventory.consumables.is_empty() {
            entries.push(menu_entry(
                locale,
                "Use consumable",
                "使用消耗牌",
                MenuCommand::Use,
            ));
        }
        if !run.inventory.jokers.is_empty() {
            entries.push(menu_entry(
                locale,
                "Sell joker",
                "出售小丑",
                MenuCommand::Sell,
            ));
        }
        entries.push(menu_entry(
            locale,
            "Peek draw/discard",
            "查看牌堆顶部",
            MenuCommand::Peek,
        ));
    }

    entries.push(menu_entry(
        locale,
        "Save run",
        "保存进度",
        MenuCommand::Save,
    ));
    entries.push(menu_entry(
        locale,
        "Load run",
        "加载进度",
        MenuCommand::Load,
    ));
    entries.push(menu_static(locale, "Show help", "显示帮助", "help"));
    entries.push(menu_entry(
        locale,
        "Type custom command",
        "输入自定义命令",
        MenuCommand::Custom,
    ));
    entries.push(menu_entry(locale, "Quit", "退出", MenuCommand::Quit));
    entries
}

fn prompt_menu_command(
    locale: UiLocale,
    line_editor: &mut LineEditor,
    run: &RunState,
    open_pack: Option<&PackOpen>,
) -> Option<String> {
    loop {
        println!();
        println!("{}", locale.text("== CUI Menu ==", "== CUI 菜单 =="));
        print_flow_summary(locale, run, open_pack);
        let entries = build_menu_entries(locale, run, open_pack);
        for (idx, entry) in entries.iter().enumerate() {
            println!("  {:>2}. {}", idx + 1, entry.label);
        }
        let selection_prompt = locale.text(
            "select number (or type command directly) > ",
            "选择编号（或直接输入命令）> ",
        );
        let line = line_editor.read_line(selection_prompt)?;
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if let Ok(choice) = input.parse::<usize>() {
            if !(1..=entries.len()).contains(&choice) {
                println!("{}", locale.text("invalid selection", "选择无效"));
                continue;
            }
            return execute_menu_command(
                locale,
                line_editor,
                run,
                open_pack,
                entries[choice - 1].command,
            );
        }
        return Some(input.to_string());
    }
}

fn execute_menu_command(
    locale: UiLocale,
    line_editor: &mut LineEditor,
    run: &RunState,
    open_pack: Option<&PackOpen>,
    command: MenuCommand,
) -> Option<String> {
    match command {
        MenuCommand::Static(value) => Some(value.to_string()),
        MenuCommand::Quit => Some("quit".to_string()),
        MenuCommand::Play => prompt_indices_command(locale, line_editor, run, "play"),
        MenuCommand::Discard => prompt_indices_command(locale, line_editor, run, "discard"),
        MenuCommand::Buy => prompt_buy_command(locale, line_editor, run),
        MenuCommand::Pick => prompt_pick_command(locale, line_editor, run, open_pack),
        MenuCommand::Use => prompt_use_command(locale, line_editor, run),
        MenuCommand::Sell => prompt_sell_command(locale, line_editor, run),
        MenuCommand::Peek => prompt_peek_command(locale, line_editor),
        MenuCommand::Save => prompt_save_or_load_command(locale, line_editor, "save"),
        MenuCommand::Load => prompt_save_or_load_command(locale, line_editor, "load"),
        MenuCommand::Custom => prompt_custom_command(locale, line_editor),
    }
}

fn prompt_indices_command(
    locale: UiLocale,
    line_editor: &mut LineEditor,
    run: &RunState,
    action: &'static str,
) -> Option<String> {
    print_hand(locale, run);
    let prompt = if action == "play" {
        locale.text(
            "play indices (example: 0 1 2 or 0-2) > ",
            "输入出牌索引（例：0 1 2 或 0-2）> ",
        )
    } else {
        locale.text(
            "discard indices (example: 0 1 2 or 0-2) > ",
            "输入弃牌索引（例：0 1 2 或 0-2）> ",
        )
    };
    loop {
        let line = line_editor.read_line(prompt)?;
        let args: Vec<&str> = line.split_whitespace().collect();
        match parse_indices_result(&args) {
            Ok(indices) => {
                let indices_text = indices
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                return Some(format!("{action} {indices_text}"));
            }
            Err(err) => println!(
                "{}: {}",
                locale.text("error", "错误"),
                localize_parse_error(locale, &err)
            ),
        }
    }
}

fn prompt_buy_command(
    locale: UiLocale,
    line_editor: &mut LineEditor,
    run: &RunState,
) -> Option<String> {
    print_shop(locale, run);
    let prompt = locale.text(
        "buy target: card|pack|voucher <index> > ",
        "输入购买目标：card|pack|voucher <index> > ",
    );
    loop {
        let line = line_editor.read_line(prompt)?;
        let args: Vec<&str> = line.split_whitespace().collect();
        if args.len() != 2 {
            println!(
                "{}",
                locale.text(
                    "usage: card|pack|voucher <index>",
                    "用法：card|pack|voucher <index>"
                )
            );
            continue;
        }
        let kind = args[0];
        if kind != "card" && kind != "pack" && kind != "voucher" {
            println!(
                "{}",
                locale.text(
                    "kind must be card, pack, or voucher",
                    "类型必须是 card、pack 或 voucher",
                )
            );
            continue;
        }
        match args[1].parse::<usize>() {
            Ok(idx) => return Some(format!("buy {kind} {idx}")),
            Err(_) => println!("{}", locale.text("invalid index", "无效索引")),
        }
    }
}

fn prompt_pick_command(
    locale: UiLocale,
    line_editor: &mut LineEditor,
    run: &RunState,
    open_pack: Option<&PackOpen>,
) -> Option<String> {
    if let Some(open) = open_pack {
        print_pack_open(locale, open, run);
    }
    let prompt = locale.text(
        "pick indices (example: 0 or 0 1) > ",
        "输入选择索引（例：0 或 0 1）> ",
    );
    loop {
        let line = line_editor.read_line(prompt)?;
        let args: Vec<&str> = line.split_whitespace().collect();
        match parse_indices_result(&args) {
            Ok(indices) => {
                let indices_text = indices
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                return Some(format!("pick {indices_text}"));
            }
            Err(err) => println!(
                "{}: {}",
                locale.text("error", "错误"),
                localize_parse_error(locale, &err)
            ),
        }
    }
}

fn prompt_use_command(
    locale: UiLocale,
    line_editor: &mut LineEditor,
    run: &RunState,
) -> Option<String> {
    print_inventory(locale, run);
    let prompt = locale.text(
        "use: <consumable_index> [selected hand idx..] > ",
        "输入 use：<消耗牌索引> [手牌索引..] > ",
    );
    loop {
        let line = line_editor.read_line(prompt)?;
        let args: Vec<&str> = line.split_whitespace().collect();
        if args.is_empty() {
            println!(
                "{}",
                locale.text(
                    "usage: <consumable_index> [selected idxs]",
                    "用法：<消耗牌索引> [手牌索引]",
                )
            );
            continue;
        }
        let idx = match args[0].parse::<usize>() {
            Ok(value) => value,
            Err(_) => {
                println!("{}", locale.text("invalid index", "无效索引"));
                continue;
            }
        };
        let selected = match parse_optional_indices(&args[1..]) {
            Ok(indices) => indices,
            Err(err) => {
                println!(
                    "{}: {}",
                    locale.text("error", "错误"),
                    localize_parse_error(locale, &err)
                );
                continue;
            }
        };
        let mut command = format!("use {idx}");
        if !selected.is_empty() {
            let selected_text = selected
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            command.push(' ');
            command.push_str(&selected_text);
        }
        return Some(command);
    }
}

fn prompt_sell_command(
    locale: UiLocale,
    line_editor: &mut LineEditor,
    run: &RunState,
) -> Option<String> {
    print_inventory(locale, run);
    let prompt = locale.text("sell joker index > ", "输入要出售的小丑索引 > ");
    loop {
        let line = line_editor.read_line(prompt)?;
        match line.trim().parse::<usize>() {
            Ok(idx) => return Some(format!("sell {idx}")),
            Err(_) => println!("{}", locale.text("invalid index", "无效索引")),
        }
    }
}

fn prompt_peek_command(locale: UiLocale, line_editor: &mut LineEditor) -> Option<String> {
    let prompt = locale.text(
        "peek target: draw|discard [count] > ",
        "输入 peek 目标：draw|discard [数量] > ",
    );
    loop {
        let line = line_editor.read_line(prompt)?;
        let args: Vec<&str> = line.split_whitespace().collect();
        if args.is_empty() {
            println!(
                "{}",
                locale.text("usage: draw|discard [count]", "用法：draw|discard [数量]")
            );
            continue;
        }
        let target = args[0];
        if target != "draw" && target != "discard" {
            println!(
                "{}",
                locale.text(
                    "target must be draw or discard",
                    "目标必须是 draw 或 discard"
                )
            );
            continue;
        }
        if let Some(count_text) = args.get(1) {
            match count_text.parse::<usize>() {
                Ok(count) => return Some(format!("peek {target} {count}")),
                Err(_) => {
                    println!("{}", locale.text("invalid count", "数量无效"));
                    continue;
                }
            }
        }
        return Some(format!("peek {target}"));
    }
}

fn prompt_save_or_load_command(
    locale: UiLocale,
    line_editor: &mut LineEditor,
    command: &'static str,
) -> Option<String> {
    let prompt = if command == "save" {
        locale.text("save path (empty=default) > ", "保存路径（留空=默认）> ")
    } else {
        locale.text("load path (empty=default) > ", "加载路径（留空=默认）> ")
    };
    let line = line_editor.read_line(prompt)?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        Some(command.to_string())
    } else {
        Some(format!("{command} {trimmed}"))
    }
}

fn prompt_custom_command(locale: UiLocale, line_editor: &mut LineEditor) -> Option<String> {
    let prompt = locale.text("custom command > ", "自定义命令 > ");
    loop {
        let line = line_editor.read_line(prompt)?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            println!("{}", locale.text("command cannot be empty", "命令不能为空"));
            continue;
        }
        return Some(trimmed.to_string());
    }
}

fn run_cui(locale: UiLocale, menu_mode: bool) {
    let mut events = EventBus::default();
    let mut line_editor = LineEditor::new();
    let mut recorded_actions: Vec<SavedAction> = Vec::new();
    println!("{}: {}", locale.text("locale", "语言"), locale.code());
    let (mut run, mod_ids, warnings, mut content_signature) = build_run(locale).expect("load run");
    if !mod_ids.is_empty() {
        println!(
            "{}: {}",
            locale.text("mods loaded", "已加载模组"),
            mod_ids.len()
        );
        for mod_id in &mod_ids {
            println!("{}: {}", locale.text("mod", "模组"), mod_id);
        }
    }
    for warning in &warnings {
        eprintln!("{}: {}", locale.text("mod warning", "模组警告"), warning);
    }
    run.start_blind(1, BlindKind::Small, &mut events)
        .expect("start blind");

    let mut open_pack: Option<PackOpen> = None;
    if menu_mode {
        println!(
            "{}",
            locale.text(
                "menu mode enabled: choose numbered actions or type commands directly",
                "菜单模式已启用：可输入编号操作，或直接输入命令",
            )
        );
    }
    print_help(locale);
    print_action_guide(locale, &run, open_pack.as_ref());
    loop {
        let mut show_flow = false;
        if let Some(outcome) = run.blind_outcome() {
            println!(
                "{}: {}",
                locale.text("blind outcome", "盲注结果"),
                match outcome {
                    BlindOutcome::Cleared => locale.text("cleared", "通过"),
                    BlindOutcome::Failed => locale.text("failed", "失败"),
                }
            );
        }
        let line = match read_next_command(
            locale,
            &mut line_editor,
            &run,
            open_pack.as_ref(),
            menu_mode,
        ) {
            Some(line) => line,
            None => break,
        };
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        let mut parts = input.split_whitespace();
        let cmd = parts.next().unwrap_or("");
        let args: Vec<&str> = parts.collect();
        match cmd {
            "help" | "h" | "?" => print_help(locale),
            "quit" | "exit" => break,
            "actions" | "a" => print_action_guide(locale, &run, open_pack.as_ref()),
            "state" | "s" => print_state(locale, &run),
            "status" => print_summary(locale, &run),
            "hand" => print_hand(locale, &run),
            "deck" => print_deck(locale, &run),
            "levels" => print_levels(locale, &run),
            "tags" => print_tags(locale, &run),
            "inv" | "inventory" => print_inventory(locale, &run),
            "reward" => print_reward(locale, &run),
            "summary" => print_summary(locale, &run),
            "board" | "overview" | "ls" => print_overview(locale, &run, open_pack.as_ref()),
            "data" | "ref" => print_reference(locale),
            "save" => {
                let Some(path) = parse_optional_path(&args) else {
                    println!(
                        "{}",
                        locale.text("save path unavailable", "无法确定保存路径")
                    );
                    continue;
                };
                match save_state_file(
                    locale,
                    run.rng.seed(),
                    &content_signature,
                    &recorded_actions,
                    &path,
                ) {
                    Ok(_) => println!(
                        "{}: {} ({}, seed={})",
                        locale.text("saved state", "已保存状态"),
                        path.display(),
                        if matches!(locale, UiLocale::ZhCn) {
                            format!("{} 个动作", recorded_actions.len())
                        } else {
                            format!("{} actions", recorded_actions.len())
                        },
                        run.rng.seed()
                    ),
                    Err(err) => println!(
                        "{}: {}",
                        locale.text("error", "错误"),
                        localize_parse_error(locale, &err)
                    ),
                }
            }
            "load" => {
                let Some(path) = parse_optional_path(&args) else {
                    println!(
                        "{}",
                        locale.text("save path unavailable", "无法确定保存路径")
                    );
                    continue;
                };
                let saved = match load_state_file(&path) {
                    Ok(saved) => saved,
                    Err(err) => {
                        println!(
                            "{}: {}",
                            locale.text("error", "错误"),
                            localize_parse_error(locale, &err)
                        );
                        continue;
                    }
                };
                let (mut restored_run, _mod_ids, _warnings, restored_signature) =
                    match build_run_with_seed(locale, saved.seed) {
                        Ok(data) => data,
                        Err(err) => {
                            println!(
                                "{}: {}",
                                locale.text("error", "错误"),
                                localize_parse_error(locale, &err)
                            );
                            continue;
                        }
                    };
                if !saved.content_signature.is_empty()
                    && saved.content_signature != restored_signature
                {
                    println!(
                        "{}: {}",
                        locale.text("error", "错误"),
                        localize_parse_error(
                            locale,
                            &format!(
                                "content signature mismatch: saved={} current={}",
                                saved.content_signature, restored_signature
                            ),
                        )
                    );
                    continue;
                }
                let mut restored_events = EventBus::default();
                if let Err(err) =
                    restored_run.start_blind(1, BlindKind::Small, &mut restored_events)
                {
                    println!(
                        "{}: {}",
                        locale.text("error", "错误"),
                        localize_parse_error(locale, &err.to_string())
                    );
                    continue;
                }
                let mut restored_open_pack: Option<PackOpen> = None;
                let mut replay_failed = None;
                for action in &saved.actions {
                    if let Err(err) = apply_saved_action(
                        &mut restored_run,
                        &mut restored_events,
                        &mut restored_open_pack,
                        action,
                    ) {
                        replay_failed = Some(err);
                        break;
                    }
                }
                if let Some(err) = replay_failed {
                    println!(
                        "{}: {}",
                        locale.text("error", "错误"),
                        localize_parse_error(locale, &err)
                    );
                    continue;
                }
                run = restored_run;
                events = restored_events;
                open_pack = restored_open_pack;
                recorded_actions = saved.actions;
                content_signature = restored_signature;
                println!(
                    "{}: {} ({}, seed={})",
                    locale.text("loaded state", "已加载状态"),
                    path.display(),
                    if matches!(locale, UiLocale::ZhCn) {
                        format!("{} 个动作", recorded_actions.len())
                    } else {
                        format!("{} actions", recorded_actions.len())
                    },
                    saved.seed
                );
                drain_events(locale, &mut events);
                print_flow_summary(locale, &run, open_pack.as_ref());
            }
            "deal" | "d" => {
                show_flow = true;
                match run.prepare_hand(&mut events) {
                    Ok(_) => {
                        println!("{}", locale.text("dealt hand", "已发牌"));
                        push_recorded_action(&mut recorded_actions, "deal", Vec::new(), None);
                    }
                    Err(err) => print_run_error(locale, &run, open_pack.as_ref(), &err),
                }
            }
            "play" | "p" => {
                show_flow = true;
                match parse_indices_result(&args) {
                    Ok(indices) => {
                        println!(
                            "{}: {:?}",
                            locale.text("selected indices", "已选择索引"),
                            indices
                        );
                        let preview = collect_played_cards(&run.hand, &indices).ok();
                        match run.play_hand(&indices, &mut events) {
                            Ok(breakdown) => {
                                print_score_breakdown(
                                    locale,
                                    &breakdown,
                                    preview.as_deref(),
                                    &run.tables,
                                    &run.last_score_trace,
                                );
                                push_recorded_action(
                                    &mut recorded_actions,
                                    "play",
                                    indices.clone(),
                                    None,
                                );
                            }
                            Err(err) => print_run_error(locale, &run, open_pack.as_ref(), &err),
                        }
                    }
                    Err(err) => println!(
                        "{}: {} ({})",
                        locale.text("error", "错误"),
                        localize_parse_error(locale, &err),
                        locale.text("usage: play <idx> <idx> ...", "用法：play <idx> <idx> ...")
                    ),
                }
            }
            "discard" | "x" => {
                show_flow = true;
                match parse_indices_result(&args) {
                    Ok(indices) => match run.discard(&indices, &mut events) {
                        Ok(_) => {
                            println!("{}", locale.text("discarded", "已弃牌"));
                            push_recorded_action(
                                &mut recorded_actions,
                                "discard",
                                indices.clone(),
                                None,
                            );
                        }
                        Err(err) => print_run_error(locale, &run, open_pack.as_ref(), &err),
                    },
                    Err(err) => println!(
                        "{}: {} ({})",
                        locale.text("error", "错误"),
                        localize_parse_error(locale, &err),
                        locale.text(
                            "usage: discard <idx> <idx> ...",
                            "用法：discard <idx> <idx> ..."
                        )
                    ),
                }
            }
            "skip" | "skip_blind" => {
                show_flow = true;
                match run.skip_blind(&mut events) {
                    Ok(_) => {
                        println!("{}", locale.text("blind skipped", "已跳过盲注"));
                        push_recorded_action(&mut recorded_actions, "skip_blind", Vec::new(), None);
                    }
                    Err(err) => print_run_error(locale, &run, open_pack.as_ref(), &err),
                }
            }
            "shop" | "sh" => {
                show_flow = true;
                match run.enter_shop(&mut events) {
                    Ok(_) => {
                        print_shop(locale, &run);
                        push_recorded_action(&mut recorded_actions, "enter_shop", Vec::new(), None);
                    }
                    Err(err) => print_run_error(locale, &run, open_pack.as_ref(), &err),
                }
            }
            "leave" => {
                show_flow = true;
                run.leave_shop();
                open_pack = None;
                println!("{}", locale.text("left shop", "已离开商店"));
                push_recorded_action(&mut recorded_actions, "leave_shop", Vec::new(), None);
            }
            "reroll" | "r" => {
                show_flow = true;
                match run.reroll_shop(&mut events) {
                    Ok(_) => {
                        print_shop(locale, &run);
                        push_recorded_action(&mut recorded_actions, "reroll", Vec::new(), None);
                    }
                    Err(err) => print_run_error(locale, &run, open_pack.as_ref(), &err),
                }
            }
            "buy" => {
                show_flow = true;
                if args.len() < 2 {
                    println!(
                        "{}",
                        locale.text(
                            "usage: buy card|pack|voucher <index>",
                            "用法：buy card|pack|voucher <index>"
                        )
                    );
                } else {
                    let kind = args[0];
                    let index = args[1].parse::<usize>().ok();
                    match (kind, index) {
                        ("card", Some(idx)) => {
                            match run.buy_shop_offer(ShopOfferRef::Card(idx), &mut events) {
                                Ok(purchase) => {
                                    if let Err(err) = run.apply_purchase(&purchase) {
                                        print_run_error(locale, &run, open_pack.as_ref(), &err);
                                    } else {
                                        println!(
                                            "{} {idx}",
                                            locale.text("bought card", "已购买卡牌")
                                        );
                                        push_recorded_action(
                                            &mut recorded_actions,
                                            "buy_card",
                                            Vec::new(),
                                            Some(idx.to_string()),
                                        );
                                    }
                                }
                                Err(err) => print_run_error(locale, &run, open_pack.as_ref(), &err),
                            }
                        }
                        ("pack", Some(idx)) => {
                            match run.buy_shop_offer(ShopOfferRef::Pack(idx), &mut events) {
                                Ok(purchase) => {
                                    match run.open_pack_purchase(&purchase, &mut events) {
                                        Ok(open) => {
                                            print_pack_open(locale, &open, &run);
                                            open_pack = Some(open);
                                            push_recorded_action(
                                                &mut recorded_actions,
                                                "buy_pack",
                                                Vec::new(),
                                                Some(idx.to_string()),
                                            );
                                        }
                                        Err(err) => {
                                            print_run_error(locale, &run, open_pack.as_ref(), &err)
                                        }
                                    }
                                }
                                Err(err) => print_run_error(locale, &run, open_pack.as_ref(), &err),
                            }
                        }
                        ("voucher", Some(idx)) => {
                            match run.buy_shop_offer(ShopOfferRef::Voucher(idx), &mut events) {
                                Ok(purchase) => {
                                    if let Err(err) = run.apply_purchase(&purchase) {
                                        print_run_error(locale, &run, open_pack.as_ref(), &err);
                                    } else {
                                        println!(
                                            "{} {idx}",
                                            locale.text("bought voucher", "已购买优惠券")
                                        );
                                        push_recorded_action(
                                            &mut recorded_actions,
                                            "buy_voucher",
                                            Vec::new(),
                                            Some(idx.to_string()),
                                        );
                                    }
                                }
                                Err(err) => print_run_error(locale, &run, open_pack.as_ref(), &err),
                            }
                        }
                        _ => println!(
                            "{}",
                            locale.text(
                                "usage: buy card|pack|voucher <index>",
                                "用法：buy card|pack|voucher <index>"
                            )
                        ),
                    }
                }
            }
            "pack" => {
                if let Some(open) = open_pack.as_ref() {
                    print_pack_open(locale, open, &run);
                } else {
                    println!("{}", locale.text("no open pack", "当前没有打开的卡包"));
                }
            }
            "edit" => {
                if args.is_empty() {
                    println!(
                        "{}",
                        locale.text(
                            "usage: edit <idx..> enh=<kind|none> ed=<kind|none> seal=<kind|none> bonus=<n|+n|-n> face_down=<0|1>",
                            "用法：edit <idx..> enh=<kind|none> ed=<kind|none> seal=<kind|none> bonus=<n|+n|-n> face_down=<0|1>"
                        )
                    );
                    continue;
                }
                match parse_edit_args(&args) {
                    Ok((indices, edits)) => {
                        match apply_card_edits(&mut run.hand, &indices, edits) {
                            Ok(_) => println!(
                                "{}: {:?}",
                                locale.text("edited cards", "已编辑牌"),
                                indices
                            ),
                            Err(err) => println!(
                                "{}: {}",
                                locale.text("error", "错误"),
                                localize_parse_error(locale, &err)
                            ),
                        }
                    }
                    Err(err) => println!(
                        "{}: {}",
                        locale.text("error", "错误"),
                        localize_parse_error(locale, &err)
                    ),
                }
            }
            "pick" => {
                show_flow = true;
                if let Some(open) = open_pack.clone() {
                    match parse_indices_result(&args) {
                        Ok(indices) => {
                            match run.choose_pack_options(&open, &indices, &mut events) {
                                Ok(_) => {
                                    println!(
                                        "{}",
                                        locale.text("picked pack options", "已选择卡包选项")
                                    );
                                    open_pack = None;
                                    push_recorded_action(
                                        &mut recorded_actions,
                                        "pick_pack",
                                        indices.clone(),
                                        None,
                                    );
                                }
                                Err(err) => print_run_error(locale, &run, open_pack.as_ref(), &err),
                            }
                        }
                        Err(err) => println!(
                            "{}: {} ({})",
                            locale.text("error", "错误"),
                            localize_parse_error(locale, &err),
                            locale
                                .text("usage: pick <idx> <idx> ...", "用法：pick <idx> <idx> ...")
                        ),
                    }
                } else {
                    println!("{}", locale.text("no open pack", "当前没有打开的卡包"));
                }
            }
            "skip_pack" | "sp" => {
                show_flow = true;
                if let Some(open) = open_pack.clone() {
                    match run.skip_pack(&open, &mut events) {
                        Ok(_) => {
                            println!("{}", locale.text("skipped pack", "已跳过卡包"));
                            open_pack = None;
                            push_recorded_action(
                                &mut recorded_actions,
                                "skip_pack",
                                Vec::new(),
                                None,
                            );
                        }
                        Err(err) => print_run_error(locale, &run, open_pack.as_ref(), &err),
                    }
                } else {
                    println!("{}", locale.text("no open pack", "当前没有打开的卡包"));
                }
            }
            "peek" => {
                if args.is_empty() {
                    println!(
                        "{}",
                        locale.text(
                            "usage: peek draw|discard [count]",
                            "用法：peek draw|discard [count]"
                        )
                    );
                } else {
                    let target = args[0];
                    let count = args
                        .get(1)
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or(5);
                    match target {
                        "draw" => print_peek(locale, &run.deck.draw, count, "draw"),
                        "discard" => print_peek(locale, &run.deck.discard, count, "discard"),
                        _ => println!(
                            "{}",
                            locale.text(
                                "usage: peek draw|discard [count]",
                                "用法：peek draw|discard [count]"
                            )
                        ),
                    }
                }
            }
            "use" => {
                show_flow = true;
                if args.is_empty() {
                    println!(
                        "{}",
                        locale.text(
                            "usage: use <consumable_index> [selected idxs]",
                            "用法：use <consumable_index> [selected idxs]"
                        )
                    );
                    continue;
                }
                let idx = match args[0].parse::<usize>() {
                    Ok(idx) => idx,
                    Err(_) => {
                        println!("{}", locale.text("invalid index", "无效索引"));
                        continue;
                    }
                };
                let selected = match parse_optional_indices(&args[1..]) {
                    Ok(selected) => selected,
                    Err(err) => {
                        println!(
                            "{}: {} ({})",
                            locale.text("error", "错误"),
                            localize_parse_error(locale, &err),
                            locale.text(
                                "usage: use <consumable_index> [selected idxs]",
                                "用法：use <consumable_index> [selected idxs]"
                            )
                        );
                        continue;
                    }
                };
                match run.use_consumable(idx, &selected, &mut events) {
                    Ok(_) => {
                        println!("{}", locale.text("consumable used", "已使用消耗牌"));
                        push_recorded_action(
                            &mut recorded_actions,
                            "use_consumable",
                            selected.clone(),
                            Some(idx.to_string()),
                        );
                    }
                    Err(err) => print_run_error(locale, &run, open_pack.as_ref(), &err),
                }
            }
            "sell" => {
                show_flow = true;
                if args.len() != 1 {
                    println!(
                        "{}",
                        locale.text("usage: sell <joker_index>", "用法：sell <joker_index>")
                    );
                    continue;
                }
                match args[0].parse::<usize>() {
                    Ok(idx) => match run.sell_joker(idx, &mut events) {
                        Ok(_) => {
                            println!("{} {idx}", locale.text("sold joker", "已出售小丑"));
                            push_recorded_action(
                                &mut recorded_actions,
                                "sell_joker",
                                Vec::new(),
                                Some(idx.to_string()),
                            );
                        }
                        Err(err) => print_run_error(locale, &run, open_pack.as_ref(), &err),
                    },
                    Err(_) => println!("{}", locale.text("invalid index", "无效索引")),
                }
            }
            "next" | "n" => {
                show_flow = true;
                open_pack = None;
                match run.start_next_blind(&mut events) {
                    Ok(_) => {
                        println!("{}", locale.text("started next blind", "已开始下一盲注"));
                        push_recorded_action(&mut recorded_actions, "next_blind", Vec::new(), None);
                    }
                    Err(err) => print_run_error(locale, &run, open_pack.as_ref(), &err),
                }
            }
            _ => println!(
                "{}: {cmd} ({})",
                locale.text("unknown command", "未知命令"),
                locale.text("type 'help'", "输入 'help' 查看帮助")
            ),
        }
        drain_events(locale, &mut events);
        if show_flow {
            print_flow_summary(locale, &run, open_pack.as_ref());
        }
    }
    line_editor.save_history();
}

fn print_help(locale: UiLocale) {
    println!("{}", locale.text("Commands:", "命令："));
    println!(
        "  help|h|?                 {}",
        locale.text("show help", "显示帮助")
    );
    println!(
        "  actions|a                {}",
        locale.text("show context-aware next actions", "显示当前推荐操作")
    );
    println!(
        "  save [path]              {}",
        locale.text("save run state to local file", "保存运行状态到本地文件")
    );
    println!(
        "  load [path]              {}",
        locale.text("load run state from local file", "从本地文件加载运行状态")
    );
    println!("  quit|exit                {}", locale.text("exit", "退出"));
    println!();
    println!("{}", locale.text("View:", "查看："));
    println!(
        "  summary|status           {}",
        locale.text("one-line run status", "单行状态")
    );
    println!(
        "  state|s                  {}",
        locale.text("detailed run state", "详细状态")
    );
    println!(
        "  board|overview|ls        {}",
        locale.text(
            "full current view (state+hand+inv+shop+pack)",
            "完整视图（状态+手牌+背包+商店+卡包）"
        )
    );
    println!(
        "  hand                      {}",
        locale.text("show hand table", "显示手牌")
    );
    println!(
        "  deck                      {}",
        locale.text("show draw/discard sizes", "显示抽牌/弃牌堆")
    );
    println!(
        "  levels                    {}",
        locale.text("show hand levels", "显示牌型等级")
    );
    println!(
        "  tags                      {}",
        locale.text("show tags", "显示标签")
    );
    println!(
        "  inv|inventory             {}",
        locale.text("show jokers and consumables", "显示小丑与消耗牌")
    );
    println!(
        "  reward                    {}",
        locale.text("estimate clear reward", "估算通关奖励")
    );
    println!(
        "  data|ref                  {}",
        locale.text(
            "print enhancement/joker/consumable reference",
            "显示效果参考"
        )
    );
    println!();
    println!("{}", locale.text("Run:", "流程："));
    println!(
        "  deal|d                    {}",
        locale.text("draw hand (Deal phase)", "发牌（Deal 阶段）")
    );
    println!(
        "  play|p <idx..>            {}",
        locale.text("play cards", "出牌")
    );
    println!(
        "  discard|x <idx..>         {}",
        locale.text("discard cards", "弃牌")
    );
    println!(
        "  skip|skip_blind           {}",
        locale.text(
            "skip current blind (Small/Big only)",
            "跳过当前盲注（仅小/大盲）"
        )
    );
    println!(
        "  next|n                    {}",
        locale.text("start next blind", "进入下一个盲注")
    );
    println!();
    println!("{}", locale.text("Shop / Pack:", "商店 / 卡包："));
    println!(
        "  shop|sh                   {}",
        locale.text("enter shop", "进入商店")
    );
    println!(
        "  reroll|r                  {}",
        locale.text("reroll shop", "刷新商店")
    );
    println!(
        "  buy card|pack|voucher <idx>  {}",
        locale.text("buy from shop", "购买商店商品")
    );
    println!(
        "  leave                     {}",
        locale.text("leave shop", "离开商店")
    );
    println!(
        "  pack                      {}",
        locale.text("show open pack options", "显示当前卡包选项")
    );
    println!(
        "  pick <idx..>              {}",
        locale.text("pick pack options", "选择卡包选项")
    );
    println!(
        "  skip_pack|sp              {}",
        locale.text("skip open pack", "跳过当前卡包")
    );
    println!();
    println!("{}", locale.text("Debug / Edit:", "调试 / 编辑："));
    println!(
        "  use <consumable_idx> [sel..]  {}",
        locale.text("use a consumable", "使用消耗牌")
    );
    println!(
        "  sell <joker_idx>          {}",
        locale.text("sell joker", "出售小丑")
    );
    println!(
        "  edit <idx..> enh=.. ed=.. seal=.. bonus=.. face_down=..  {}",
        locale.text("edit cards in hand", "编辑手牌属性")
    );
    println!(
        "  peek draw|discard [n]     {}",
        locale.text("peek deck top cards", "查看牌堆顶部")
    );
    println!(
        "{}",
        locale.text(
            "note: indices support comma and ranges (e.g. 0,2-4 7)",
            "说明：索引支持逗号和区间（如 0,2-4 7）"
        )
    );
    println!(
        "{}",
        locale.text(
            "tip: actions print a flow summary automatically",
            "提示：操作后会自动显示流程摘要"
        )
    );
    println!(
        "{}",
        locale.text(
            "tip: use Up/Down for command history, Tab for command completion",
            "提示：可使用 上/下 键浏览历史命令，Tab 自动补全命令",
        )
    );
    println!(
        "{}",
        locale.text(
            "tip: run with --auto for scripted demo",
            "提示：可使用 --auto 运行自动演示"
        )
    );
    println!(
        "{}",
        locale.text(
            "tip: set language with --lang zh_CN (or env RULATRO_LANG=zh_CN)",
            "提示：可用 --lang zh_CN（或环境变量 RULATRO_LANG=zh_CN）切换中文"
        )
    );
    println!(
        "{}",
        locale.text(
            "tip: save/load default path can be overridden via RULATRO_SAVE",
            "提示：save/load 默认路径可通过环境变量 RULATRO_SAVE 覆盖"
        )
    );
}

fn print_reference(locale: UiLocale) {
    println!("{}", locale.text("== Reference ==", "== 参考 ==",));
    println!("{}", locale.text("Enhancements:", "增强效果："));
    println!(
        "  {}",
        locale.text("Bonus +30 chips (scored)", "Bonus：+30 筹码（计分时）")
    );
    println!(
        "  {}",
        locale.text("Mult +4 mult (scored)", "Mult：+4 倍率（计分时）")
    );
    println!(
        "  {}",
        locale.text(
            "Glass x2 mult (scored), 1/4 break",
            "Glass：x2 倍率（计分时），1/4 概率破碎"
        )
    );
    println!(
        "  {}",
        locale.text(
            "Stone +50 chips (scored), no rank/suit",
            "Stone：+50 筹码（计分时），无点数/花色"
        )
    );
    println!(
        "  {}",
        locale.text(
            "Lucky 1/5 +20 mult, 1/15 +$20",
            "Lucky：1/5 +20 倍率，1/15 +$20"
        )
    );
    println!(
        "  {}",
        locale.text("Steel x1.5 mult (held)", "Steel：x1.5 倍率（留手）")
    );
    println!(
        "  {}",
        locale.text("Gold +$3 end of round (held)", "Gold：回合结束 +$3（留手）")
    );
    println!(
        "  {}",
        locale.text("Wild counts as any suit", "Wild：可视为任意花色")
    );
    println!(
        "{}",
        locale.text(
            "Seals: Red retrigger; Gold +$3 scored; Blue planet on round end; Purple tarot on discard",
            "蜡封：Red 重触发；Gold 计分 +$3；Blue 回合结束给星球；Purple 弃牌给塔罗",
        )
    );
    println!(
        "{}",
        locale.text(
            "Editions: Foil +50 chips; Holo +10 mult; Polychrome x1.5 mult; Negative +1 joker slot",
            "版本：Foil +50 筹码；Holo +10 倍率；Polychrome x1.5 倍率；Negative +1 小丑槽",
        )
    );
    println!();
    println!(
        "{}",
        locale.text(
            "Joker DSL triggers (on ...): played, scored_pre, scored, held, independent,",
            "Joker DSL 触发器（on ...）：played, scored_pre, scored, held, independent,",
        )
    );
    println!("  discard, discard_batch, card_destroyed, card_added, round_end, hand_end,");
    println!("  blind_start, blind_failed, shop_enter, shop_reroll, shop_exit,");
    println!("  pack_opened, pack_skipped, use, sell, any_sell, acquire, passive");
    println!(
        "{}",
        locale.text("Common DSL condition identifiers:", "常用 DSL 条件标识符：")
    );
    println!("  hand, blind, ante, blind_score, target, money, hands_left, discards_left,");
    println!("  played_count, scoring_count, held_count, deck_count,");
    println!("  card.rank, card.suit, card.enhancement, card.edition, card.seal,");
    println!("  card.is_face/odd/even/stone/wild, consumable.kind/id");
    println!(
        "{}",
        locale.text("Common DSL functions:", "常用 DSL 函数：")
    );
    println!("  contains(hand, HandKind), count(scope,target), count_joker(name/id),");
    println!("  count_rarity(rarity), suit_match(suit|id), hand_count(hand), var(key),");
    println!("  roll(n), rand(min,max), min/max/floor/ceil/pow");
    println!();
    println!("{}", locale.text("Consumable effects:", "消耗牌效果："));
    println!("  EnhanceSelected/AddEditionToSelected/AddSealToSelected");
    println!("  ConvertSelectedSuit/IncreaseSelectedRank/DestroySelected/CopySelected");
    println!("  AddRandomConsumable/AddJoker/AddRandomJoker/UpgradeHand/UpgradeAllHands");
    println!("  AddMoney/SetMoney/DoubleMoney/AddMoneyFromJokers");
    println!(
        "{}",
        locale.text(
            "Selection rules: selection required for *Selected/*LeftIntoRight ops;",
            "选择规则：*Selected/*LeftIntoRight 操作需要提供选择索引；"
        )
    );
    println!(
        "{}",
        locale.text("indices refer to current hand.", "索引均基于当前手牌。")
    );
}

fn print_action_guide(locale: UiLocale, run: &RunState, open_pack: Option<&PackOpen>) {
    println!("{}", locale.text("next actions:", "下一步建议："));
    if open_pack.is_some() {
        println!(
            "{}",
            locale.text(
                "  pack open: pick <idx..> | skip_pack | pack",
                "  卡包已打开：pick <idx..> | skip_pack | pack"
            )
        );
        println!(
            "  {}",
            locale.text("info: board | hand | inv", "查看：board | hand | inv")
        );
        return;
    }
    if let Some(outcome) = run.blind_outcome() {
        match outcome {
            BlindOutcome::Cleared => {
                if run.state.phase == Phase::Shop {
                    println!(
                        "  {}",
                        locale.text(
                            "shop: buy card|pack|voucher <idx> | reroll | leave",
                            "商店：buy card|pack|voucher <idx> | reroll | leave"
                        )
                    );
                } else {
                    println!(
                        "  {}",
                        locale.text("blind cleared: shop | next", "已通过盲注：shop | next")
                    );
                }
            }
            BlindOutcome::Failed => {
                println!("  {}", locale.text("blind failed: next", "盲注失败：next"))
            }
        }
        println!(
            "  {}",
            locale.text(
                "info: board | hand | inv | reward",
                "查看：board | hand | inv | reward"
            )
        );
        return;
    }
    match run.state.phase {
        Phase::Deal => {
            println!("  {}", locale.text("deal phase: deal", "发牌阶段：deal"));
            if run.state.blind != BlindKind::Boss {
                println!(
                    "  {}",
                    locale.text("optional: skip (to take a tag)", "可选：skip（获得标签）")
                );
            }
        }
        Phase::Play => println!(
            "  {}",
            locale.text(
                "play phase: play <idx..> | discard <idx..>",
                "出牌阶段：play <idx..> | discard <idx..>"
            )
        ),
        Phase::Shop => println!(
            "  {}",
            locale.text(
                "shop: buy card|pack|voucher <idx> | reroll | leave",
                "商店：buy card|pack|voucher <idx> | reroll | leave"
            )
        ),
        Phase::Setup => println!("  {}", locale.text("setup: next", "准备阶段：next")),
        Phase::Score | Phase::Cleanup => println!(
            "  {}",
            locale.text("transition: board | summary", "过渡阶段：board | summary")
        ),
    }
    println!(
        "  {}",
        locale.text(
            "info: board | hand | inv | levels | tags | reward",
            "查看：board | hand | inv | levels | tags | reward"
        )
    );
}

fn print_run_error(locale: UiLocale, run: &RunState, open_pack: Option<&PackOpen>, err: &RunError) {
    println!("{}: {err}", locale.text("error", "错误"));
    if let Some(hint) = run_error_hint(locale, run, open_pack, err) {
        println!("{}: {hint}", locale.text("hint", "提示"));
    }
}

fn run_error_hint(
    locale: UiLocale,
    run: &RunState,
    open_pack: Option<&PackOpen>,
    err: &RunError,
) -> Option<String> {
    if open_pack.is_some() {
        return Some(
            locale
                .text(
                    "pack is open: use 'pick <idx..>' or 'skip_pack' first",
                    "卡包已打开：请先使用 'pick <idx..>' 或 'skip_pack'",
                )
                .to_string(),
        );
    }
    match err {
        RunError::InvalidPhase(phase) => match phase {
            Phase::Deal => Some(
                locale
                    .text(
                        "use 'deal' first, then 'play' or 'discard'",
                        "请先执行 'deal'，然后再 'play' 或 'discard'",
                    )
                    .to_string(),
            ),
            Phase::Play => Some(
                locale
                    .text(
                        "use 'play <idx..>' or 'discard <idx..>'",
                        "请使用 'play <idx..>' 或 'discard <idx..>'",
                    )
                    .to_string(),
            ),
            Phase::Shop => Some(
                locale
                    .text(
                        "use shop commands: 'buy', 'reroll', or 'leave'",
                        "请使用商店命令：'buy'、'reroll' 或 'leave'",
                    )
                    .to_string(),
            ),
            Phase::Cleanup => {
                if run.blind_cleared() {
                    Some(
                        locale
                            .text(
                                "blind is cleared: use 'shop' or 'next'",
                                "盲注已通过：使用 'shop' 或 'next'",
                            )
                            .to_string(),
                    )
                } else {
                    Some(
                        locale
                            .text(
                                "round ended: use 'deal' for next hand",
                                "本手已结束：使用 'deal' 继续下一手",
                            )
                            .to_string(),
                    )
                }
            }
            Phase::Setup => Some(
                locale
                    .text("start with 'next'", "请从 'next' 开始")
                    .to_string(),
            ),
            Phase::Score => Some(
                locale
                    .text(
                        "scoring is resolving; check 'summary' then continue",
                        "结算进行中：先查看 'summary' 再继续",
                    )
                    .to_string(),
            ),
        },
        RunError::NoHandsLeft => {
            if run.blind_cleared() {
                Some(
                    locale
                        .text(
                            "blind is already cleared: use 'shop' or 'next'",
                            "盲注已通过：使用 'shop' 或 'next'",
                        )
                        .to_string(),
                )
            } else {
                Some(
                    locale
                        .text(
                            "no hands left; use 'next' to move on",
                            "出牌次数已用尽：使用 'next' 继续",
                        )
                        .to_string(),
                )
            }
        }
        RunError::NoDiscardsLeft => Some(
            locale
                .text(
                    "no discards left; use 'play <idx..>'",
                    "弃牌次数已用尽：请使用 'play <idx..>'",
                )
                .to_string(),
        ),
        RunError::BlindNotCleared => {
            let remaining = (run.state.target - run.state.blind_score).max(0);
            Some(if matches!(locale, UiLocale::ZhCn) {
                format!("进入商店前还需要 {remaining} 分")
            } else {
                format!("need {remaining} more score before entering shop")
            })
        }
        RunError::NotEnoughMoney => Some(
            locale
                .text(
                    "not enough money; check price and reroll cost",
                    "资金不足：请检查价格和刷新费用",
                )
                .to_string(),
        ),
        RunError::InvalidSelection => Some(
            locale
                .text(
                    "check indices with 'hand' or 'pack' and try again",
                    "请用 'hand' 或 'pack' 检查索引后重试",
                )
                .to_string(),
        ),
        RunError::InvalidCardCount => Some(
            locale
                .text("pick between 1 and 5 card indices", "请选择 1 到 5 张牌")
                .to_string(),
        ),
        RunError::PackNotAvailable => Some(
            locale
                .text(
                    "buy a pack in shop first: 'buy pack <idx>'",
                    "请先在商店购买卡包：'buy pack <idx>'",
                )
                .to_string(),
        ),
        RunError::CannotSkipBoss => Some(
            locale
                .text("boss blind cannot be skipped", "Boss 盲注不能跳过")
                .to_string(),
        ),
        RunError::InvalidOfferIndex => Some(
            locale
                .text(
                    "invalid shop index; use 'shop' to inspect current offers",
                    "无效的商店索引；请使用 'shop' 查看当前商品",
                )
                .to_string(),
        ),
        RunError::InvalidJokerIndex => Some(
            locale
                .text(
                    "invalid joker index; use 'inv' to list jokers",
                    "无效的小丑索引；请使用 'inv' 查看",
                )
                .to_string(),
        ),
        _ => None,
    }
}

fn prompt_text(locale: UiLocale, run: &RunState, open_pack: Option<&PackOpen>) -> String {
    let pack = if open_pack.is_some() { " PK" } else { "" };
    format!(
        "[A{} {} {} ${} {}/{} H{}/{} D{}/{} SK{}{}] > ",
        run.state.ante,
        blind_label(locale, run.state.blind),
        phase_label(locale, run.state.phase),
        run.state.money,
        run.state.blind_score,
        run.state.target,
        run.state.hands_left,
        run.state.hands_max,
        run.state.discards_left,
        run.state.discards_max,
        run.state.blinds_skipped,
        pack
    )
}

fn print_state(locale: UiLocale, run: &RunState) {
    println!("{}", locale.text("== State ==", "== 状态 =="));
    println!(
        "{} {} | {} {} | {} {}",
        locale.text("Ante", "底注"),
        run.state.ante,
        locale.text("Blind", "盲注"),
        blind_label(locale, run.state.blind),
        locale.text("Phase", "阶段"),
        phase_label(locale, run.state.phase)
    );
    println!(
        "{} {}/{} | {} {}/{} | {} {}",
        locale.text("Score", "分数"),
        run.state.blind_score,
        run.state.target,
        locale.text("Hands", "出牌次数"),
        run.state.hands_left,
        run.state.hands_max,
        locale.text("Discards", "弃牌次数"),
        format!("{}/{}", run.state.discards_left, run.state.discards_max)
    );
    println!(
        "{} ${} | {} {}/{} | {} {}",
        locale.text("Money", "金钱"),
        run.state.money,
        locale.text("Hand Size", "手牌上限"),
        run.state.hand_size,
        run.state.hand_size_base,
        locale.text("Skipped", "已跳过"),
        run.state.blinds_skipped
    );
    println!(
        "{} {} | {} {}",
        locale.text("Draw Pile", "抽牌堆"),
        run.deck.draw.len(),
        locale.text("Discard Pile", "弃牌堆"),
        run.deck.discard.len()
    );
}

fn print_levels(locale: UiLocale, run: &RunState) {
    println!("{}", locale.text("== Hand Levels ==", "== 牌型等级 =="));
    for kind in rulatro_core::HandKind::ALL {
        let level = run.state.hand_levels.get(&kind).copied().unwrap_or(1);
        println!("  {:<18} {}", format!("{kind:?}"), level);
    }
}

fn print_tags(locale: UiLocale, run: &RunState) {
    if run.state.tags.is_empty() {
        println!("{}", locale.text("tags: none", "标签：无"));
    } else {
        let labels: Vec<String> = run
            .state
            .tags
            .iter()
            .map(|id| {
                let name = run
                    .content
                    .tag_by_id(id)
                    .map(|tag| tag.name.clone())
                    .unwrap_or_else(|| id.clone());
                format!("{id} ({name})")
            })
            .collect();
        println!("{} {}", locale.text("tags:", "标签："), labels.join(", "));
    }
    if run.state.duplicate_next_tag {
        if let Some(exclude) = &run.state.duplicate_tag_exclude {
            if matches!(locale, UiLocale::ZhCn) {
                println!("下一个标签复制（排除 {exclude}）");
            } else {
                println!("duplicate next tag (excluding {exclude})");
            }
        } else {
            println!("{}", locale.text("duplicate next tag", "下一个标签将复制"));
        }
    }
}

fn print_reward(locale: UiLocale, run: &RunState) {
    if run.state.target <= 0 {
        println!(
            "{}",
            locale.text("reward: blind not started", "奖励：盲注尚未开始")
        );
        return;
    }
    let economy = &run.config.economy;
    let base = match run.state.blind {
        BlindKind::Small => economy.reward_small,
        BlindKind::Big => economy.reward_big,
        BlindKind::Boss => economy.reward_boss,
    };
    let interest = estimate_interest(run);
    let reward = base + economy.per_hand_reward * run.state.hands_left as i64 + interest;
    println!(
        "{} {} ({} {} + {} {} + {} {})",
        locale.text("reward estimate:", "奖励预估："),
        reward,
        locale.text("base", "基础"),
        base,
        locale.text("hand bonus", "手牌奖励"),
        economy.per_hand_reward * run.state.hands_left as i64,
        locale.text("interest", "利息"),
        interest
    );
}

fn print_summary(locale: UiLocale, run: &RunState) {
    println!(
        "{} {} {} {} | ${} | {}/{} | {} {}/{} | {} {}/{} | {} {}",
        locale.text("A", "A"),
        run.state.ante,
        blind_label(locale, run.state.blind),
        phase_label(locale, run.state.phase),
        run.state.money,
        run.state.blind_score,
        run.state.target,
        locale.text("hands", "出牌"),
        run.state.hands_left,
        run.state.hands_max,
        locale.text("discards", "弃牌"),
        run.state.discards_left,
        run.state.discards_max,
        locale.text("skipped", "跳过"),
        run.state.blinds_skipped
    );
}

fn print_flow_summary(locale: UiLocale, run: &RunState, open_pack: Option<&PackOpen>) {
    let pack = if open_pack.is_some() {
        locale.text(" | pack open", " | 卡包已打开")
    } else {
        ""
    };
    println!(
        "=> {}{} {} {} | ${} | {}/{} | {} {}/{} {} {}/{} | {} {}{}",
        locale.text("A", "A"),
        run.state.ante,
        blind_label(locale, run.state.blind),
        phase_label(locale, run.state.phase),
        run.state.money,
        run.state.blind_score,
        run.state.target,
        locale.text("hands", "出牌"),
        run.state.hands_left,
        run.state.hands_max,
        locale.text("discards", "弃牌"),
        run.state.discards_left,
        run.state.discards_max,
        locale.text("skipped", "跳过"),
        run.state.blinds_skipped,
        pack
    );
    println!(
        "   {}: {}",
        locale.text("next", "下一步"),
        next_step_hint(locale, run, open_pack)
    );
}

fn print_overview(locale: UiLocale, run: &RunState, open_pack: Option<&PackOpen>) {
    print_summary(locale, run);
    print_tags(locale, run);
    print_hand(locale, run);
    print_inventory(locale, run);
    if run.shop.is_some() {
        print_shop(locale, run);
    }
    if let Some(open) = open_pack {
        print_pack_open(locale, open, run);
    }
}

fn next_step_hint(locale: UiLocale, run: &RunState, open_pack: Option<&PackOpen>) -> String {
    if open_pack.is_some() {
        return locale
            .text("pick <idx..> or skip_pack", "pick <idx..> 或 skip_pack")
            .to_string();
    }
    if let Some(outcome) = run.blind_outcome() {
        return match outcome {
            BlindOutcome::Cleared => {
                if run.state.phase == Phase::Shop {
                    locale
                        .text("buy/reroll/leave, then next", "buy/reroll/leave，然后 next")
                        .to_string()
                } else {
                    locale.text("shop or next", "shop 或 next").to_string()
                }
            }
            BlindOutcome::Failed => locale.text("next", "next").to_string(),
        };
    }
    match run.state.phase {
        Phase::Deal => {
            if run.state.blind == BlindKind::Boss {
                locale.text("deal", "deal").to_string()
            } else {
                locale.text("deal (or skip)", "deal（或 skip）").to_string()
            }
        }
        Phase::Play => locale
            .text(
                "play <idx..> or discard <idx..>",
                "play <idx..> 或 discard <idx..>",
            )
            .to_string(),
        Phase::Shop => locale
            .text("buy/reroll/leave", "buy/reroll/leave")
            .to_string(),
        Phase::Setup => locale.text("next", "next").to_string(),
        Phase::Score | Phase::Cleanup => locale.text("summary", "summary").to_string(),
    }
}

fn print_hand(locale: UiLocale, run: &RunState) {
    println!(
        "{} {}",
        locale.text("== Hand ==", "== 手牌 =="),
        format!("({} {})", run.hand.len(), locale.text("cards", "张"))
    );
    println!(
        "{:>4}  {:<14} {:>6}  {}",
        locale.text("idx", "序号"),
        locale.text("card", "卡牌"),
        locale.text("value", "点数"),
        locale.text("detail", "详情")
    );
    for (idx, card) in run.hand.iter().enumerate() {
        let value = card_value(card, &run.tables);
        println!(
            "{:>4}  {:<14} {:>6}  {}",
            idx,
            format_card(card),
            value,
            card_detail(card)
        );
    }
}

fn print_deck(locale: UiLocale, run: &RunState) {
    println!(
        "{} {}",
        locale.text("draw pile:", "抽牌堆："),
        run.deck.draw.len()
    );
    println!(
        "{} {}",
        locale.text("discard pile:", "弃牌堆："),
        run.deck.discard.len()
    );
}

fn print_inventory(locale: UiLocale, run: &RunState) {
    println!(
        "{} ({}/{}):",
        locale.text("Jokers", "小丑"),
        run.inventory.jokers.len(),
        run.inventory.joker_capacity()
    );
    for (idx, joker) in run.inventory.jokers.iter().enumerate() {
        let edition = joker.edition.map(edition_short).unwrap_or("");
        let suffix = if edition.is_empty() {
            "".to_string()
        } else {
            format!(" [{edition}]")
        };
        let name = find_joker_name(run, &joker.id);
        println!(
            "{:>2}: {} ({}){} ({:?})",
            idx, joker.id, name, suffix, joker.rarity
        );
    }
    println!(
        "{} ({}/{}):",
        locale.text("Consumables", "消耗牌"),
        run.inventory.consumable_count(),
        run.inventory.consumable_slots
    );
    for (idx, item) in run.inventory.consumables.iter().enumerate() {
        let edition = item.edition.map(edition_short).unwrap_or("");
        let suffix = if edition.is_empty() {
            "".to_string()
        } else {
            format!(" [{edition}]")
        };
        let name = find_consumable_name(run, item.kind, &item.id);
        println!(
            "{:>2}: {} ({}) {:?}{}",
            idx, item.id, name, item.kind, suffix
        );
    }
}

fn print_shop(locale: UiLocale, run: &RunState) {
    let Some(shop) = &run.shop else {
        println!("{}", locale.text("shop not available", "商店不可用"));
        return;
    };
    println!(
        "{}: {} {} {} {} {} {} {} {}",
        locale.text("shop", "商店"),
        locale.text("cards", "卡牌"),
        shop.cards.len(),
        locale.text("packs", "卡包"),
        shop.packs.len(),
        locale.text("vouchers", "优惠券"),
        shop.vouchers,
        locale.text("reroll", "刷新"),
        shop.reroll_cost
    );
    println!("{}:", locale.text("cards", "卡牌"));
    for (idx, card) in shop.cards.iter().enumerate() {
        let item_name = match card.kind {
            rulatro_core::ShopCardKind::Joker => find_joker_name(run, &card.item_id),
            rulatro_core::ShopCardKind::Tarot => {
                find_consumable_name(run, ConsumableKind::Tarot, &card.item_id)
            }
            rulatro_core::ShopCardKind::Planet => {
                find_consumable_name(run, ConsumableKind::Planet, &card.item_id)
            }
        };
        let rarity = card
            .rarity
            .map(|value| format!("{value:?}"))
            .unwrap_or_else(|| "-".to_string());
        let edition = card.edition.map(edition_short).unwrap_or("-");
        println!(
            "  {:>2}: {:<10?} {:<22} ({:<18}) {} {:>3} {} {:<8} {} {}",
            idx,
            card.kind,
            card.item_id,
            item_name,
            locale.text("price", "价格"),
            card.price,
            locale.text("rarity", "稀有度"),
            rarity,
            locale.text("edition", "版本"),
            edition
        );
    }
    println!("{}:", locale.text("packs", "卡包"));
    for (idx, pack) in shop.packs.iter().enumerate() {
        println!(
            "  {:>2}: {:<9?} {:<6?} {} {:>2} {} {:>2} {} {:>3}",
            idx,
            pack.kind,
            pack.size,
            locale.text("options", "选项"),
            pack.options,
            locale.text("picks", "可选"),
            pack.picks,
            locale.text("price", "价格"),
            pack.price
        );
    }
    println!("{}: {}", locale.text("vouchers", "优惠券"), shop.vouchers);
}

fn print_pack_open(locale: UiLocale, open: &PackOpen, run: &RunState) {
    println!(
        "{}: {:?} {:?} ({} {})",
        locale.text("pack opened", "已打开卡包"),
        open.offer.kind,
        open.offer.size,
        locale.text("pick", "选择"),
        open.offer.picks
    );
    for (idx, option) in open.options.iter().enumerate() {
        match option {
            PackOption::Joker(id) => {
                let name = find_joker_name(run, id);
                println!(
                    "{:>2}: {} {} ({})",
                    idx,
                    locale.text("joker", "小丑"),
                    id,
                    name
                );
            }
            PackOption::Consumable(kind, id) => {
                let name = find_consumable_name(run, *kind, id);
                println!("{:>2}: {:?} {} ({})", idx, kind, id, name);
            }
            PackOption::PlayingCard(card) => {
                println!(
                    "{:>2}: {} {}",
                    idx,
                    locale.text("card", "卡牌"),
                    format_card(card)
                );
            }
        }
    }
}

fn print_peek(locale: UiLocale, cards: &[Card], count: usize, label: &str) {
    let label = if matches!(locale, UiLocale::ZhCn) {
        match label {
            "draw" => "抽牌堆",
            "discard" => "弃牌堆",
            _ => label,
        }
    } else {
        label
    };
    if cards.is_empty() {
        println!("{}: {}", label, locale.text("empty", "空"));
        return;
    }
    let total = cards.len();
    let start = total.saturating_sub(count);
    println!(
        "{} {} {}/{}:",
        label,
        locale.text("top", "顶部"),
        total - start,
        total
    );
    for (offset, card) in cards[start..].iter().rev().enumerate() {
        let index = total - 1 - offset;
        println!("{:>2}: {}", index, format_card(card));
    }
}

fn drain_events(locale: UiLocale, events: &mut EventBus) {
    for event in events.drain() {
        println!(
            "{}: {}",
            locale.text("event", "事件"),
            format_event_localized(locale, &event)
        );
    }
}

fn parse_indices_result(args: &[&str]) -> Result<Vec<usize>, String> {
    if args.is_empty() {
        return Err("missing indices".to_string());
    }
    let mut indices = Vec::new();
    for arg in args {
        for part in arg.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            if let Some((start, end)) = part.split_once('-') {
                let start = start
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "invalid range start".to_string())?;
                let end = end
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| "invalid range end".to_string())?;
                if start > end {
                    return Err("range start larger than end".to_string());
                }
                for idx in start..=end {
                    indices.push(idx);
                }
            } else {
                let idx = part
                    .parse::<usize>()
                    .map_err(|_| format!("invalid index '{part}'"))?;
                indices.push(idx);
            }
        }
    }
    if indices.is_empty() {
        return Err("missing indices".to_string());
    }
    Ok(indices)
}

fn parse_optional_indices(args: &[&str]) -> Result<Vec<usize>, String> {
    if args.is_empty() {
        return Ok(Vec::new());
    }
    parse_indices_result(args)
}

fn localize_parse_error(locale: UiLocale, err: &str) -> String {
    if !matches!(locale, UiLocale::ZhCn) {
        return err.to_string();
    }
    if err == "missing indices" {
        return "缺少索引参数".to_string();
    }
    if err == "invalid range start" {
        return "区间起始索引无效".to_string();
    }
    if err == "invalid range end" {
        return "区间结束索引无效".to_string();
    }
    if err == "range start larger than end" {
        return "区间起始值不能大于结束值".to_string();
    }
    if let Some(idx) = err
        .strip_prefix("invalid index '")
        .and_then(|value| value.strip_suffix('\''))
    {
        return format!("无效索引 '{idx}'");
    }
    if let Some(key) = err
        .strip_prefix("unknown edit key '")
        .and_then(|value| value.strip_suffix('\''))
    {
        return format!("未知编辑字段 '{key}'");
    }
    if err.starts_with("invalid enhancement") {
        return err.replacen("invalid enhancement", "无效增强类型", 1);
    }
    if err.starts_with("invalid edition") {
        return err.replacen("invalid edition", "无效版本类型", 1);
    }
    if err.starts_with("invalid seal") {
        return err.replacen("invalid seal", "无效蜡封类型", 1);
    }
    if err == "invalid bonus delta" {
        return "bonus 增量无效".to_string();
    }
    if err == "invalid bonus value" {
        return "bonus 数值无效".to_string();
    }
    if err.starts_with("invalid boolean") {
        return err.replacen("invalid boolean", "无效布尔值", 1);
    }
    if let Some(index) = err
        .strip_prefix("index ")
        .and_then(|v| v.strip_suffix(" out of range"))
    {
        return format!("索引 {index} 超出范围");
    }
    if let Some(name) = err
        .strip_prefix("unknown saved action '")
        .and_then(|value| value.strip_suffix('\''))
    {
        return format!("存档中存在未知动作 '{name}'");
    }
    if err.starts_with("unsupported save version") {
        return err.replacen("unsupported save version", "不支持的存档版本", 1);
    }
    if err.starts_with("content signature mismatch:") {
        return err
            .replacen("content signature mismatch:", "存档校验失败：", 1)
            .replacen("saved=", "存档=", 1)
            .replacen("current=", "当前=", 1);
    }
    err.to_string()
}

fn collect_played_cards(hand: &[Card], indices: &[usize]) -> Result<Vec<Card>, RunError> {
    if indices.is_empty() {
        return Err(RunError::InvalidSelection);
    }
    let mut unique = indices.to_vec();
    unique.sort_unstable();
    unique.dedup();
    if unique.iter().any(|&idx| idx >= hand.len()) {
        return Err(RunError::InvalidSelection);
    }
    unique.sort_unstable_by(|a, b| b.cmp(a));
    let mut picked = Vec::with_capacity(unique.len());
    for idx in unique {
        picked.push(hand[idx]);
    }
    Ok(picked)
}

#[derive(Debug, Clone, Copy)]
enum BonusEdit {
    Set(i64),
    Add(i64),
}

#[derive(Debug, Clone)]
struct CardEdits {
    enhancement: Option<Option<Enhancement>>,
    edition: Option<Option<Edition>>,
    seal: Option<Option<Seal>>,
    bonus: Option<BonusEdit>,
    face_down: Option<bool>,
}

fn parse_edit_args(args: &[&str]) -> Result<(Vec<usize>, CardEdits), String> {
    let mut index_tokens = Vec::new();
    let mut edits = CardEdits {
        enhancement: None,
        edition: None,
        seal: None,
        bonus: None,
        face_down: None,
    };

    for arg in args {
        if let Some((key, value)) = arg.split_once('=') {
            let key = key.trim().to_lowercase();
            let value = value.trim();
            match key.as_str() {
                "enh" | "enhancement" => {
                    edits.enhancement = Some(parse_optional_enhancement(value)?);
                }
                "ed" | "edition" => {
                    edits.edition = Some(parse_optional_edition(value)?);
                }
                "seal" => {
                    edits.seal = Some(parse_optional_seal(value)?);
                }
                "bonus" => {
                    edits.bonus = Some(parse_bonus_edit(value)?);
                }
                "face" | "face_down" => {
                    edits.face_down = Some(parse_bool(value)?);
                }
                _ => return Err(format!("unknown edit key '{key}'")),
            }
        } else {
            index_tokens.push(*arg);
        }
    }

    let indices = parse_indices_result(&index_tokens)?;
    Ok((indices, edits))
}

fn apply_card_edits(hand: &mut [Card], indices: &[usize], edits: CardEdits) -> Result<(), String> {
    if indices.is_empty() {
        return Err("missing indices".to_string());
    }
    for &idx in indices {
        if idx >= hand.len() {
            return Err(format!("index {idx} out of range"));
        }
    }
    for &idx in indices {
        let card = &mut hand[idx];
        if let Some(enh) = edits.enhancement {
            card.enhancement = enh;
        }
        if let Some(edition) = edits.edition {
            card.edition = edition;
        }
        if let Some(seal) = edits.seal {
            card.seal = seal;
        }
        if let Some(bonus) = edits.bonus {
            match bonus {
                BonusEdit::Set(value) => card.bonus_chips = value,
                BonusEdit::Add(delta) => card.bonus_chips = card.bonus_chips.saturating_add(delta),
            }
        }
        if let Some(face_down) = edits.face_down {
            card.face_down = face_down;
        }
    }
    Ok(())
}

fn parse_optional_enhancement(value: &str) -> Result<Option<Enhancement>, String> {
    if is_none(value) {
        return Ok(None);
    }
    parse_enhancement(value).map(Some)
}

fn parse_enhancement(value: &str) -> Result<Enhancement, String> {
    let value = value.trim().to_lowercase();
    match value.as_str() {
        "bonus" => Ok(Enhancement::Bonus),
        "mult" => Ok(Enhancement::Mult),
        "wild" => Ok(Enhancement::Wild),
        "glass" => Ok(Enhancement::Glass),
        "steel" => Ok(Enhancement::Steel),
        "stone" => Ok(Enhancement::Stone),
        "lucky" => Ok(Enhancement::Lucky),
        "gold" => Ok(Enhancement::Gold),
        _ => Err(format!("invalid enhancement '{value}'")),
    }
}

fn parse_optional_edition(value: &str) -> Result<Option<Edition>, String> {
    if is_none(value) {
        return Ok(None);
    }
    parse_edition(value).map(Some)
}

fn parse_edition(value: &str) -> Result<Edition, String> {
    let value = value.trim().to_lowercase();
    match value.as_str() {
        "foil" => Ok(Edition::Foil),
        "holo" | "holographic" => Ok(Edition::Holographic),
        "poly" | "polychrome" => Ok(Edition::Polychrome),
        "neg" | "negative" => Ok(Edition::Negative),
        _ => Err(format!("invalid edition '{value}'")),
    }
}

fn parse_optional_seal(value: &str) -> Result<Option<Seal>, String> {
    if is_none(value) {
        return Ok(None);
    }
    parse_seal(value).map(Some)
}

fn parse_seal(value: &str) -> Result<Seal, String> {
    let value = value.trim().to_lowercase();
    match value.as_str() {
        "red" => Ok(Seal::Red),
        "blue" => Ok(Seal::Blue),
        "gold" => Ok(Seal::Gold),
        "purple" => Ok(Seal::Purple),
        _ => Err(format!("invalid seal '{value}'")),
    }
}

fn parse_bonus_edit(value: &str) -> Result<BonusEdit, String> {
    let value = value.trim();
    if let Some(rest) = value.strip_prefix('+') {
        let amount = rest
            .parse::<i64>()
            .map_err(|_| "invalid bonus delta".to_string())?;
        return Ok(BonusEdit::Add(amount));
    }
    if let Some(rest) = value.strip_prefix('-') {
        let amount = rest
            .parse::<i64>()
            .map_err(|_| "invalid bonus delta".to_string())?;
        return Ok(BonusEdit::Add(-amount));
    }
    let amount = value
        .parse::<i64>()
        .map_err(|_| "invalid bonus value".to_string())?;
    Ok(BonusEdit::Set(amount))
}

fn parse_bool(value: &str) -> Result<bool, String> {
    let value = value.trim().to_lowercase();
    match value.as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(format!("invalid boolean '{value}'")),
    }
}

fn is_none(value: &str) -> bool {
    matches!(
        value.trim().to_lowercase().as_str(),
        "none" | "null" | "clear"
    )
}

fn print_score_breakdown(
    locale: UiLocale,
    breakdown: &ScoreBreakdown,
    played: Option<&[Card]>,
    tables: &ScoreTables,
    trace: &[ScoreTraceStep],
) {
    println!("{}: {:?}", locale.text("hand", "牌型"), breakdown.hand);
    if let Some(cards) = played {
        println!(
            "{}",
            locale.text("played cards (order used):", "已出牌（实际计分顺序）：")
        );
        for (idx, card) in cards.iter().enumerate() {
            println!("  {:>2}: {}", idx, format_card(card));
        }
    }
    println!(
        "{}: {:?}",
        locale.text("scoring indices", "计分索引"),
        breakdown.scoring_indices
    );
    println!(
        "{}: {}={} {}={:.2}",
        locale.text("base", "基础"),
        locale.text("chips", "筹码"),
        breakdown.base.chips,
        locale.text("mult", "倍率"),
        breakdown.base.mult
    );
    if let Some(cards) = played {
        let mut rank_total = 0i64;
        println!("{}", locale.text("rank chips breakdown:", "牌面筹码明细："));
        for &idx in &breakdown.scoring_indices {
            if let Some(card) = cards.get(idx) {
                let chips = if card.is_stone() {
                    0
                } else {
                    tables.rank_chips(card.rank)
                };
                rank_total += chips;
                println!("  {:>2}: {} => {}", idx, format_card(card), chips);
            }
        }
        println!(
            "{}: {}",
            locale.text("rank chips total", "牌面筹码合计"),
            rank_total
        );
    } else {
        println!(
            "{}: {}",
            locale.text("rank chips total", "牌面筹码合计"),
            breakdown.rank_chips
        );
    }
    println!(
        "{}: {} {} + {} {} = {} ({})",
        locale.text("chips", "筹码"),
        locale.text("base", "基础"),
        breakdown.base.chips,
        locale.text("rank", "牌面"),
        breakdown.rank_chips,
        breakdown.base.chips + breakdown.rank_chips,
        locale.text("before effects", "效果结算前")
    );
    println!(
        "{}: {}={} {}={:.2} {}={}",
        locale.text("final", "最终"),
        locale.text("chips", "筹码"),
        breakdown.total.chips,
        locale.text("mult", "倍率"),
        breakdown.total.mult,
        locale.text("score", "总分"),
        breakdown.total.total()
    );

    if trace.is_empty() {
        println!("{}", locale.text("effect steps: none", "效果步骤：无"));
    } else {
        println!("{}", locale.text("effect steps:", "效果步骤："));
        for (idx, step) in trace.iter().enumerate() {
            println!(
                "  {:>2}. {} | {:?} | {}×{:.2} -> {}×{:.2}",
                idx + 1,
                step.source,
                step.effect,
                step.before.chips,
                step.before.mult,
                step.after.chips,
                step.after.mult
            );
        }
    }
}

fn estimate_interest(run: &RunState) -> i64 {
    let economy = &run.config.economy;
    if economy.interest_step <= 0 || economy.interest_per <= 0 {
        return 0;
    }
    let steps = (run.state.money / economy.interest_step).max(0);
    let cap_steps = if economy.interest_per > 0 {
        economy.interest_cap / economy.interest_per
    } else {
        0
    };
    let capped = steps.min(cap_steps);
    capped * economy.interest_per
}

fn format_card(card: &Card) -> String {
    if card.face_down {
        return "??".to_string();
    }
    let mut out = format!("{}{}", rank_short(card.rank), suit_short(card.suit));
    let mut tags = Vec::new();
    if let Some(enhancement) = card.enhancement {
        tags.push(enhancement_short(enhancement));
    }
    if let Some(edition) = card.edition {
        tags.push(edition_short(edition));
    }
    if let Some(seal) = card.seal {
        tags.push(seal_short(seal));
    }
    if card.bonus_chips != 0 {
        tags.push("Bonus");
    }
    if !tags.is_empty() {
        out.push_str(" [");
        out.push_str(&tags.join(","));
        out.push(']');
    }
    out
}

fn card_value(card: &Card, tables: &ScoreTables) -> i64 {
    if card.is_stone() {
        return 0;
    }
    tables.rank_chips(card.rank) + card.bonus_chips
}

fn card_detail(card: &Card) -> String {
    if card.face_down {
        return "face_down".to_string();
    }
    let mut tags = Vec::new();
    tags.push(format!("{:?}{:?}", card.rank, card.suit));
    if let Some(enhancement) = card.enhancement {
        tags.push(format!("enh={}", enhancement_short(enhancement)));
    }
    if let Some(edition) = card.edition {
        tags.push(format!("ed={}", edition_short(edition)));
    }
    if let Some(seal) = card.seal {
        tags.push(format!("seal={}", seal_short(seal)));
    }
    if card.bonus_chips != 0 {
        tags.push(format!("bonus={}", card.bonus_chips));
    }
    tags.join(" ")
}

fn phase_short(phase: Phase) -> &'static str {
    match phase {
        Phase::Setup => "Setup",
        Phase::Deal => "Deal",
        Phase::Play => "Play",
        Phase::Score => "Score",
        Phase::Cleanup => "Clean",
        Phase::Shop => "Shop",
    }
}

fn phase_label(locale: UiLocale, phase: Phase) -> &'static str {
    if matches!(locale, UiLocale::ZhCn) {
        match phase {
            Phase::Setup => "准备",
            Phase::Deal => "发牌",
            Phase::Play => "出牌",
            Phase::Score => "计分",
            Phase::Cleanup => "清理",
            Phase::Shop => "商店",
        }
    } else {
        phase_short(phase)
    }
}

fn blind_short(blind: BlindKind) -> &'static str {
    match blind {
        BlindKind::Small => "Small",
        BlindKind::Big => "Big",
        BlindKind::Boss => "Boss",
    }
}

fn blind_label(locale: UiLocale, blind: BlindKind) -> &'static str {
    if matches!(locale, UiLocale::ZhCn) {
        match blind {
            BlindKind::Small => "小盲",
            BlindKind::Big => "大盲",
            BlindKind::Boss => "Boss",
        }
    } else {
        blind_short(blind)
    }
}

fn format_event(event: &Event) -> String {
    match event {
        Event::BlindStarted {
            ante,
            blind,
            target,
            hands,
            discards,
        } => format!(
            "blind started: ante {ante} {blind:?} target {target} hands {hands} discards {discards}"
        ),
        Event::BlindSkipped { ante, blind, tag } => format!(
            "blind skipped: ante {ante} {blind:?} tag {}",
            tag.as_deref().unwrap_or("none")
        ),
        Event::HandDealt { count } => format!("hand dealt: {count} cards"),
        Event::HandScored {
            hand,
            chips,
            mult,
            total,
        } => format!("hand scored: {hand:?} {chips}x{mult:.2} = {total}"),
        Event::ShopEntered {
            offers,
            reroll_cost,
            reentered,
        } => format!(
            "shop entered: offers {offers} reroll {reroll_cost}{}",
            if *reentered { " (reenter)" } else { "" }
        ),
        Event::ShopRerolled {
            offers,
            reroll_cost,
            cost,
            money,
        } => {
            format!("shop rerolled: offers {offers} reroll {reroll_cost} cost {cost} money {money}")
        }
        Event::ShopBought { offer, cost, money } => {
            format!("shop bought: {offer:?} cost {cost} money {money}")
        }
        Event::PackOpened {
            kind,
            options,
            picks,
        } => format!("pack opened: {kind:?} options {options} picks {picks}"),
        Event::PackChosen { picks } => format!("pack chosen: {picks}"),
        Event::JokerSold {
            id,
            sell_value,
            money,
        } => format!("joker sold: {id} value {sell_value} money {money}"),
        Event::BlindCleared {
            score,
            reward,
            money,
        } => {
            format!("blind cleared: score {score} reward {reward} money {money}")
        }
        Event::BlindFailed { score } => format!("blind failed: score {score}"),
    }
}

fn format_event_localized(locale: UiLocale, event: &Event) -> String {
    if !matches!(locale, UiLocale::ZhCn) {
        return format_event(event);
    }
    match event {
        Event::BlindStarted {
            ante,
            blind,
            target,
            hands,
            discards,
        } => format!(
            "盲注开始：底注 {ante} {} 目标 {target} 出牌 {hands} 弃牌 {discards}",
            blind_label(locale, *blind)
        ),
        Event::BlindSkipped { ante, blind, tag } => format!(
            "盲注已跳过：底注 {ante} {} 标签 {}",
            blind_label(locale, *blind),
            tag.as_deref().unwrap_or("无")
        ),
        Event::HandDealt { count } => format!("已发牌：{count} 张"),
        Event::HandScored {
            hand,
            chips,
            mult,
            total,
        } => format!("手牌计分：{hand:?} {chips}x{mult:.2} = {total}"),
        Event::ShopEntered {
            offers,
            reroll_cost,
            reentered,
        } => format!(
            "进入商店：商品 {offers} 刷新费用 {reroll_cost}{}",
            if *reentered { "（重新进入）" } else { "" }
        ),
        Event::ShopRerolled {
            offers,
            reroll_cost,
            cost,
            money,
        } => format!("商店刷新：商品 {offers} 刷新费用 {reroll_cost} 花费 {cost} 金钱 {money}"),
        Event::ShopBought { offer, cost, money } => {
            format!("商店购买：{offer:?} 花费 {cost} 金钱 {money}")
        }
        Event::PackOpened {
            kind,
            options,
            picks,
        } => format!("卡包打开：{kind:?} 选项 {options} 可选 {picks}"),
        Event::PackChosen { picks } => format!("卡包选择：{picks}"),
        Event::JokerSold {
            id,
            sell_value,
            money,
        } => format!("出售小丑：{id} 价值 {sell_value} 金钱 {money}"),
        Event::BlindCleared {
            score,
            reward,
            money,
        } => {
            format!("盲注通过：分数 {score} 奖励 {reward} 金钱 {money}")
        }
        Event::BlindFailed { score } => format!("盲注失败：分数 {score}"),
    }
}

fn rank_short(rank: Rank) -> &'static str {
    match rank {
        Rank::Ace => "A",
        Rank::King => "K",
        Rank::Queen => "Q",
        Rank::Jack => "J",
        Rank::Ten => "T",
        Rank::Nine => "9",
        Rank::Eight => "8",
        Rank::Seven => "7",
        Rank::Six => "6",
        Rank::Five => "5",
        Rank::Four => "4",
        Rank::Three => "3",
        Rank::Two => "2",
        Rank::Joker => "Jk",
    }
}

fn suit_short(suit: Suit) -> &'static str {
    match suit {
        Suit::Spades => "S",
        Suit::Hearts => "H",
        Suit::Clubs => "C",
        Suit::Diamonds => "D",
        Suit::Wild => "W",
    }
}

fn enhancement_short(kind: Enhancement) -> &'static str {
    match kind {
        Enhancement::Bonus => "Bonus",
        Enhancement::Mult => "Mult",
        Enhancement::Wild => "Wild",
        Enhancement::Glass => "Glass",
        Enhancement::Steel => "Steel",
        Enhancement::Stone => "Stone",
        Enhancement::Lucky => "Lucky",
        Enhancement::Gold => "Gold",
    }
}

fn edition_short(kind: Edition) -> &'static str {
    match kind {
        Edition::Foil => "Foil",
        Edition::Holographic => "Holo",
        Edition::Polychrome => "Poly",
        Edition::Negative => "Neg",
    }
}

fn seal_short(kind: Seal) -> &'static str {
    match kind {
        Seal::Red => "R",
        Seal::Blue => "B",
        Seal::Gold => "G",
        Seal::Purple => "P",
    }
}

fn find_joker_name(run: &RunState, id: &str) -> String {
    run.content
        .jokers
        .iter()
        .find(|joker| joker.id == id)
        .map(|joker| joker.name.clone())
        .unwrap_or_else(|| "-".to_string())
}

fn find_consumable_name(run: &RunState, kind: ConsumableKind, id: &str) -> String {
    let list = match kind {
        ConsumableKind::Tarot => &run.content.tarots,
        ConsumableKind::Planet => &run.content.planets,
        ConsumableKind::Spectral => &run.content.spectrals,
    };
    list.iter()
        .find(|card| card.id == id)
        .map(|card| card.name.clone())
        .unwrap_or_else(|| "-".to_string())
}
