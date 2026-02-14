use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    None,
    Quit,
    ToggleHelp,
    NextFocus,
    PrevFocus,
    MoveUp,
    MoveDown,
    ToggleSelect,
    ClearSelection,
    Activate,
    Deal,
    PlaySelected,
    DiscardSelected,
    SkipBlind,
    EnterOrLeaveShop,
    RerollShop,
    BuySelected,
    PickSelectedPack,
    SkipPack,
    NextBlind,
    UseConsumable,
    SellJoker,
    SaveState,
    LoadState,
}

pub fn map_key(key: KeyEvent) -> InputAction {
    match key.code {
        KeyCode::Esc => InputAction::ClearSelection,
        KeyCode::Tab => InputAction::NextFocus,
        KeyCode::BackTab => InputAction::PrevFocus,
        KeyCode::Up => InputAction::MoveUp,
        KeyCode::Down => InputAction::MoveDown,
        KeyCode::Enter => InputAction::Activate,
        KeyCode::Char('q') => InputAction::Quit,
        KeyCode::Char('?') => InputAction::ToggleHelp,
        KeyCode::Char(' ') => InputAction::ToggleSelect,
        KeyCode::Char('k') => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                InputAction::SkipBlind
            } else {
                InputAction::MoveUp
            }
        }
        KeyCode::Char('j') => InputAction::MoveDown,
        KeyCode::Char('d') => InputAction::Deal,
        KeyCode::Char('p') => InputAction::PlaySelected,
        KeyCode::Char('x') => InputAction::DiscardSelected,
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            InputAction::SaveState
        }
        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            InputAction::LoadState
        }
        KeyCode::Char('s') => InputAction::EnterOrLeaveShop,
        KeyCode::Char('r') => InputAction::RerollShop,
        KeyCode::Char('b') => InputAction::BuySelected,
        KeyCode::Char('c') => InputAction::PickSelectedPack,
        KeyCode::Char('z') => InputAction::SkipPack,
        KeyCode::Char('n') => InputAction::NextBlind,
        KeyCode::Char('u') => InputAction::UseConsumable,
        KeyCode::Char('v') => InputAction::SellJoker,
        KeyCode::Char('S') => InputAction::SaveState,
        KeyCode::Char('L') => InputAction::LoadState,
        _ => InputAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_basic_actions() {
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE)),
            InputAction::Deal
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE)),
            InputAction::PlaySelected
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
            InputAction::Quit
        );
    }

    #[test]
    fn maps_save_load_shortcuts() {
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)),
            InputAction::SaveState
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL)),
            InputAction::LoadState
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('S'), KeyModifiers::SHIFT)),
            InputAction::SaveState
        );
        assert_eq!(
            map_key(KeyEvent::new(KeyCode::Char('L'), KeyModifiers::SHIFT)),
            InputAction::LoadState
        );
    }
}
