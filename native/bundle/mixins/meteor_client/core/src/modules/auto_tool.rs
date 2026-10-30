// SPDX-License-Identifier: GPL-3.0-only
// AutoTool.java score policies. Copyright (c) Meteor Development.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preference { None, Fortune, SilkTouch }
#[derive(Clone, Copy, Debug, Default)]
pub struct Tool {
    pub is_tool: bool, pub correct_drops: bool, pub sword: bool, pub shears: bool,
    pub speed: f32, pub component_speed: f32, pub max_damage: i32, pub damage: i32,
    pub unbreaking: i32, pub efficiency: i32, pub mending: i32, pub fortune: i32, pub silk: i32,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Block { pub bamboo: bool, pub leaves: bool, pub wool: bool, pub ender_chest: bool, pub fortunable: bool }
pub fn stop(tool: &Tool, anti_break: bool, percent: i32) -> bool {
    anti_break && tool.max_damage.wrapping_sub(tool.damage)
        < tool.max_damage.wrapping_mul(percent) / 100
}
pub fn score(tool: &Tool, block: Block, good: bool, silk_chests: bool, fortune_ores: bool, prefer: Preference) -> f64 {
    if !good || !tool.is_tool { return -1.0; }
    // Preserve Java's parentheses: wool is allowed independently of ShearsItem.
    if !tool.correct_drops && !(tool.sword && block.bamboo) && !(tool.shears && block.leaves || block.wool) { return -1.0; }
    if silk_chests && block.ender_chest && tool.silk == 0 { return -1.0; }
    if fortune_ores && block.fortunable && tool.fortune == 0 { return -1.0; }
    let mut value = f64::from(tool.speed * 1000.0_f32);
    value += f64::from(tool.unbreaking);
    value += f64::from(tool.efficiency);
    value += f64::from(tool.mending);
    if prefer == Preference::Fortune { value += f64::from(tool.fortune); }
    if prefer == Preference::SilkTouch { value += f64::from(tool.silk); }
    if tool.sword && block.bamboo { value += f64::from(9000.0_f32 + tool.component_speed * 1000.0_f32); }
    value
}
pub fn best(scores: impl IntoIterator<Item=(usize,f64)>) -> Option<(usize,f64)> {
    let mut best = None;
    for (slot, score) in scores {
        if score >= 0.0 && best.is_none_or(|(_, old)| score > old) { best = Some((slot,score)); }
    }
    best
}
/// Shared InvUtils.previousSlot, not an AutoTool-local restoration slot.
#[derive(Default, Debug)]
pub struct Swaps { pub previous: Option<usize> }
impl Swaps {
    pub fn swap(&mut self, current: usize, requested: i32, remember: bool) -> Option<usize> {
        if requested == 40 { return Some(current); } // SlotUtils.OFFHAND: successful no-op
        if !(0..=8).contains(&requested) { return None; }
        if remember && self.previous.is_none() { self.previous = Some(current); }
        else if !remember { self.previous = None; }
        Some(requested as usize)
    }
    pub fn back(&mut self, current: usize) -> Option<usize> {
        let old = self.previous?;
        let result = self.swap(current, old as i32, false);
        self.previous = None;
        result
    }
}
#[derive(Default, Debug)]
pub struct State { pub was_pressed: bool, pub should_switch: bool, pub ticks: i32, pub best_slot: i32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TickAction { None, SwapBack, Swap { slot:i32, remember:bool } }
impl State {
    pub fn post(&mut self, infinity_miner: bool, switch_back: bool, attack: bool, previous: Option<usize>) -> TickAction {
        if infinity_miner { return TickAction::None; }
        if switch_back && !attack && self.was_pressed && previous.is_some() {
            self.was_pressed = false;
            return TickAction::SwapBack;
        }
        let action = if self.ticks <= 0 && self.should_switch && self.best_slot != -1 {
            self.should_switch = false;
            TickAction::Swap { slot:self.best_slot, remember:switch_back }
        } else {
            self.ticks = self.ticks.wrapping_sub(1);
            TickAction::None
        };
        self.was_pressed = attack;
        action
    }
}
#[cfg(test)] mod tests {
    use super::*;
    fn tool() -> Tool { Tool { is_tool:true,correct_drops:true,speed:8.0,max_damage:100, ..Tool::default() } }
    #[test] fn durability_threshold_is_strict_integer_and_wraps_like_java() {
        let t = Tool { damage:90, ..tool() };
        assert!(!stop(&t,true,10));
        assert!(stop(&Tool { damage:91,..t },true,10));
        assert!(!stop(&Tool { max_damage:9,damage:9,..t },true,10));
        assert!(!stop(&Tool { max_damage:i32::MAX,damage:i32::MAX-1,..t },true,10));
    }
    #[test] fn wool_exception_is_not_restricted_to_shears() {
        let t = Tool { correct_drops:false,..tool() };
        assert_eq!(score(&t,Block::default(),true,true,false,Preference::None),-1.0);
        assert_eq!(score(&t,Block { wool:true,..Block::default() },true,true,false,Preference::None),8000.0);
    }
    #[test] fn enchantment_gates_and_preferences_are_independent() {
        let t = Tool { fortune:3,silk:1,unbreaking:3,efficiency:5,mending:1,..tool() };
        assert_eq!(score(&t,Block::default(),true,true,false,Preference::Fortune),8012.0);
        assert_eq!(score(&t,Block::default(),true,true,false,Preference::SilkTouch),8010.0);
        assert_eq!(score(&Tool { silk:0,..t },Block { ender_chest:true,..Block::default() },true,true,false,Preference::None),-1.0);
        assert_eq!(score(&Tool { fortune:0,..t },Block { fortunable:true,..Block::default() },true,false,true,Preference::None),-1.0);
    }
    #[test] fn tool_gate_precedes_bamboo_sword_exception() {
        let sword = Tool { sword:true,speed:1.0,component_speed:15.0,..Tool::default() };
        let bamboo = Block { bamboo:true,..Block::default() };
        assert_eq!(score(&sword,bamboo,true,true,false,Preference::None),-1.0);
        // Custom data can put a sword in a supported tool tag.
        assert_eq!(score(&Tool { is_tool:true,..sword },bamboo,true,true,false,Preference::None),25000.0);
    }
    #[test] fn first_tie_wins_and_invalid_candidates_do_not_win() {
        assert_eq!(best([(0,-1.0),(1,8000.0),(2,8000.0),(3,8001.0)]),Some((3,8001.0)));
        assert_eq!(best([(0,-1.0),(1,f64::NAN)]),None);
        assert_eq!(best([(4,0.0),(5,0.0)]),Some((4,0.0)));
    }
    #[test] fn shared_swaps_remember_first_slot_and_invalid_noop_preserves_it() {
        let mut swaps = Swaps::default();
        assert_eq!(swaps.swap(2,5,true),Some(5));
        assert_eq!(swaps.swap(5,7,true),Some(7));
        assert_eq!(swaps.previous,Some(2));
        assert_eq!(swaps.swap(7,-1,false),None);
        assert_eq!(swaps.swap(7,40,false),Some(7));
        assert_eq!(swaps.previous,Some(2));
        assert_eq!(swaps.back(7),Some(2));
        assert_eq!(swaps.previous,None);
    }
    #[test] fn released_attack_restore_precedes_pending_switch_and_preserves_timer() {
        let mut state = State { was_pressed:true,should_switch:true,ticks:3,best_slot:4 };
        assert_eq!(state.post(false,true,false,Some(1)),TickAction::SwapBack);
        assert_eq!(state.ticks,3);
        assert!(state.should_switch);
        state.post(false,true,false,None);
        assert_eq!(state.ticks,2);
    }
    #[test] fn delayed_swap_and_disabled_infinity_keep_source_state() {
        let mut state = State { was_pressed:true,should_switch:true,ticks:1,best_slot:4 };
        assert_eq!(state.post(true,false,false,None),TickAction::None);
        assert_eq!(state.ticks,1);
        assert!(state.was_pressed);
        assert_eq!(state.post(false,false,true,None),TickAction::None);
        assert_eq!(state.post(false,false,true,None),TickAction::Swap { slot:4,remember:false });
        assert!(!state.should_switch);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListMode { Whitelist, Blacklist }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub prefer: Preference,
    pub silk_touch_for_ender_chest: bool,
    pub fortune_for_ores_and_crops: bool,
    pub anti_break: bool,
    pub anti_break_percentage: i32,
    pub switch_back: bool,
    pub switch_delay: i32,
    pub list_mode: ListMode,
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
}
impl Default for Settings {
    fn default() -> Self { Self { prefer:Preference::Fortune,silk_touch_for_ender_chest:true,fortune_for_ores_and_crops:false,anti_break:false,anti_break_percentage:10,switch_back:false,switch_delay:0,list_mode:ListMode::Blacklist,whitelist:vec![],blacklist:vec![] } }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=100).contains(&self.anti_break_percentage) { return Err("anti-break-percentage must be between 1 and 100".into()); }
        // IntSetting has no range on switch-delay: negative values are source-valid.
        for list in [&self.whitelist,&self.blacklist] { for id in list { if id.is_empty() || !id.contains(':') { return Err("Expected a namespaced item id".into()); } } }
        Ok(())
    }
    /// ItemListSetting's filter is applied on parsing/loading, preserving accepted order.
    pub fn filter_items(&mut self, is_tool: impl Fn(&str)->bool) {
        self.whitelist.retain(|id|is_tool(id));
        self.blacklist.retain(|id|is_tool(id));
    }
    pub fn allows(&self, item: &str) -> bool { let list=if self.list_mode==ListMode::Whitelist { &self.whitelist } else { &self.blacklist }; list.iter().any(|id|id==item)==(self.list_mode==ListMode::Whitelist) }
    pub fn stopped(&self, tool:&Tool)->bool { stop(tool,self.anti_break,self.anti_break_percentage) }
    pub fn score(&self, tool:&Tool, block:Block)->f64 { score(tool,block,!self.stopped(tool),self.silk_touch_for_ender_chest,self.fortune_for_ores_and_crops,self.prefer) }
}
impl State {
    /// Source HIGH StartBreakingBlock handler up to its post-swap anti-break check.
    /// The host checks cancellation using the resulting held stack after an immediate swap.
    pub fn start(&mut self, settings:&Settings, current:&Tool, block:Block, candidates:impl IntoIterator<Item=(usize,Tool,bool)>) -> TickAction {
        let chosen=best(candidates.into_iter().filter(|(_,_,allowed)|*allowed).map(|(slot,t,_)|(slot,settings.score(&t,block))));
        self.best_slot=chosen.map_or(-1,|(slot,_)|slot as i32);
        if chosen.is_some_and(|(_,score)|score>settings.score(current,block)) || settings.stopped(current) || !current.is_tool {
            self.ticks=settings.switch_delay;
            if self.ticks==0 { return TickAction::Swap { slot:self.best_slot,remember:true }; }
            self.should_switch=true;
        }
        TickAction::None
    }
}
