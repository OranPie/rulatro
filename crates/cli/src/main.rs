use rulatro_core::{BlindKind, BlindOutcome, EventBus, RunState, ShopOfferRef};
use rulatro_data::{load_content_with_mods, load_game_config};
use rulatro_modding::ModManager;
use std::path::Path;

fn main() {
    let mut events = EventBus::default();
    let config = load_game_config(Path::new("assets")).expect("load config");

    let modded = load_content_with_mods(Path::new("assets"), Path::new("mods"))
        .expect("load content");
    if !modded.mods.is_empty() {
        println!("mods loaded: {}", modded.mods.len());
        for item in &modded.mods {
            println!("mod: {}", item.manifest.meta.id);
        }
    }
    for warning in &modded.warnings {
        eprintln!("mod warning: {}", warning);
    }
    let mut runtime = ModManager::new();
    runtime
        .load_mods(&modded.mods)
        .expect("load mod runtime");
    let mut run = RunState::new(config, modded.content, 0xC0FFEE);
    run.set_mod_runtime(Some(Box::new(runtime)));
    run.start_blind(1, BlindKind::Small, &mut events)
        .expect("start blind");

    let mut blinds_completed = 0;
    loop {
        run.prepare_hand(&mut events).expect("prepare hand");

        let play_count = run.hand.len().min(5);
        let indices: Vec<usize> = (0..play_count).collect();
        let breakdown = run
            .play_hand(&indices, &mut events)
            .expect("play hand");

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
