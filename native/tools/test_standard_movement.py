#!/usr/bin/env python3
"""Combined normal .hnmod movement runtime harness; launch only when executed.

Runs the unchanged AutoJump harness regression, then isolated AirJump/HighJump
checks using actual chat/input, authoritative fixture commands and public state.
"""
from __future__ import annotations
import argparse
import json
import math
from pathlib import Path
import re
import time

import test_standard_autojump as autojump_harness
zoom_harness = autojump_harness.zoom_harness

AIR = re.compile(r"AirJump (enabled|disabled) \(maintain-level (true|false), captured level (None|Some\((-?\d+)\))\)")
HIGH = re.compile(r"HighJump (enabled|disabled) \(multiplier ([^)]+)\)")
ORDER = re.compile(r"Pre order (\[[^\]]*\])")
DEFAULT_PREFIX = "Zoom+AutoJump+AirJump+HighJump .hnmod migration candidate:"


class MovementHarness(autojump_harness.AutoJumpHarness):
    def __init__(self, args):
        super().__init__(args)
        self.candidate_prefix = args.candidate_prefix
        self.error_prefix = self.candidate_prefix.replace(".hnmod ", "")
        zoom_harness.PREFIXES = (self.candidate_prefix, self.error_prefix)
        self.in_regression = False
        with (self.home / "options.txt").open("a", encoding="utf8") as stream:
            stream.write("key_key.jump:key.keyboard.j\n")
        self.report["kind"] = "normal-hnmod-zoom-autojump-airjump-highjump-candidate"
        self.report["candidate_prefix"] = self.candidate_prefix
        self.script_paths = {"movement":Path(__file__).resolve(),
                             "autojump":Path(autojump_harness.__file__).resolve(),
                             "zoom":Path(zoom_harness.__file__).resolve()}
        self.report["movement_harness_hashes"] = {k:zoom_harness.sha256(v) for k,v in self.script_paths.items()}
        self.report["limitations"] = [
            "Sampled heights/velocities are observations, not exact mathematical apex values.",
            "No direct control exposes per-jump needs_sync/exhaustion; saved native jump counters are collected separately when available without client-wrapper attribution.",
            "Continuing horizontal motion and native identity tests do not prove exact statistic/exhaustion counts.",
            "Source AutoJump has no independently invented GUI guard.",
            "Only generated worlds/settings and this process's private control socket are used.",
            "AirJump exercises rebound keyboard J; mouse jump bindings are not exercised.",
        ]
        self.report["regression_prefix_compatibility"] = (
            "Actual combined responses are preserved in commands. During inherited .2 regression only, "
            "error return labels are normalized to its hardcoded Zoom+AutoJump prefix; error body/config checks stay unchanged.")

    def meteor(self, arguments):
        actual = super().meteor(arguments)
        if self.in_regression and actual.startswith(self.error_prefix):
            return "Zoom+AutoJump migration candidate:" + actual[len(self.error_prefix):]
        return actual

    def status(self):
        result = super().status()
        air, high, order = AIR.search(result["text"]), HIGH.search(result["text"]), ORDER.search(result["text"])
        if not air or not high or not order:
            raise RuntimeError("Combined movement status schema mismatch: " + result["text"])
        air_enabled, maintain, _, level = air.groups()
        high_enabled, multiplier = high.groups()
        result.update(air_jump_enabled=air_enabled=="enabled", maintain_level=maintain=="true",
                      captured_level=None if level is None else int(level),
                      high_jump_enabled=high_enabled=="enabled", jump_multiplier=float(multiplier),
                      pre_order=json.loads(order.group(1)))
        return result

    def module(self, name, enabled):
        status = self.status()
        field = name.replace("-", "_") + "_enabled"
        if status[field] != enabled:
            self.meteor(name + " toggle")
            status = self.status()
        return status

    def all_off(self):
        self.ctl("release")
        self.jump_enabled(False)
        self.module("air-jump", False)
        self.module("high-jump", False)

    def airborne(self, height=18):
        x,y,z = self.origin
        self.ctl("release")
        self.ctl("command", f"tp @s {x+0.5} {y+height} {z+0.5}")
        self.ctl("wait", 2)
        state = self.check_playing()
        self.check("Generated airborne fixture is actually off ground", not state["player"]["on_ground"],state["player"])
        return state

    def trace(self, label, count=24, first=None, baseline=None):
        """Observe one actual jump with varied spacing and retain every sample."""
        records = [] if first is None else [{"elapsed":0.,"player":first["player"],"screen":first["screen"]}]
        start = time.monotonic()
        for i in range(count):
            delay = (0,1,0,2,1,0,3)[i%7]
            if delay: self.ctl("wait",delay)
            state = self.check_playing()
            records.append({"elapsed":time.monotonic()-start,"player":state["player"],"screen":state["screen"]})
        self.report["samples"][label] = records
        players = [row["player"] for row in records]
        base = self.floor_y if baseline is None else baseline
        summary = {"rise":max(p["position"][1] for p in players)-base,
                   "min_y":min(p["position"][1] for p in players),
                   "max_vy":max(p["velocity"][1] for p in players),
                   "max_horizontal_speed":max(math.hypot(p["velocity"][0],p["velocity"][2]) for p in players),
                   "airborne":sum(not p["on_ground"] for p in players),"count":len(players)}
        self.report.setdefault("sample_summaries",{})[label] = summary
        return summary

    def native_stat_snapshot(self, label):
        # Read only this generated world's actual saved native statistic files.
        # A saved authoritative counter does not by itself identify the client
        # wrapper callback or establish exact exhaustion/sync side effects.
        result={"files":{},"scope":"generated saved native jump counters; observational"}
        try:
            result["save_reply"]=self.ctl("save")
            for path in self.home.rglob("stats/*.json"):
                if "players" not in path.relative_to(self.home).parts:continue
                data=json.loads(path.read_text(encoding="utf-8-sig"))
                value=data.get("stats",{}).get("minecraft:custom",{}).get("minecraft:jump",0)
                result["files"][str(path.relative_to(self.home))]={"jump":value,"sha256":zoom_harness.sha256(path)}
        except (RuntimeError,OSError,ValueError) as error:
            result["unavailable"]=repr(error)
        self.report.setdefault("native_stat_snapshots",{})[label]=result
        return result

    def manual_jump(self, label, multiplier=None, boost=None):
        self.all_off()
        self.ctl("command","effect clear @s")
        self.ctl("wait",4)
        self.reset_player()
        if multiplier is not None:
            self.meteor("high-jump jump-multiplier " + str(multiplier))
            self.module("high-jump",True)
        if boost is not None:
            self.ctl("command",f"effect give @s minecraft:jump_boost 60 {boost} true")
            self.ctl("wait",6)
            effects = self.check_playing()["player"].get("effects",[])
            self.check(label+" boost reaches native player",any(e.get("amplifier")==boost for e in effects),effects)
        stat_before=self.native_stat_snapshot(label+"_before")
        self.ctl("look",0,0)
        self.ctl("key","j","down")
        self.ctl("wait",1)
        first = self.check_playing()
        self.ctl("key","j","up")
        result = self.trace(label,24,first)
        self.ctl("wait",30)
        stat_after=self.native_stat_snapshot(label+"_after")
        self.report.setdefault("observed_native_jump_stat_deltas",{})[label]={
            path:row["jump"]-stat_before["files"][path]["jump"]
            for path,row in stat_after["files"].items() if path in stat_before["files"]}
        return result

    def air_checks(self):
        self.ctl("release")
        self.ctl("command","gamemode survival")
        self.ctl("wait",4)
        air_mode=self.check_playing()
        self.check("AirJump tests use survival without creative flight",
                   air_mode.get("gamemode")=="survival" and not air_mode["player"].get("flying"),air_mode["player"])
        self.all_off()
        self.reset_player()
        self.ctl("command","effect give @s minecraft:slow_falling 1800 0 true")
        self.ctl("wait",4)
        self.meteor("air-jump maintain-level false")
        activated = self.module("air-jump",True)
        self.check("AirJump captures grounded activation block height",activated["captured_level"]==math.floor(self.floor_y),activated)
        before = self.airborne(18)
        captured = self.status()["captured_level"]
        self.ctl("key","space","tap")
        space = self.status()
        self.check("Unbound Space does not recapture AirJump height",space["captured_level"]==captured,space)
        before = self.check_playing()
        self.ctl("key","j","down")
        after = self.check_playing()
        self.ctl("key","j","up")
        jumped = self.status()
        self.report["air_rebound"] = {"before":before,"after":after,"status":jumped}
        self.check("Rebound J triggers an actual airborne native jump",
                   after["player"]["velocity"][1]>0 or after["player"]["position"][1]>before["player"]["position"][1]+0.05,
                   self.report["air_rebound"])
        self.check("J recaptures current actual airborne block height",
                   jumped["captured_level"] is not None and abs(jumped["captured_level"]-math.floor(before["player"]["position"][1]))<=1,jumped)
        level = jumped["captured_level"]
        self.airborne(26)
        self.ctl("key","j","repeat")
        self.ctl("key","j","up")
        repeated = self.status()
        self.check("Keyboard repeats do not recapture AirJump",repeated["captured_level"]==level,repeated)
        self.ctl("key","shift","tap")
        sneaked = self.status()
        self.check("Airborne sneak press decrements captured level once",sneaked["captured_level"]==level-1,sneaked)
        self.ctl("key","shift","repeat")
        self.ctl("key","shift","up")
        self.check("Sneak repeats do not decrement AirJump",self.status()["captured_level"]==level-1)
        self.airborne(26)
        level = self.status()["captured_level"]
        self.ctl("key","t","tap")
        self.ctl("key","j","tap")
        self.ctl("key","escape","tap")
        self.check("AirJump rejects jump presses while chat screen is open",self.status()["captured_level"]==level)
        off = self.module("air-jump",False)
        self.check("AirJump disable clears captured level",off["captured_level"] is None,off)
        # Typed commands advance time. Compare with a stable authoritative
        # grounded reset, not a falling position sampled several calls earlier.
        reset = self.reset_player()
        position = reset["player"]["position"]
        again = self.module("air-jump",True)
        self.check("AirJump re-enable captures fresh current height",again["captured_level"] is not None and
                   again["captured_level"]==math.floor(position[1]),{"reset":reset,"status":again})
        self.meteor("air-jump maintain-level true")
        self.airborne(18)
        self.ctl("key","j","tap")
        before_hold_level=self.status()["captured_level"]
        self.ctl("key","j","down")
        # Never open chat or another screen while the action must remain down.
        # This second physical Press captures its own current height; query that
        # same retained level only after releasing J, when the trace is complete.
        maintained = self.trace("airjump_maintain_rebound_j",36,baseline=self.floor_y)
        self.ctl("key","j","up")
        level=self.status()["captured_level"]
        self.check("Held rebound J maintains captured block level",level is not None and maintained["min_y"]>=level-0.05,
                   {"level":level,"before_hold_level":before_hold_level,"samples":maintained,
                    "screen_opened_while_held":False})
        self.meteor("air-jump maintain-level false")
        self.module("air-jump",False)
        self.ctl("command","effect clear @s")
        self.ctl("wait",40)
        saved = self.saved()
        self.check("AirJump settings persist in generated local JSON",not saved["air_jump_enabled"] and
                   saved["air_jump"]["maintain_level"] is False,saved)

    def high_checks(self):
        self.ctl("command","gamemode survival")
        self.ctl("wait",4)
        self.check("Manual jump tests use survival without creative flight",self.check_playing().get("gamemode")=="survival"
                   and not self.check_playing()["player"].get("flying"))
        normal = self.manual_jump("manual_normal")
        self.check("Manual rebound J produces normal rise",0.8<normal["rise"]<1.6,normal)
        doubled = self.manual_jump("manual_highjump2",2)
        self.check("HighJump2 raises manual native jump above ordinary",doubled["rise"]>normal["rise"]*2.5 and doubled["rise"]<6,doubled)
        zero = self.manual_jump("manual_highjump0",0)
        self.check("HighJump0 permits zero-height manual native jump",abs(zero["rise"])<0.05,zero)
        boosted = self.manual_jump("manual_highjump0_boost5",0,4)
        self.check("Additive native JumpBoost survives HighJump multiplier0",boosted["rise"]>1.0,boosted)
        tiny = self.manual_jump("manual_highjump_tiny",1e-8)
        self.check("Tiny positive HighJump multiplier is not rejected or clamped",
                   abs(tiny["rise"])<0.05 and self.saved()["high_jump"]["jump_multiplier"]==1e-8,tiny)
        restored = self.manual_jump("manual_restored")
        self.check("Disabling HighJump restores ordinary measured rise",abs(restored["rise"]-normal["rise"])<0.25,restored)
        # Zero still executes the source sprint impulse, even when vertical motion is0.
        self.all_off();self.ctl("command","effect clear @s");self.reset_player()
        self.meteor("high-jump jump-multiplier 0");self.module("high-jump",True)
        self.ctl("key","w","down");self.ctl("key","ctrl","down");self.ctl("wait",12)
        before = self.check_playing()
        self.check("Zero-power impulse fixture is genuinely sprinting",before["player"].get("sprinting"),before["player"])
        speed = math.hypot(before["player"]["velocity"][0],before["player"]["velocity"][2])
        self.ctl("key","j","down")
        first = self.check_playing()
        self.ctl("key","j","up")
        impulse = self.trace("zero_highjump_sprint_impulse",8,first)
        self.ctl("release")
        self.check("HighJump0 retains measurable native sprint impulse",
                   impulse["max_horizontal_speed"]>speed*1.08+0.005,
                   {"before_speed":speed,"trace":impulse,"before":before})
        self.module("high-jump",False)
        self.reset_player()
        self.meteor("high-jump jump-multiplier 2");self.module("high-jump",True)
        self.configure();self.jump_enabled(True)
        combined = self.sample("autojump_with_highjump2",30)
        self.check("AutoJump real jump helper composes with HighJump2",combined["rise"]>2.5,combined)
        self.jump_enabled(False);self.module("high-jump",False);self.reset_player()
        self.ctl("command","effect give @s minecraft:slow_falling 1800 0 true")
        self.meteor("air-jump maintain-level false");self.module("air-jump",True)
        self.meteor("high-jump jump-multiplier 2");self.module("high-jump",True)
        before = self.airborne(18)
        base = before["player"]["position"][1]
        self.ctl("key","j","down");first=self.check_playing();self.ctl("key","j","up")
        combined_air = self.trace("airjump_with_highjump2",26,first,base)
        self.check("AirJump rebound helper composes with HighJump2",combined_air["rise"]>2.5,combined_air)
        self.all_off();self.ctl("command","effect clear @s");self.ctl("wait",40)
        self.report["native_tail_runtime_scope"] = {"statistics":"generated native saved jump-counter snapshots collected when available; deltas are observational, not client-wrapper attribution",
            "exhaustion":"not exposed; food/saturation is insufficient for exact per-jump amounts",
            "needs_sync":"not exposed","exactly_once":"source-path audit and separate native identity tests; not dynamic count proof"}

    def lifecycle(self):
        if self.args.skip_lifecycle:
            self.report["world_lifecycle"]={"verified":False,"reason":"explicit --skip-lifecycle"};return
        self.all_off();self.reset_player()
        self.ctl("command","effect give @s minecraft:slow_falling 1800 0 true")
        self.meteor("air-jump maintain-level false");self.module("air-jump",True)
        self.airborne(18);self.ctl("key","j","tap")
        old = self.status()["captured_level"]
        x,y,z=self.origin
        self.fixture_fill(f"fill {x-2} {y+4} {z-2} {x+2} {y+4} {z+2} minecraft:stone")
        self.ctl("command",f"tp @s {x+0.5} {y+5} {z+0.5}")
        self.ctl("wait",20)
        self.ctl("command","effect clear @s")
        grounded = self.check_playing()
        self.check("Lifecycle generated elevated platform is grounded",grounded["player"]["on_ground"],grounded["player"])
        self.check("Lifecycle setup has distinct retained activation level",old!=math.floor(grounded["player"]["position"][1]),old)
        world = self.report["launch"][self.report["launch"].index("--world")+1]
        # This exact world was created by this harness, in its fresh private cwd.
        self.ctl("release");self.ctl("disconnect")
        deadline=time.monotonic()+60
        while self.ctl("state").get("player") is not None:
            if time.monotonic()>deadline:raise TimeoutError("Generated world did not disconnect")
            time.sleep(0.2)
        self.ctl("open",world)
        deadline=time.monotonic()+180
        while True:
            state=self.ctl("state")
            if state.get("player") and state.get("screen")=="playing":break
            if time.monotonic()>deadline:raise TimeoutError("Generated world did not reopen")
            time.sleep(0.5)
        status=self.status()
        expected=math.floor(state["player"]["position"][1])
        self.check("Generated world leave/rejoin refreshes AirJump activation height",
                   status["air_jump_enabled"] and status["captured_level"]==expected and status["captured_level"]!=old,
                   {"old":old,"new":status,"rejoined":state})
        self.report["world_lifecycle"]={"verified":True,"world":world,"old_level":old,"new_level":status["captured_level"]}
        self.all_off()

    def clear_air_room(self):
        x,y,z=self.origin
        command=f"fill {x-10} {y} {z-10} {x+10} {y+32} {z+10} minecraft:air"
        deadline=time.monotonic()+60
        while True:
            reply=self.ctl("command",command)
            lines=reply.get("chat",[])
            self.report.setdefault("fixture_commands",[]).append({"command":command,"reply":reply})
            if any("not loaded" in str(line).lower() for line in lines):
                if time.monotonic()>deadline:raise TimeoutError("HighJump air-room chunks did not load")
                self.ctl("wait",20);continue
            # A generated room already consisting of air is a valid no-op fill.
            valid=any(str(line).startswith("Successfully filled") or "no blocks were filled" in str(line).lower() or "no blocks filled" in str(line).lower() for line in lines)
            self.check("Generated high-jump air room is cleared or already air",valid,reply)
            return

    def run_checks(self):
        initial=self.status()
        self.check("Combined candidate starts AirJump and HighJump disabled",
                   not initial["air_jump_enabled"] and not initial["high_jump_enabled"] and initial["jump_multiplier"]==1,initial)
        before=len(self.report["checks"]);self.in_regression=True
        try:super().run_checks()
        finally:self.in_regression=False
        self.report["inherited_autojump_regression_checks"]=len(self.report["checks"])-before
        # Repeated jumps exercise motion rather than fall injury. This rule
        # affects only damage in this newly generated world, not jump physics.
        self.ctl("command","gamerule fall_damage false")
        rule = self.ctl("command","gamerule fall_damage")
        self.report["fixture_fall_damage_rule"] = rule
        self.check("Generated movement world has fall damage disabled",
                   any("false" in line.lower() for line in rule.get("chat",[])),rule)
        self.clear_air_room()
        self.air_checks();self.high_checks();self.lifecycle()
        self.report["final_config"]=self.saved();self.state("movement_final")

    def execute(self):
        code=super().execute()
        after={k:zoom_harness.sha256(v) for k,v in self.script_paths.items()}
        self.report["movement_harness_hashes_after"]=after
        unchanged=self.report["movement_harness_hashes"]==after
        self.report["checks"].append({"name":"Own movement and both inherited harness files stayed byte-immutable","passed":unchanged})
        self.report["passed"]=bool(self.report["passed"] and unchanged)
        text=json.dumps(self.report,indent=2,ensure_ascii=False)
        (self.scratch/"report.json").write_text(text,encoding="utf8")
        (self.root/"standard-movement-runtime-current.json").write_text(text,encoding="utf8")
        return 0 if self.report["passed"] else 1


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ["client-binary","control-binary","resources","scratch-root"]:
        parser.add_argument("--"+name,type=Path,required=True)
    parser.add_argument("--candidate-prefix",default=DEFAULT_PREFIX)
    parser.add_argument("--skip-lifecycle",action="store_true",help="Record generated world leave/rejoin as unverified")
    args=parser.parse_args()
    for name in ["client_binary","control_binary","resources"]:
        path=getattr(args,name).resolve(strict=True)
        if not (path.is_dir() if name=="resources" else path.is_file()):parser.error(name+" has wrong path type")
        setattr(args,name,path)
    args.skip_gui=True
    return MovementHarness(args).execute()


if __name__=="__main__":raise SystemExit(main())
