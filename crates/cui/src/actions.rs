use crate::app::App;
use crate::input::InputAction;

pub fn dispatch(app: &mut App, action: InputAction) {
    match action {
        InputAction::None => {}
        InputAction::Quit => app.should_quit = true,
        InputAction::ToggleHelp => app.show_help = !app.show_help,
        InputAction::NextFocus => app.cycle_focus(true),
        InputAction::PrevFocus => app.cycle_focus(false),
        InputAction::MoveUp => app.move_cursor(false),
        InputAction::MoveDown => app.move_cursor(true),
        InputAction::ToggleSelect => app.toggle_focused_selection(),
        InputAction::ClearSelection => {
            if app.show_help {
                app.show_help = false;
            } else {
                app.clear_selection();
            }
        }
        InputAction::Activate => app.activate_primary(),
        InputAction::Deal => app.deal(),
        InputAction::PlaySelected => app.play_selected(),
        InputAction::DiscardSelected => app.discard_selected(),
        InputAction::SkipBlind => app.skip_blind(),
        InputAction::EnterOrLeaveShop => app.enter_or_leave_shop(),
        InputAction::RerollShop => app.reroll_shop(),
        InputAction::BuySelected => app.buy_selected_offer(),
        InputAction::PickSelectedPack => app.pick_pack_selected(),
        InputAction::SkipPack => app.skip_pack(),
        InputAction::NextBlind => app.next_blind(),
        InputAction::UseConsumable => app.use_selected_consumable(),
        InputAction::SellJoker => app.sell_selected_joker(),
        InputAction::SaveState => app.open_save_prompt(),
        InputAction::LoadState => app.open_load_prompt(),
    }
}
