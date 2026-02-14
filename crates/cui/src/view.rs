use crate::app::{App, FocusPane, PathPromptMode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Alignment, Color, Line, Modifier, Style, Stylize};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(12),
            Constraint::Length(12),
        ])
        .split(frame.area());

    draw_header(frame, root[0], app);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(root[1]);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(4)])
        .split(middle[0]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Min(8)])
        .split(middle[1]);

    draw_state(frame, left[0], app);
    draw_hand(frame, left[1], app);
    draw_shop_or_pack(frame, right[0], app);
    draw_inventory(frame, right[1], app);
    draw_events(frame, root[2], app);

    if app.show_help {
        draw_help_popup(frame, app);
    }
    if app.path_prompt_mode.is_some() {
        draw_path_prompt(frame, app);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = format!(
        "{} | {}: {} | {}: {}",
        app.locale.text("Rulatro CUI", "Rulatro CUI"),
        app.locale.text("Focus", "焦点"),
        app.focus_label(app.focus),
        app.locale.text("Hint", "提示"),
        app.next_hint()
    );
    let summary = format!(
        "A{} {} {}  ${}  {} {}/{}  H {}/{}  D {}/{}  {} {}",
        app.run.state.ante,
        app.blind_label(app.run.state.blind),
        app.phase_label(app.run.state.phase),
        app.run.state.money,
        app.locale.text("Score", "分数"),
        app.run.state.blind_score,
        app.run.state.target,
        app.run.state.hands_left,
        app.run.state.hands_max,
        app.run.state.discards_left,
        app.run.state.discards_max,
        app.locale.text("Skip", "跳过"),
        app.run.state.blinds_skipped
    );
    let extra = format!(
        "{} {} | {} {} | {} {}",
        app.locale.text("Seed", "种子"),
        app.seed,
        app.locale.text("Lang", "语言"),
        app.locale.code(),
        app.locale.text("Outcome", "结果"),
        app.blind_outcome_label()
    );
    let lines = vec![
        Line::from(title.bold()),
        Line::from(summary),
        Line::from(extra),
        Line::from(format!(
            "{}: {}",
            app.locale.text("Status", "状态"),
            app.status_line
        )),
    ];
    let block = Block::default()
        .borders(Borders::ALL)
        .title(app.locale.text("Overview", "概览"));
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true }).block(block);
    frame.render_widget(paragraph, area);
}

fn draw_state(frame: &mut Frame, area: Rect, app: &App) {
    let content_sig_short: String = app.content_signature.chars().take(8).collect();
    let mut lines = vec![
        Line::from(format!(
            "{}: {}",
            app.locale.text("Deck draw", "抽牌堆"),
            app.run.deck.draw.len()
        )),
        Line::from(format!(
            "{}: {}",
            app.locale.text("Deck discard", "弃牌堆"),
            app.run.deck.discard.len()
        )),
        Line::from(format!(
            "{}: {}/{}",
            app.locale.text("Jokers", "小丑"),
            app.run.inventory.jokers.len(),
            app.run.inventory.joker_capacity()
        )),
        Line::from(format!(
            "{}: {}/{}",
            app.locale.text("Consumables", "消耗牌"),
            app.run.inventory.consumable_count(),
            app.run.inventory.consumable_slots
        )),
        Line::from(format!(
            "{}: {}",
            app.locale.text("Tags", "标签"),
            app.run.state.tags.len()
        )),
        Line::from(format!(
            "{}: {}",
            app.locale.text("Content Sig", "内容签名"),
            if content_sig_short.is_empty() {
                "-".to_string()
            } else {
                content_sig_short
            }
        )),
        Line::from(format!(
            "{}: {}",
            app.locale.text("Outcome", "结果"),
            app.blind_outcome_label()
        )),
    ];
    lines.push(Line::from(format!(
        "{}: {}",
        app.locale.text("Boss", "Boss"),
        app.boss_status_label()
    )));
    for effect in app.boss_effect_lines(2) {
        lines.push(Line::from(format!(
            "  {} {}",
            app.locale.text("effect", "效果"),
            effect
        )));
    }
    for voucher in app.active_voucher_lines(2) {
        lines.push(Line::from(format!(
            "  {} {}",
            app.locale.text("voucher", "优惠券"),
            voucher
        )));
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .title(app.locale.text("Run", "对局"));
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_hand(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem<'_>> = if app.run.hand.is_empty() {
        vec![ListItem::new(app.locale.text("empty", "空"))]
    } else {
        app.run
            .hand
            .iter()
            .enumerate()
            .map(|(idx, card)| ListItem::new(app.card_label(idx, card)))
            .collect()
    };
    let block = pane_block(
        app.locale.text("Hand", "手牌"),
        app.focus == FocusPane::Hand,
    );
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    let mut state = ListState::default();
    if !app.run.hand.is_empty() {
        state.select(Some(app.hand_cursor.min(app.run.hand.len() - 1)));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_shop_or_pack(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(open) = app.open_pack.as_ref() {
        let items: Vec<ListItem<'_>> = if open.options.is_empty() {
            vec![ListItem::new(
                app.locale.text("empty pack options", "卡包选项为空"),
            )]
        } else {
            open.options
                .iter()
                .enumerate()
                .map(|(idx, option)| ListItem::new(app.pack_option_label(idx, option)))
                .collect()
        };
        let pack_title = format!(
            "{} {:?}/{:?} {} {}",
            app.locale.text("Pack", "卡包"),
            open.offer.kind,
            open.offer.size,
            app.locale.text("picks", "可选"),
            open.offer.picks
        );
        let block = pane_block(pack_title.as_str(), app.focus == FocusPane::Shop);
        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
        let mut state = ListState::default();
        if !open.options.is_empty() {
            state.select(Some(app.pack_cursor.min(open.options.len() - 1)));
        }
        frame.render_stateful_widget(list, area, &mut state);
        return;
    }

    let rows = app.shop_rows();
    if rows.is_empty() {
        let block = pane_block(
            app.locale.text("Shop", "商店"),
            app.focus == FocusPane::Shop,
        );
        let text = if app.run.state.phase == rulatro_core::Phase::Shop {
            app.locale.text("no shop offers", "商店无商品")
        } else {
            app.locale.text("shop unavailable", "商店不可用")
        };
        frame.render_widget(
            Paragraph::new(text)
                .alignment(Alignment::Center)
                .block(block),
            area,
        );
        return;
    }

    let items: Vec<ListItem<'_>> = rows
        .iter()
        .map(|row| ListItem::new(row.label.clone()))
        .collect();
    let shop_title = format!(
        "{} {} ${}",
        app.locale.text("Shop", "商店"),
        app.locale.text("reroll", "刷新"),
        app.run
            .shop
            .as_ref()
            .map(|shop| shop.reroll_cost)
            .unwrap_or(0)
    );
    let block = pane_block(shop_title.as_str(), app.focus == FocusPane::Shop);
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    let mut state = ListState::default();
    state.select(Some(app.shop_cursor.min(rows.len() - 1)));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_inventory(frame: &mut Frame, area: Rect, app: &App) {
    let rows = app.inventory_rows();
    let items: Vec<ListItem<'_>> = if rows.is_empty() {
        vec![ListItem::new(app.locale.text("empty", "空"))]
    } else {
        rows.iter()
            .map(|row| ListItem::new(row.label.clone()))
            .collect()
    };
    let block = pane_block(
        app.locale.text("Inventory", "库存"),
        app.focus == FocusPane::Inventory,
    );
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    let mut state = ListState::default();
    if !rows.is_empty() {
        state.select(Some(app.inventory_cursor.min(rows.len() - 1)));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_events(frame: &mut Frame, area: Rect, app: &App) {
    let capacity = area.height.saturating_sub(2) as usize;
    let start = app.event_log.len().saturating_sub(capacity);
    let lines: Vec<Line<'_>> = app
        .event_log
        .iter()
        .skip(start)
        .map(|line| Line::from(line.clone()))
        .collect();
    let block = pane_block(
        app.locale.text("Events", "事件"),
        app.focus == FocusPane::Events,
    );
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_help_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, area);
    let lines = vec![
        Line::from(app.locale.text(
            "q quit | ? help | tab focus | arrows/jk move",
            "q 退出 | ? 帮助 | tab 切焦点 | 方向键/jk 移动",
        )),
        Line::from(app.locale.text(
            "space toggle select | enter context action",
            "空格 选中/取消 | 回车 执行上下文动作",
        )),
        Line::from(app.locale.text(
            "0-9 quick select by index (focus pane)",
            "0-9 按索引快速选择（当前焦点）",
        )),
        Line::from(app.locale.text(
            "d deal | p play | x discard | Shift+k skip blind",
            "d 发牌 | p 出牌 | x 弃牌 | Shift+k 跳过盲注",
        )),
        Line::from(app.locale.text(
            "s enter/leave shop | b buy | r reroll",
            "s 进/离商店 | b 购买 | r 刷新",
        )),
        Line::from(
            app.locale
                .text("c pick pack | z skip pack", "c 选卡包 | z 跳过卡包"),
        ),
        Line::from(app.locale.text(
            "u use consumable | v sell joker | n next blind",
            "u 使用消耗牌 | v 卖小丑 | n 下一盲注",
        )),
        Line::from(app.locale.text(
            "Shift+S/Ctrl+S save | Shift+L/Ctrl+L load",
            "Shift+S/Ctrl+S 保存 | Shift+L/Ctrl+L 读取",
        )),
        Line::from(app.locale.text(
            "play now writes detailed effect trace to Events",
            "出牌后会在事件面板写入详细效果轨迹",
        )),
    ];
    let block = Block::default()
        .title(app.locale.text("Help", "帮助"))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_path_prompt(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 28, frame.area());
    frame.render_widget(Clear, area);
    let mode = app.path_prompt_mode.expect("checked above");
    let title = match mode {
        PathPromptMode::Save => app.locale.text("Save Path", "保存路径"),
        PathPromptMode::Load => app.locale.text("Load Path", "读取路径"),
    };
    let action_hint = match mode {
        PathPromptMode::Save => app
            .locale
            .text("Enter=save  Esc=cancel", "回车=保存  Esc=取消"),
        PathPromptMode::Load => app
            .locale
            .text("Enter=load  Esc=cancel", "回车=读取  Esc=取消"),
    };
    let lines = vec![
        Line::from(action_hint),
        Line::from(
            app.locale
                .text("Leave empty to use default path:", "留空将使用默认路径："),
        ),
        Line::from(format!("  {}", app.prompt_default_path_hint())),
        Line::from(""),
        Line::from(format!("> {}", app.path_prompt_input)),
    ];
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn pane_block(title: &str, focused: bool) -> Block<'_> {
    let mut block = Block::default().title(title).borders(Borders::ALL);
    if focused {
        block = block.border_style(Style::default().fg(Color::Yellow));
    }
    block
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
