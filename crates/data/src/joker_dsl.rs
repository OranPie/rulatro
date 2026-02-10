use anyhow::{anyhow, bail, Context, Result};
use rulatro_core::{
    Action, ActionOp, ActivationType, BinaryOp, Expr, JokerDef, JokerEffect, JokerRarity,
    UnaryOp,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn load_jokers_dsl(path: &Path) -> Result<Vec<JokerDef>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let expanded = expand_templates(&raw)?;
    parse_jokers(&expanded)
}

#[derive(Debug, Clone)]
struct Template {
    params: Vec<String>,
    body: String,
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
    let name_end = rest.find('(').ok_or_else(|| anyhow!("template missing '('"))?;
    let name = rest[..name_end].trim().to_string();
    let rest = &rest[name_end + 1..];
    let params_end = rest.find(')').ok_or_else(|| anyhow!("template missing ')'"))?;
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

fn parse_jokers(src: &str) -> Result<Vec<JokerDef>> {
    let mut jokers = Vec::new();
    let mut lines = src.lines().peekable();

    while let Some(line) = lines.next() {
        let line = strip_comments(line);
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.starts_with("joker ") {
            continue;
        }

        let (id, name, rarity, remainder) = parse_joker_header(trimmed)?;
        let mut body = String::new();
        let mut depth = 1;
        if !remainder.is_empty() {
            let (chunk, new_depth, done) = consume_until_close(&remainder, depth);
            body.push_str(chunk.trim_end());
            body.push('\n');
            depth = new_depth;
            if done {
                let effects = parse_effects(&body)?;
                jokers.push(JokerDef {
                    id,
                    name,
                    rarity,
                    effects,
                });
                continue;
            }
        }

        while depth > 0 {
            let next_line = lines
                .next()
                .ok_or_else(|| anyhow!("unterminated joker block"))?;
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

        let effects = parse_effects(&body)?;
        jokers.push(JokerDef {
            id,
            name,
            rarity,
            effects,
        });
    }

    Ok(jokers)
}

fn parse_joker_header(line: &str) -> Result<(String, String, JokerRarity, String)> {
    let parts: Vec<&str> = line.splitn(2, '{').collect();
    let header = parts
        .get(0)
        .map(|s| s.trim())
        .ok_or_else(|| anyhow!("invalid joker header"))?;
    let remainder = parts.get(1).map(|s| s.to_string()).unwrap_or_default();

    let tokens = tokenize_simple(header)?;
    if tokens.is_empty() || tokens[0] != Token::Ident("joker".to_string()) {
        bail!("joker header must start with 'joker'");
    }
    let mut idx = 1;
    let id = token_to_string(tokens.get(idx))
        .ok_or_else(|| anyhow!("joker id missing"))?;
    idx += 1;
    let name = token_to_string(tokens.get(idx))
        .ok_or_else(|| anyhow!("joker name missing"))?;
    idx += 1;
    let rarity_str = token_to_string(tokens.get(idx))
        .ok_or_else(|| anyhow!("joker rarity missing"))?;
    let rarity = parse_rarity(&rarity_str)?;

    Ok((id, name, rarity, remainder))
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
        let op = ActionOp::from_keyword(op_str)
            .ok_or_else(|| anyhow!("unknown action '{}'", op_str))?;
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
        "scored" => Ok(ActivationType::OnScored),
        "held" => Ok(ActivationType::OnHeld),
        "independent" => Ok(ActivationType::Independent),
        "discard" | "discarded" => Ok(ActivationType::OnDiscard),
        "discard_batch" | "discarded_batch" | "discard_group" => Ok(ActivationType::OnDiscardBatch),
        "destroyed" | "card_destroyed" | "carddestroyed" => Ok(ActivationType::OnCardDestroyed),
        "round_end" | "roundend" => Ok(ActivationType::OnRoundEnd),
        "blind_start" | "blindstart" | "blind_selected" | "blindselect" => Ok(ActivationType::OnBlindStart),
        "shop_enter" | "shopenter" | "shop_start" | "shopstart" => Ok(ActivationType::OnShopEnter),
        "shop_reroll" | "shopreroll" => Ok(ActivationType::OnShopReroll),
        "pack_opened" | "packopen" | "pack_open" | "booster_opened" => Ok(ActivationType::OnPackOpened),
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
