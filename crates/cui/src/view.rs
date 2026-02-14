use crate::app::{App, FocusPane, PathPromptMode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Alignment, Color, Line, Modifier, Style, Stylize};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(14),
            Constraint::Length(10),
        ])
        .split(frame.area());

    draw_header(frame, root[0], app);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(root[1]);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(6)])
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
        draw_help_popup(frame);
    }
    if app.path_prompt_mode.is_some() {
        draw_path_prompt(frame, app);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = format!(
        "Rulatro CUI | Focus: {} | Hint: {}",
        app.focus_label(app.focus),
        app.next_hint()
    );
    let summary = format!(
        "A{} {:?} {:?}  ${}  Score {}/{}  H {}/{}  D {}/{}  Skip {}",
        app.run.state.ante,
        app.run.state.blind,
        app.run.state.phase,
        app.run.state.money,
        app.run.state.blind_score,
        app.run.state.target,
        app.run.state.hands_left,
        app.run.state.hands_max,
        app.run.state.discards_left,
        app.run.state.discards_max,
        app.run.state.blinds_skipped
    );
    let lines = vec![
        Line::from(title.bold()),
        Line::from(summary),
        Line::from(format!("Status: {}", app.status_line)),
    ];
    let block = Block::default().borders(Borders::ALL).title("Overview");
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true }).block(block);
    frame.render_widget(paragraph, area);
}

fn draw_state(frame: &mut Frame, area: Rect, app: &App) {
    let blind_outcome = app
        .run
        .blind_outcome()
        .map(|value| format!("{value:?}"))
        .unwrap_or_else(|| "-".to_string());
    let lines = vec![
        Line::from(format!("Deck draw: {}", app.run.deck.draw.len())),
        Line::from(format!("Deck discard: {}", app.run.deck.discard.len())),
        Line::from(format!(
            "Jokers: {}/{}",
            app.run.inventory.jokers.len(),
            app.run.inventory.joker_capacity()
        )),
        Line::from(format!(
            "Consumables: {}/{}",
            app.run.inventory.consumable_count(),
            app.run.inventory.consumable_slots
        )),
        Line::from(format!("Outcome: {blind_outcome}")),
    ];
    let block = Block::default().borders(Borders::ALL).title("Run");
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_hand(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem<'_>> = if app.run.hand.is_empty() {
        vec![ListItem::new("empty")]
    } else {
        app.run
            .hand
            .iter()
            .enumerate()
            .map(|(idx, card)| ListItem::new(app.card_label(idx, card)))
            .collect()
    };
    let block = pane_block("Hand", app.focus == FocusPane::Hand);
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
            vec![ListItem::new("empty pack options")]
        } else {
            open.options
                .iter()
                .enumerate()
                .map(|(idx, option)| ListItem::new(app.pack_option_label(idx, option)))
                .collect()
        };
        let pack_title = format!(
            "Pack {:?}/{:?} picks {}",
            open.offer.kind, open.offer.size, open.offer.picks
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
        let block = pane_block("Shop", app.focus == FocusPane::Shop);
        let text = if app.run.state.phase == rulatro_core::Phase::Shop {
            "no shop offers"
        } else {
            "shop unavailable"
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
        "Shop reroll ${}",
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
        vec![ListItem::new("empty")]
    } else {
        rows.iter()
            .map(|row| ListItem::new(row.label.clone()))
            .collect()
    };
    let block = pane_block("Inventory", app.focus == FocusPane::Inventory);
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
    let block = pane_block("Events", app.focus == FocusPane::Events);
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_help_popup(frame: &mut Frame) {
    let area = centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, area);
    let lines = vec![
        Line::from("q quit | ? help | tab focus | arrows/jk move"),
        Line::from("space toggle select | enter context action"),
        Line::from("d deal | p play | x discard | Shift+k skip blind"),
        Line::from("s enter/leave shop | b buy | r reroll"),
        Line::from("c pick pack | z skip pack"),
        Line::from("u use consumable | v sell joker | n next blind"),
        Line::from("Shift+S/Ctrl+S save | Shift+L/Ctrl+L load"),
    ];
    let block = Block::default()
        .title("Help")
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
        PathPromptMode::Save => "Save Path",
        PathPromptMode::Load => "Load Path",
    };
    let action_hint = match mode {
        PathPromptMode::Save => "Enter=save  Esc=cancel",
        PathPromptMode::Load => "Enter=load  Esc=cancel",
    };
    let lines = vec![
        Line::from(action_hint),
        Line::from("Leave empty to use default path:"),
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
