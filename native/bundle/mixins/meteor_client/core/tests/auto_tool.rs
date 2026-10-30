// SPDX-License-Identifier: GPL-3.0-only
use meteor_client::{Config,modules::auto_tool::*};
fn tool(speed:f32)->Tool { Tool {is_tool:true,correct_drops:true,speed,max_damage:100,..Tool::default()} }
#[test] fn defaults_and_transactional_source_settings() {
 let mut config=Config::default(); assert_eq!(config.auto_tool.prefer,Preference::Fortune); assert!(config.auto_tool.silk_touch_for_ender_chest);
 assert!(!config.auto_tool.switch_back); assert_eq!(config.auto_tool.list_mode,ListMode::Blacklist);
 config.set("auto-tool","anti-break-percentage","1").unwrap(); assert!(config.set("auto-tool","anti-break-percentage","0").is_err()); assert_eq!(config.auto_tool.anti_break_percentage,1);
 config.set("auto-tool","switch-delay","-3").unwrap(); assert_eq!(config.auto_tool.switch_delay,-3);
 assert!(Config::from_json(&config.to_json().unwrap()).unwrap().modules.modules.iter().any(|m|m.id=="auto-tool"));
}
#[test] fn immediate_swap_remembers_even_when_switchback_disabled_and_first_equal_wins() {
 let settings=Settings::default(); let mut state=State::default();
 assert_eq!(state.start(&settings,&tool(1.0),Block::default(),[(1,tool(4.0),true),(2,tool(4.0),true)]),TickAction::Swap{slot:1,remember:true});
 assert_eq!(state.start(&settings,&tool(4.0),Block::default(),[(1,tool(4.0),true)]),TickAction::None);
}
#[test] fn anti_break_replacement_or_cancel_and_missing_candidate_preserves_invalid_swap() {
 let settings=Settings {anti_break:true,..Settings::default()}; let mut state=State::default(); let bad=Tool{damage:95,..tool(8.0)};
 assert!(settings.stopped(&bad));
 assert_eq!(state.start(&settings,&bad,Block::default(),[(0,bad,true),(3,tool(2.0),true)]),TickAction::Swap{slot:3,remember:true});
 assert_eq!(state.start(&settings,&bad,Block::default(),[(0,bad,true)]),TickAction::Swap{slot:-1,remember:true});
}
#[test] fn lists_filter_candidates_but_do_not_filter_current_tool_comparison() {
 let settings=Settings{list_mode:ListMode::Whitelist,whitelist:vec!["minecraft:shears".into()],..Settings::default()};
 assert!(settings.allows("minecraft:shears")); assert!(!settings.allows("minecraft:diamond_pickaxe"));
 let mut state=State::default(); assert_eq!(state.start(&settings,&tool(8.0),Block::default(),[(0,tool(4.0),true),(1,tool(9.0),false)]),TickAction::None);
}
#[test] fn delayed_switch_source_ticks_and_pending_state_survive_later_immediate_start() {
 let mut settings=Settings{switch_delay:2,..Settings::default()}; let mut state=State::default();
 assert_eq!(state.start(&settings,&tool(1.0),Block::default(),[(4,tool(5.0),true)]),TickAction::None);
 assert_eq!(state.post(false,false,true,None),TickAction::None); assert_eq!(state.post(false,false,true,None),TickAction::None);
 assert_eq!(state.post(false,false,true,None),TickAction::Swap{slot:4,remember:false});
 settings.switch_delay=3; state.start(&settings,&tool(1.0),Block::default(),[(5,tool(5.0),true)]);
 settings.switch_delay=0; state.start(&settings,&tool(1.0),Block::default(),[(6,tool(5.0),true)]);
 assert!(state.should_switch); assert_eq!(state.post(false,false,false,None),TickAction::Swap{slot:6,remember:false});
}
#[test] fn shared_previous_slot_survives_auto_tool_toggle_then_false_swap_clears_it() {
 let mut swap=Swaps::default(); assert_eq!(swap.swap(1,4,true),Some(4));
 // No deactivate reset exists in Java; this shared object outlives module subscriptions.
 let mut state=State{was_pressed:true,..State::default()};
 assert_eq!(state.post(false,true,false,swap.previous),TickAction::SwapBack);
 assert_eq!(swap.back(4),Some(1)); assert_eq!(swap.swap(1,5,true),Some(5)); assert_eq!(swap.swap(5,6,false),Some(6)); assert_eq!(swap.previous,None);
}

#[test] fn source_itemlist_filter_drops_known_non_tools_and_retains_valid_tool_order() {
 let mut settings=Settings {whitelist:vec!["minecraft:stone".into(),"minecraft:diamond_pickaxe".into(),"minecraft:stick".into(),"minecraft:shears".into()],blacklist:vec!["minecraft:stick".into(),"minecraft:shears".into()],..Settings::default()};
 settings.filter_items(|id|matches!(id,"minecraft:diamond_pickaxe"|"minecraft:shears"));
 assert_eq!(settings.whitelist,["minecraft:diamond_pickaxe","minecraft:shears"]);assert_eq!(settings.blacklist,["minecraft:shears"]);
}
