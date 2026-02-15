use anyhow::{anyhow, bail, Context, Result};
use rulatro_core::{
    Action, ActionOp, ActivationType, BinaryOp, BossDef, Expr, JokerDef, JokerEffect, JokerRarity,
    TagDef, UnaryOp,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[allow(dead_code)]
pub fn load_jokers_dsl(path: &Path) -> Result<Vec<JokerDef>> {
    load_jokers_dsl_with_locale(path, None)
}

pub fn load_jokers_dsl_with_locale(path: &Path, locale: Option<&str>) -> Result<Vec<JokerDef>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let expanded = expand_templates(&raw)?;
    parse_jokers(&expanded, locale)
}

#[allow(dead_code)]
pub fn load_bosses_dsl(path: &Path) -> Result<Vec<BossDef>> {
    load_bosses_dsl_with_locale(path, None)
}

pub fn load_bosses_dsl_with_locale(path: &Path, locale: Option<&str>) -> Result<Vec<BossDef>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let expanded = expand_templates(&raw)?;
    parse_named_defs(&expanded, "boss", locale).map(|defs| {
        defs.into_iter()
            .map(|def| BossDef {
                id: def.id,
                name: def.name,
                effects: def.effects,
            })
            .collect()
    })
}

#[allow(dead_code)]
pub fn load_tags_dsl(path: &Path) -> Result<Vec<TagDef>> {
    load_tags_dsl_with_locale(path, None)
}

pub fn load_tags_dsl_with_locale(path: &Path, locale: Option<&str>) -> Result<Vec<TagDef>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let expanded = expand_templates(&raw)?;
    parse_named_defs(&expanded, "tag", locale).map(|defs| {
        defs.into_iter()
            .map(|def| TagDef {
                id: def.id,
                name: def.name,
                effects: def.effects,
            })
            .collect()
    })
}

pub(crate) fn load_joker_mixin_refs(path: &Path) -> Result<HashMap<String, Vec<String>>> {
    load_named_mixin_refs(path, "joker")
}

pub(crate) fn load_tag_mixin_refs(path: &Path) -> Result<HashMap<String, Vec<String>>> {
    load_named_mixin_refs(path, "tag")
}

pub(crate) fn load_boss_mixin_refs(path: &Path) -> Result<HashMap<String, Vec<String>>> {
    load_named_mixin_refs(path, "boss")
}

pub(crate) fn parse_effect_dsl_line(line: &str) -> Result<JokerEffect> {
    parse_effect_line(line.trim())
}

#[derive(Debug, Clone)]
struct Template {
    params: Vec<String>,
    body: String,
}

#[derive(Debug, Clone)]
struct NamedDef {
    id: String,
    name: String,
    effects: Vec<JokerEffect>,
}

fn expand_templates(src: &str) -> Result<String> {
    let mut templates: HashMap<String, Template> = HashMap::new();
    let mut output = String::new();
    let mut lines = src.lines().peekable();

    while let Some(line) = lines.next() {
        let line = strip_comments(line);
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("template ") {
            let (name, params, remainder) = parse_template_header(trimmed)?;
            let mut body = String::new();
            let mut depth = 1;
            if !remainder.is_empty() {
                let (chunk, new_depth, done) = consume_until_close(&remainder, depth);
                body.push_str(chunk.trim_end());
                body.push('\n');
                depth = new_depth;
                if done {
                    templates.insert(name, Template { params, body });
                    continue;
                }
            }

            while depth > 0 {
                let next_line = lines
                    .next()
                    .ok_or_else(|| anyhow!("unterminated template block"))?;
                let clean = strip_comments(next_line);
                let (chunk, new_depth, done) = consume_until_close(&clean, depth);
                if !chunk.trim().is_empty() {
                    body.push_str(chunk.trim_end());
                    body.push('\n');
                }
                depth = new_depth;
                if done {
                    break;
                }
            }
            templates.insert(name, Template { params, body });
            continue;
        }

        if trimmed.starts_with("use ") {
            let (name, args) = parse_use(trimmed)?;
            let template = templates
                .get(&name)
                .ok_or_else(|| anyhow!("unknown template '{}'", name))?;
            if template.params.len() != args.len() {
                bail!(
                    "template '{}' expects {} args, got {}",
                    name,
                    template.params.len(),
                    args.len()
                );
            }
            let mut expanded = template.body.clone();
            for (param, arg) in template.params.iter().zip(args.iter()) {
                expanded = expanded.replace(&format!("${}", param), arg);
            }
            output.push_str(expanded.trim_end());
            output.push('\n');
            continue;
        }

        output.push_str(line);
        output.push('\n');
    }

    Ok(output)
}

fn parse_template_header(line: &str) -> Result<(String, Vec<String>, String)> {
    let rest = line
        .trim()
        .strip_prefix("template")
        .ok_or_else(|| anyhow!("missing template keyword"))?
        .trim();
    let name_end = rest
        .find('(')
        .ok_or_else(|| anyhow!("template missing '('"))?;
    let name = rest[..name_end].trim().to_string();
    let rest = &rest[name_end + 1..];
    let params_end = rest
        .find(')')
        .ok_or_else(|| anyhow!("template missing ')'"))?;
    let params_str = rest[..params_end].trim();
    let params = split_args(params_str)
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect();
    let rest = rest[params_end + 1..].trim();
    let brace_idx = rest
        .find('{')
        .ok_or_else(|| anyhow!("template missing '{{'"))?;
    let remainder = rest[brace_idx + 1..].to_string();
    Ok((name, params, remainder))
}

fn parse_use(line: &str) -> Result<(String, Vec<String>)> {
    let rest = line
        .trim()
        .strip_prefix("use")
        .ok_or_else(|| anyhow!("missing use keyword"))?
        .trim();
    let name_end = rest.find('(').ok_or_else(|| anyhow!("use missing '('"))?;
    let name = rest[..name_end].trim().to_string();
    let rest = &rest[name_end + 1..];
    let args_end = rest.find(')').ok_or_else(|| anyhow!("use missing ')'"))?;
    let args_str = rest[..args_end].trim();
    Ok((name, split_args(args_str)))
}

fn parse_jokers(src: &str, locale: Option<&str>) -> Result<Vec<JokerDef>> {
    let blocks = parse_blocks(src, "joker")?;
    let mut jokers = Vec::new();
    for block in blocks {
        if block.tokens.len() < 4 {
            bail!("joker header missing id/name/rarity");
        }
        let id = token_to_string(block.tokens.get(1)).ok_or_else(|| anyhow!("joker id missing"))?;
        let name =
            token_to_string(block.tokens.get(2)).ok_or_else(|| anyhow!("joker name missing"))?;
        let name = resolve_localized_name(name, &block.body, locale)?;
        let rarity_str =
            token_to_string(block.tokens.get(3)).ok_or_else(|| anyhow!("joker rarity missing"))?;
        let rarity = parse_rarity(&rarity_str)?;
        let effects = parse_effects(&block.body)?;
        jokers.push(JokerDef {
            id,
            name,
            rarity,
            effects,
        });
    }
    Ok(jokers)
}

fn parse_named_defs(src: &str, keyword: &str, locale: Option<&str>) -> Result<Vec<NamedDef>> {
    let blocks = parse_blocks(src, keyword)?;
    let mut defs = Vec::new();
    for block in blocks {
        if block.tokens.len() < 3 {
            bail!("{} header missing id/name", keyword);
        }
        let id = token_to_string(block.tokens.get(1))
            .ok_or_else(|| anyhow!("{} id missing", keyword))?;
        let name = token_to_string(block.tokens.get(2))
            .ok_or_else(|| anyhow!("{} name missing", keyword))?;
        let name = resolve_localized_name(name, &block.body, locale)?;
        let effects = parse_effects(&block.body)?;
        defs.push(NamedDef { id, name, effects });
    }
    Ok(defs)
}

fn load_named_mixin_refs(path: &Path, keyword: &str) -> Result<HashMap<String, Vec<String>>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let expanded = expand_templates(&raw)?;
    let blocks = parse_blocks(&expanded, keyword)?;
    let mut refs = HashMap::new();
    for block in blocks {
        let id = token_to_string(block.tokens.get(1))
            .ok_or_else(|| anyhow!("{} id missing", keyword))?;
        refs.insert(id, parse_block_mixin_refs(&block.body)?);
    }
    Ok(refs)
}

fn parse_block_mixin_refs(body: &str) -> Result<Vec<String>> {
    let mut refs = Vec::new();
    for raw_line in body.lines() {
        let clean = strip_comments(raw_line);
        let line = clean.trim();
        if line.is_empty() {
            continue;
        }
        if line == "mixin" || line == "mixins" {
            bail!("invalid mixin line '{}': missing mixin id", line);
        }
        if !line.starts_with("mixin ") && !line.starts_with("mixins ") {
            continue;
        }
        let (_, tail) = line
            .split_once(char::is_whitespace)
            .ok_or_else(|| anyhow!("invalid mixin line '{}'", line))?;
        let mut found = false;
        for token in tail.replace(',', " ").split_whitespace() {
            let id = token.trim();
            if id.is_empty() {
                continue;
            }
            refs.push(id.to_string());
            found = true;
        }
        if !found {
            bail!("invalid mixin line '{}': missing mixin id", line);
        }
    }
    Ok(refs)
}

fn resolve_localized_name(base_name: String, body: &str, locale: Option<&str>) -> Result<String> {
    let Some(locale) = locale else {
        return Ok(base_name);
    };
    let locale = normalize_locale_key(locale);
    if locale == "en_US" {
        return Ok(base_name);
    }
    let overrides = parse_name_overrides(body)?;
    Ok(overrides.get(&locale).cloned().unwrap_or(base_name))
}

fn parse_name_overrides(body: &str) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for raw_line in body.lines() {
        let clean = strip_comments(raw_line);
        let line = clean.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("on ") {
            continue;
        }
        if line.starts_with("i18n ") {
            let tokens = tokenize_simple(line)?;
            if tokens.len() < 3 {
                bail!(
                    "invalid i18n line '{}': expected i18n <locale> \"<name>\"",
                    line
                );
            }
            let locale =
                token_to_string(tokens.get(1)).ok_or_else(|| anyhow!("i18n locale missing"))?;
            let value =
                token_to_string(tokens.get(2)).ok_or_else(|| anyhow!("i18n name missing"))?;
            map.insert(normalize_locale_key(&locale), value);
            continue;
        }
        if let Some((prefix, value_part)) = line.split_once(char::is_whitespace) {
            if let Some(locale) = prefix.strip_prefix("name.") {
                let value_tokens = tokenize_simple(value_part.trim())?;
                let value = token_to_string(value_tokens.get(0))
                    .ok_or_else(|| anyhow!("localized name missing"))?;
                map.insert(normalize_locale_key(locale), value);
            }
        }
    }
    Ok(map)
}

fn normalize_locale_key(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return "en_US".to_string();
    }
    let lowered = trimmed.replace('-', "_").to_ascii_lowercase();
    match lowered.as_str() {
        "zh" | "zh_cn" | "zh_hans" | "zh_hans_cn" => "zh_CN".to_string(),
        "en" | "en_us" => "en_US".to_string(),
        _ => trimmed.replace('-', "_"),
    }
}

#[derive(Debug, Clone)]
struct Block {
    tokens: Vec<Token>,
    body: String,
}

fn parse_blocks(src: &str, keyword: &str) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();
    let mut lines = src.lines().peekable();

    while let Some(line) = lines.next() {
        let line = strip_comments(line);
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.starts_with(keyword) {
            continue;
        }
        let parts: Vec<&str> = trimmed.splitn(2, '{').collect();
        let header = parts
            .get(0)
            .map(|s| s.trim())
            .ok_or_else(|| anyhow!("invalid {} header", keyword))?;
        let remainder = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
        let tokens = tokenize_simple(header)?;
        if tokens.is_empty() || tokens[0] != Token::Ident(keyword.to_string()) {
            bail!("{} header must start with '{}'", keyword, keyword);
        }

        let mut body = String::new();
        let mut depth = 1;
        if !remainder.is_empty() {
            let (chunk, new_depth, done) = consume_until_close(&remainder, depth);
            body.push_str(chunk.trim_end());
            body.push('\n');
            depth = new_depth;
            if done {
                blocks.push(Block { tokens, body });
                continue;
            }
        }

        while depth > 0 {
            let next_line = lines
                .next()
                .ok_or_else(|| anyhow!("unterminated {} block", keyword))?;
            let clean = strip_comments(next_line);
            let (chunk, new_depth, done) = consume_until_close(&clean, depth);
            if !chunk.trim().is_empty() {
                body.push_str(chunk.trim_end());
                body.push('\n');
            }
            depth = new_depth;
            if done {
                break;
            }
        }
        blocks.push(Block { tokens, body });
    }

    Ok(blocks)
}

fn parse_effects(body: &str) -> Result<Vec<JokerEffect>> {
    let mut effects = Vec::new();
    for line in body.lines() {
        let clean = strip_comments(line);
        let trimmed = clean.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.starts_with("on ") {
            continue;
        }
        effects.push(parse_effect_line(trimmed)?);
    }
    Ok(effects)
}

fn parse_effect_line(line: &str) -> Result<JokerEffect> {
    let brace_start = line
        .find('{')
        .ok_or_else(|| anyhow!("effect missing '{{'"))?;
    let brace_end = line
        .rfind('}')
        .ok_or_else(|| anyhow!("effect missing '}}'"))?;
    let header = line[..brace_start].trim();
    let actions_str = line[brace_start + 1..brace_end].trim();

    let header = header
        .strip_prefix("on")
        .ok_or_else(|| anyhow!("effect missing 'on'"))?
        .trim();

    let (trigger_str, when_str) = if let Some(pos) = header.find(" when ") {
        let trigger = header[..pos].trim();
        let when = header[pos + 6..].trim();
        (trigger, Some(when))
    } else {
        (header, None)
    };

    let trigger = parse_trigger(trigger_str)?;
    let when = if let Some(expr) = when_str {
        parse_expr(expr)?
    } else {
        Expr::Bool(true)
    };
    let actions = parse_actions(actions_str)?;

    Ok(JokerEffect {
        trigger,
        when,
        actions,
    })
}

fn parse_actions(input: &str) -> Result<Vec<Action>> {
    let mut actions = Vec::new();
    for part in split_actions(input) {
        let piece = part.trim();
        if piece.is_empty() {
            continue;
        }
        let mut iter = piece.splitn(2, char::is_whitespace);
        let op_str = iter.next().unwrap_or_default();
        let op =
            ActionOp::from_keyword(op_str).ok_or_else(|| anyhow!("unknown action '{}'", op_str))?;
        let expr_str = iter.next().unwrap_or("1").trim();
        if op.requires_target() {
            let mut arg_iter = expr_str.splitn(2, char::is_whitespace);
            let target_raw = arg_iter.next().unwrap_or_default().trim();
            if target_raw.is_empty() {
                bail!("action '{}' requires a target name", op_str);
            }
            let target = parse_action_target(target_raw)?;
            let rest = arg_iter.next().unwrap_or("").trim();
            let value = if rest.is_empty() {
                Expr::Number(1.0)
            } else {
                parse_expr(rest)?
            };
            actions.push(Action {
                op,
                target: Some(target),
                value,
            });
        } else {
            let value = if expr_str.is_empty() {
                Expr::Number(1.0)
            } else {
                parse_expr(expr_str)?
            };
            actions.push(Action {
                op,
                target: None,
                value,
            });
        }
    }
    Ok(actions)
}

fn parse_action_target(input: &str) -> Result<String> {
    let tokens = tokenize_simple(input)?;
    match tokens.get(0) {
        Some(Token::Ident(value)) => Ok(value.clone()),
        Some(Token::Str(value)) => Ok(value.clone()),
        _ => Err(anyhow!("invalid action target '{}'", input)),
    }
}

fn parse_trigger(input: &str) -> Result<ActivationType> {
    match input.trim().to_lowercase().as_str() {
        "played" => Ok(ActivationType::OnPlayed),
        "scored_pre" | "score_pre" | "scored_before" => Ok(ActivationType::OnScoredPre),
        "scored" => Ok(ActivationType::OnScored),
        "held" => Ok(ActivationType::OnHeld),
        "independent" => Ok(ActivationType::Independent),
        "discard" | "discarded" => Ok(ActivationType::OnDiscard),
        "discard_batch" | "discarded_batch" | "discard_group" => Ok(ActivationType::OnDiscardBatch),
        "destroyed" | "card_destroyed" | "carddestroyed" => Ok(ActivationType::OnCardDestroyed),
        "card_added" | "cardadded" | "deck_added" | "deckadded" => Ok(ActivationType::OnCardAdded),
        "round_end" | "roundend" => Ok(ActivationType::OnRoundEnd),
        "hand_end" | "handend" | "hand_scored" | "handscored" => Ok(ActivationType::OnHandEnd),
        "blind_start" | "blindstart" | "blind_selected" | "blindselect" => {
            Ok(ActivationType::OnBlindStart)
        }
        "blind_failed" | "blindfail" | "blind_fail" => Ok(ActivationType::OnBlindFailed),
        "shop_enter" | "shopenter" | "shop_start" | "shopstart" => Ok(ActivationType::OnShopEnter),
        "shop_reroll" | "shopreroll" => Ok(ActivationType::OnShopReroll),
        "shop_exit" | "shopexit" | "shop_end" | "shopend" => Ok(ActivationType::OnShopExit),
        "pack_opened" | "packopen" | "pack_open" | "booster_opened" => {
            Ok(ActivationType::OnPackOpened)
        }
        "pack_skipped" | "pack_skip" | "booster_skipped" => Ok(ActivationType::OnPackSkipped),
        "use" => Ok(ActivationType::OnUse),
        "sell" | "sold" => Ok(ActivationType::OnSell),
        "sell_any" | "any_sell" | "sold_any" => Ok(ActivationType::OnAnySell),
        "acquire" | "acquired" | "gain" | "gained" => Ok(ActivationType::OnAcquire),
        "passive" => Ok(ActivationType::Passive),
        "other_jokers" | "otherjokers" => Ok(ActivationType::OnOtherJokers),
        _ => Err(anyhow!("unknown trigger '{}'", input)),
    }
}

fn parse_rarity(value: &str) -> Result<JokerRarity> {
    match value.trim().to_lowercase().as_str() {
        "common" => Ok(JokerRarity::Common),
        "uncommon" => Ok(JokerRarity::Uncommon),
        "rare" => Ok(JokerRarity::Rare),
        "legendary" => Ok(JokerRarity::Legendary),
        _ => Err(anyhow!("unknown rarity '{}'", value)),
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    Str(String),
}

fn tokenize_simple(input: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.peek().copied() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        if ch == '"' {
            chars.next();
            let mut value = String::new();
            while let Some(next) = chars.next() {
                if next == '"' {
                    break;
                }
                value.push(next);
            }
            tokens.push(Token::Str(value));
            continue;
        }
        let mut ident = String::new();
        while let Some(next) = chars.peek().copied() {
            if next.is_whitespace() || next == '{' || next == '}' {
                break;
            }
            ident.push(next);
            chars.next();
        }
        if !ident.is_empty() {
            tokens.push(Token::Ident(ident));
        } else {
            chars.next();
        }
    }
    Ok(tokens)
}

fn token_to_string(token: Option<&Token>) -> Option<String> {
    match token {
        Some(Token::Ident(value)) => Some(value.clone()),
        Some(Token::Str(value)) => Some(value.clone()),
        None => None,
    }
}

fn split_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '"' {
            in_string = !in_string;
            current.push(ch);
            continue;
        }
        if ch == ',' && !in_string {
            args.push(current.trim().to_string());
            current.clear();
            continue;
        }
        current.push(ch);
    }
    if !current.trim().is_empty() {
        args.push(current.trim().to_string());
    }
    args
}

fn split_actions(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut paren_depth = 0i32;

    for ch in input.chars() {
        if ch == '"' {
            in_string = !in_string;
            current.push(ch);
            continue;
        }
        if !in_string {
            if ch == '(' {
                paren_depth += 1;
            } else if ch == ')' {
                paren_depth -= 1;
            } else if (ch == ';' || ch == ',') && paren_depth == 0 {
                parts.push(current.trim().to_string());
                current.clear();
                continue;
            }
        }
        current.push(ch);
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

fn consume_until_close(line: &str, mut depth: i32) -> (&str, i32, bool) {
    let mut in_string = false;
    for (idx, ch) in line.char_indices() {
        if ch == '"' {
            in_string = !in_string;
        }
        if in_string {
            continue;
        }
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return (&line[..idx], depth, true);
            }
        }
    }
    (line, depth, false)
}

fn strip_comments(line: &str) -> &str {
    let mut in_string = false;
    let mut prev = '\0';
    for (idx, ch) in line.char_indices() {
        if ch == '"' && prev != '\\' {
            in_string = !in_string;
        }
        if !in_string {
            if ch == '#' {
                return &line[..idx];
            }
            if prev == '/' && ch == '/' {
                return &line[..idx - 1];
            }
        }
        prev = ch;
    }
    line
}

fn parse_expr(input: &str) -> Result<Expr> {
    let mut parser = ExprParser::new(input)?;
    let expr = parser.parse_expr()?;
    Ok(expr)
}

#[derive(Debug, Clone, PartialEq)]
enum ExprToken {
    Ident(String),
    Number(f64),
    Str(String),
    Op(String),
    LParen,
    RParen,
    Comma,
}

struct ExprParser {
    tokens: Vec<ExprToken>,
    pos: usize,
}

impl ExprParser {
    fn new(input: &str) -> Result<Self> {
        let tokens = tokenize_expr(input)?;
        Ok(Self { tokens, pos: 0 })
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut node = self.parse_and()?;
        while self.match_op("||") {
            let right = self.parse_and()?;
            node = Expr::Binary {
                left: Box::new(node),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        Ok(node)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut node = self.parse_eq()?;
        while self.match_op("&&") {
            let right = self.parse_eq()?;
            node = Expr::Binary {
                left: Box::new(node),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }
        Ok(node)
    }

    fn parse_eq(&mut self) -> Result<Expr> {
        let mut node = self.parse_rel()?;
        loop {
            if self.match_op("==") {
                let right = self.parse_rel()?;
                node = Expr::Binary {
                    left: Box::new(node),
                    op: BinaryOp::Eq,
                    right: Box::new(right),
                };
            } else if self.match_op("!=") {
                let right = self.parse_rel()?;
                node = Expr::Binary {
                    left: Box::new(node),
                    op: BinaryOp::Ne,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_rel(&mut self) -> Result<Expr> {
        let mut node = self.parse_add()?;
        loop {
            if self.match_op("<=") {
                let right = self.parse_add()?;
                node = Expr::Binary {
                    left: Box::new(node),
                    op: BinaryOp::Le,
                    right: Box::new(right),
                };
            } else if self.match_op(">=") {
                let right = self.parse_add()?;
                node = Expr::Binary {
                    left: Box::new(node),
                    op: BinaryOp::Ge,
                    right: Box::new(right),
                };
            } else if self.match_op("<") {
                let right = self.parse_add()?;
                node = Expr::Binary {
                    left: Box::new(node),
                    op: BinaryOp::Lt,
                    right: Box::new(right),
                };
            } else if self.match_op(">") {
                let right = self.parse_add()?;
                node = Expr::Binary {
                    left: Box::new(node),
                    op: BinaryOp::Gt,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_add(&mut self) -> Result<Expr> {
        let mut node = self.parse_mul()?;
        loop {
            if self.match_op("+") {
                let right = self.parse_mul()?;
                node = Expr::Binary {
                    left: Box::new(node),
                    op: BinaryOp::Add,
                    right: Box::new(right),
                };
            } else if self.match_op("-") {
                let right = self.parse_mul()?;
                node = Expr::Binary {
                    left: Box::new(node),
                    op: BinaryOp::Sub,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_mul(&mut self) -> Result<Expr> {
        let mut node = self.parse_unary()?;
        loop {
            if self.match_op("*") {
                let right = self.parse_unary()?;
                node = Expr::Binary {
                    left: Box::new(node),
                    op: BinaryOp::Mul,
                    right: Box::new(right),
                };
            } else if self.match_op("/") {
                let right = self.parse_unary()?;
                node = Expr::Binary {
                    left: Box::new(node),
                    op: BinaryOp::Div,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(node)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if self.match_op("!") {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            });
        }
        if self.match_op("-") {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(expr),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.next_token() {
            Some(ExprToken::Number(value)) => Ok(Expr::Number(value)),
            Some(ExprToken::Str(value)) => Ok(Expr::String(value)),
            Some(ExprToken::Ident(value)) => {
                if self.match_lparen() {
                    let mut args = Vec::new();
                    if !self.peek_rparen() {
                        loop {
                            args.push(self.parse_expr()?);
                            if self.match_comma() {
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect_rparen()?;
                    Ok(Expr::Call { name: value, args })
                } else {
                    match value.as_str() {
                        "true" => Ok(Expr::Bool(true)),
                        "false" => Ok(Expr::Bool(false)),
                        _ => Ok(Expr::Ident(value)),
                    }
                }
            }
            Some(ExprToken::LParen) => {
                let expr = self.parse_expr()?;
                self.expect_rparen()?;
                Ok(expr)
            }
            other => Err(anyhow!("unexpected token in expression: {:?}", other)),
        }
    }

    fn match_op(&mut self, op: &str) -> bool {
        if let Some(ExprToken::Op(value)) = self.peek_token() {
            if value == op {
                self.pos += 1;
                return true;
            }
        }
        false
    }

    fn match_lparen(&mut self) -> bool {
        if let Some(ExprToken::LParen) = self.peek_token() {
            self.pos += 1;
            return true;
        }
        false
    }

    fn match_comma(&mut self) -> bool {
        if let Some(ExprToken::Comma) = self.peek_token() {
            self.pos += 1;
            return true;
        }
        false
    }

    fn peek_rparen(&self) -> bool {
        matches!(self.peek_token(), Some(ExprToken::RParen))
    }

    fn expect_rparen(&mut self) -> Result<()> {
        match self.next_token() {
            Some(ExprToken::RParen) => Ok(()),
            other => Err(anyhow!("expected ')', found {:?}", other)),
        }
    }

    fn peek_token(&self) -> Option<&ExprToken> {
        self.tokens.get(self.pos)
    }

    fn next_token(&mut self) -> Option<ExprToken> {
        if self.pos >= self.tokens.len() {
            return None;
        }
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        Some(tok)
    }
}

fn tokenize_expr(input: &str) -> Result<Vec<ExprToken>> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.peek().copied() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        if ch == '"' {
            chars.next();
            let mut value = String::new();
            while let Some(next) = chars.next() {
                if next == '"' {
                    break;
                }
                value.push(next);
            }
            tokens.push(ExprToken::Str(value));
            continue;
        }
        if ch.is_ascii_digit() || ch == '.' {
            let mut value = String::new();
            while let Some(next) = chars.peek().copied() {
                if !next.is_ascii_digit() && next != '.' {
                    break;
                }
                value.push(next);
                chars.next();
            }
            let number: f64 = value.parse().with_context(|| "invalid number")?;
            tokens.push(ExprToken::Number(number));
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            let mut ident = String::new();
            while let Some(next) = chars.peek().copied() {
                if !next.is_ascii_alphanumeric() && next != '_' && next != '.' {
                    break;
                }
                ident.push(next);
                chars.next();
            }
            tokens.push(ExprToken::Ident(ident));
            continue;
        }

        let two = {
            let mut tmp = String::new();
            tmp.push(ch);
            if let Some(next) = chars.clone().nth(1) {
                tmp.push(next);
            }
            tmp
        };
        if matches!(two.as_str(), "&&" | "||" | "==" | "!=" | "<=" | ">=") {
            tokens.push(ExprToken::Op(two));
            chars.next();
            chars.next();
            continue;
        }

        match ch {
            '(' => {
                tokens.push(ExprToken::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(ExprToken::RParen);
                chars.next();
            }
            ',' => {
                tokens.push(ExprToken::Comma);
                chars.next();
            }
            '+' | '-' | '*' | '/' | '<' | '>' | '!' => {
                tokens.push(ExprToken::Op(ch.to_string()));
                chars.next();
            }
            _ => {
                chars.next();
            }
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_block_mixin_refs() {
        let refs = parse_block_mixin_refs(
            r#"
            i18n zh_CN "样例"
            mixin base_bonus
            mixins extra_one, extra_two
            on independent { add_mult 4 }
        "#,
        )
        .expect("parse mixin refs");
        assert_eq!(
            refs,
            vec![
                "base_bonus".to_string(),
                "extra_one".to_string(),
                "extra_two".to_string()
            ]
        );
    }

    #[test]
    fn rejects_empty_mixin_line() {
        let err = parse_block_mixin_refs("mixin   ").expect_err("empty mixin must fail");
        assert!(err.to_string().contains("missing mixin id"));
    }
}
