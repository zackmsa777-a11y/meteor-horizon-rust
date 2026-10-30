#!/usr/bin/env python3
"""Fresh-world native Zoom+AutoJump candidate runtime checks. No launch on import.

Reuses the verified standard Zoom harness's exact-binary hashing, isolation,
actual chat command transport, local styled feedback and graceful shutdown.
All test worlds/files are new children of required --scratch-root.
"""
from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
import re
import statistics
import time
import uuid

import test_standard_zoom as zoom_harness

# Only this process's imported module is configured; the Zoom harness file stays unchanged.
zoom_harness.PREFIXES = ("Zoom+AutoJump .hnmod migration candidate:",
                         "Zoom+AutoJump migration candidate:")
AUTOJUMP = re.compile(r"AutoJump (enabled|disabled) \((Jump|LowHop), "
                      r"(Always|Walking|Sprinting), LowHop height ([^)]+)\)")


class AutoJumpHarness(zoom_harness.Harness):
    def __init__(self, args):
        # The inherited harness's latest-zoom pointer stays inside a new namespace,
        # so a candidate run cannot overwrite a verified Zoom run's evidence.
        requested = args.scratch_root.resolve()
        args.scratch_root = requested / ("autojump-run-" + uuid.uuid4().hex)
        super().__init__(args)
        self.report["kind"] = "normal-hnmod-zoom-autojump-candidate"
        self.report["requested_scratch_root"] = str(requested)
        self.report["limitations"] = [
            "Zoom and AutoJump candidate only; other retained modules are not integrated.",
            "Heights are sampled observations, not exact mathematical apex values.",
            "LowHop0 walking is observed without assuming unchanged steady speed: zero Y changes ground-state friction.",
            "No independent GUI-screen filter is assumed for source AutoJump.",
            "Generated local JSON persistence is checked without restarting the client.",
        ]
        self.report["samples"] = {}
        self.report["harness_hashes"] = {
            "autojump": zoom_harness.sha256(Path(__file__)),
            "zoom_base": zoom_harness.sha256(Path(zoom_harness.__file__)),
        }

    def fixture_fill(self, command):
        # Console transport can return ok:true with a command failure in chat.
        # Never continue movement checks on an uncreated platform.
        deadline = time.monotonic() + 60
        while True:
            reply = self.ctl("command", command)
            lines = reply.get("chat", [])
            self.report.setdefault("fixture_commands", []).append({"command": command, "reply": reply})
            if any("not loaded" in str(line).lower() for line in lines):
                if time.monotonic() >= deadline:
                    raise TimeoutError("Forced fixture chunks did not load: " + repr(lines))
                self.ctl("wait", 20)
                continue
            self.check("Generated fixture fill succeeds: " + command.split()[-1],
                       any(str(line).startswith("Successfully filled") for line in lines), lines)
            return

    def meteor(self, arguments):
        # Ordered feedback detects a repeated newest status even when an identical
        # old line is evicted from the bounded100-message history window.
        prefixes = zoom_harness.PREFIXES
        before = [line for line in self.lines() if line.startswith(prefixes)]
        self.check_playing()
        self.ctl("key", "t", "tap")
        if self.ctl("state").get("screen") != "chat":
            raise RuntimeError("T did not open the actual chat screen")
        self.ctl("type", "/meteor-migration " + arguments)
        self.ctl("key", "enter", "tap")
        deadline = time.monotonic() + 8
        while True:
            lines = [line for line in self.lines() if line.startswith(prefixes)]
            if lines and lines != before:
                response = lines[0]
                self.report["commands"].append({"command": "/meteor-migration " + arguments,
                                                "response": response})
                self.check_playing()
                return response
            if time.monotonic() >= deadline:
                raise TimeoutError(f"No fresh ordered native feedback for {arguments!r}: {lines}")
            self.ctl("wait", 1)

    def status(self):
        text = self.meteor("status")
        zoom = zoom_harness.STATUS.search(text)
        jump = AUTOJUMP.search(text)
        if not zoom or not jump:
            raise RuntimeError("Combined native status schema mismatch: " + text)
        enabled, requested, current, smooth = zoom.groups()
        jump_enabled, mode, jump_if, height = jump.groups()
        return {"enabled": enabled == "enabled", "requested": float(requested),
                "current": float(current), "smooth": smooth == "true",
                "auto_jump_enabled": jump_enabled == "enabled", "mode": mode,
                "jump_if": jump_if, "velocity_height": float(height), "text": text}

    def jump_enabled(self, enabled):
        current = self.status()
        if current["auto_jump_enabled"] != enabled:
            self.meteor("auto-jump toggle")
            return self.status()
        return current

    def configure(self, mode="Jump", when="Always", height=0.25):
        self.jump_enabled(False)
        self.meteor("auto-jump mode " + mode)
        self.meteor("auto-jump jump-if " + when)
        self.meteor("auto-jump velocity-height " + str(height))

    def reset_player(self):
        self.jump_enabled(False)
        self.ctl("release")
        x, y, z = self.origin
        self.ctl("command", f"tp @s {x+0.5} {y} {z+0.5}")
        self.ctl("wait", 18)
        self.ctl("look", 0, 0)
        value = self.check_playing()
        self.check("Generated fixture reset is grounded", value["player"]["on_ground"], value["player"])
        self.floor_y = value["player"]["position"][1]
        return value

    def sample(self, label, count=28):
        """Vary tick spacing to avoid repeatedly polling the same jump phase."""
        records = []
        start = time.monotonic()
        pattern = (1, 2, 0, 1, 3, 0, 2)
        for i in range(count):
            if pattern[i % len(pattern)]:
                self.ctl("wait", pattern[i % len(pattern)])
            value = self.check_playing()
            player = value["player"]
            records.append({"elapsed": time.monotonic()-start, "player": player,
                            "screen": value["screen"], "gamemode": value.get("gamemode")})
        self.report["samples"][label] = records
        players = [row["player"] for row in records]
        heights = [p["position"][1] for p in players]
        speeds = [math.hypot(p["velocity"][0], p["velocity"][2]) for p in players]
        first, last = players[0]["position"], players[-1]["position"]
        result = {
            "rise": max(heights)-self.floor_y, "lowest": min(heights)-self.floor_y,
            "airborne": sum(not p["on_ground"] for p in players),
            "sprinting": sum(p.get("sprinting", False) for p in players),
            "sneaking": sum(p.get("sneaking", False) for p in players),
            "max_vy": max(p["velocity"][1] for p in players),
            "horizontal_distance": math.hypot(last[0]-first[0], last[2]-first[2]),
            "steady_horizontal_speed": statistics.median(speeds[-8:]),
            "max_horizontal_speed": max(speeds), "count": len(records),
        }
        self.report.setdefault("sample_summaries", {})[label] = result
        return result

    def flat(self, label, result):
        self.check(label, result["rise"] < 0.05 and result["lowest"] > -0.05 and
                   result["airborne"] == 0, result)

    def jumping(self, label, result):
        self.check(label, result["rise"] > 1.0 and result["airborne"] > 0, result)

    def level(self, label, result):
        # Native Entity.move recomputes ground from a downward collision. A
        # source-faithful Y=0 assignment can clear that flag without rising.
        self.check(label, result["rise"] < 0.05 and result["lowest"] > -0.05
                   and result["max_vy"] <= 0, result)

    def run_checks(self):
        self.ctl("release")
        self.ctl("command", "difficulty peaceful")
        self.ctl("command", "gamemode creative")
        self.ctl("command", "time set 6000")
        initial = self.state("initial")
        self.check("Generated test world is creative", initial.get("gamemode") == "creative")
        self.origin = tuple(math.floor(v) for v in initial["player"]["position"])
        x, y, z = self.origin
        self.report["fixture_origin"] = self.origin
        # Long generated corridor; each fill stays below vanilla's32768block limit.
        self.report["fixture_forceload"] = self.ctl("command", f"forceload add {x-12} {z-120} {x+12} {z+120}")
        self.ctl("wait", 40)
        self.fixture_fill(f"fill {x-12} {y-1} {z-120} {x+12} {y-1} {z+120} minecraft:stone")
        self.fixture_fill(f"fill {x-12} {y} {z-120} {x+12} {y+4} {z+120} minecraft:air")
        self.reset_player()
        initial_status = self.status()
        self.check("AutoJump source defaults are disabled Jump Always height0.25",
                   not initial_status["auto_jump_enabled"] and initial_status["mode"] == "Jump" and
                   initial_status["jump_if"] == "Always" and initial_status["velocity_height"] == 0.25,
                   initial_status)
        self.flat("Disabled AutoJump leaves stationary player grounded", self.sample("disabled_stationary", 18))

        # Small real-host Zoom regression, while AutoJump remains disabled.
        base_motion = self.mouse_response("zoom_regression_baseline")
        self.meteor("zoom toggle")
        zoom = self.settle()
        self.check("Combined candidate preserves factor6 Zoom", zoom["enabled"] and abs(zoom["current"]-6)<0.002, zoom)
        zoom_motion = self.mouse_response("zoom_regression_enabled")
        self.check("Combined candidate preserves actual Zoom yaw ratio1/3",
                   abs(zoom_motion/base_motion-1/3)<0.035,
                   {"baseline":base_motion,"zoom":zoom_motion})
        self.meteor("zoom toggle")
        off = self.settle()
        self.check("Combined candidate disables Zoom back to factor1", not off["enabled"] and off["current"] == 1, off)
        off_motion = self.mouse_response("zoom_regression_restored")
        self.check("Combined candidate restores real mouse sensitivity", abs(off_motion/base_motion-1)<0.035,
                   {"baseline":base_motion,"restored":off_motion})
        defaults = self.saved()
        self.check("Generated config persists source AutoJump defaults alongside Zoom",
                   defaults["schema"] == 1 and not defaults["auto_jump_enabled"] and
                   defaults["auto_jump"] == {"mode":"Jump","jump_if":"Always","velocity_height":0.25}, defaults)

        self.configure()
        self.reset_player()
        self.jump_enabled(True)
        self.jumping("Always Jump causes real airborne elevation above1block", self.sample("always_jump", 34))

        self.configure(when="Walking")
        self.reset_player()
        self.jump_enabled(True)
        self.flat("Walking predicate ignores a stationary player", self.sample("walking_stationary", 18))
        self.ctl("key", "w", "down")
        walking = self.sample("walking_jumps", 30)
        self.ctl("key", "w", "up")
        self.jumping("Walking predicate jumps while moving", walking)
        self.check("Walking fixture actually travels horizontally", walking["horizontal_distance"] > 1, walking)

        self.configure(when="Sprinting")
        self.reset_player()
        self.jump_enabled(True)
        self.flat("Sprinting predicate ignores stationary player", self.sample("sprinting_stationary", 18))
        self.ctl("key", "w", "down")
        just_walking = self.sample("sprinting_walking", 20)
        self.flat("Sprinting predicate ignores ordinary walking", just_walking)
        self.check("Ordinary-walking control did not sprint", just_walking["sprinting"] == 0 and
                   just_walking["horizontal_distance"] > 1, just_walking)
        self.ctl("key", "ctrl", "down")
        sprinting = self.sample("sprinting_jumps", 34)
        self.ctl("release")
        self.check("Sprinting fixture enters actual native sprint state", sprinting["sprinting"] > 0, sprinting)
        self.jumping("Sprinting predicate jumps during actual sprint", sprinting)

        self.configure()
        self.reset_player()
        self.jump_enabled(True)
        self.ctl("key", "shift", "down")
        # A jump can already be in flight after the actual chat toggle; settle it
        # while Shift is held before measuring suppression. No GUI guard invented.
        self.ctl("wait", 40)
        sneaking = self.sample("sneak_suppression", 20)
        self.flat("Sneak suppresses Always AutoJump after landing", sneaking)
        self.check("Sneak suppression uses actual held sneak input", sneaking["sneaking"] == sneaking["count"], sneaking)
        self.ctl("key", "shift", "up")
        self.jumping("Releasing sneak permits Always jumps again", self.sample("sneak_released", 28))

        self.configure(mode="LowHop", height=0.25)
        self.reset_player()
        self.jump_enabled(True)
        low = self.sample("lowhop_025", 34)
        self.check("LowHop0.25 produces smaller real airborne rise", 0.20 < low["rise"] < 0.85 and
                   low["airborne"] > 0 and low["max_vy"] > 0, low)

        # Observe horizontal motion too. Source changes Y alone, but its zero-Y
        # ground-flag cycle changes native friction, so equal steady speed is
        # not a valid requirement. The measured speed ratio is retained.
        self.reset_player()
        self.ctl("key", "w", "down")
        horizontal_baseline = self.sample("horizontal_disabled", 26)
        self.ctl("release")
        self.flat("Horizontal control remains on flat ground", horizontal_baseline)
        self.configure(mode="LowHop", height=0)
        self.reset_player()
        self.jump_enabled(True)
        self.level("LowHop0 causes no stationary elevation", self.sample("lowhop_zero_stationary", 18))
        self.ctl("key", "w", "down")
        horizontal_zero = self.sample("lowhop_zero_walking", 26)
        self.ctl("release")
        self.level("LowHop0 keeps walking height level", horizontal_zero)
        baseline_speed = horizontal_baseline["steady_horizontal_speed"]
        lowhop_speed = horizontal_zero["steady_horizontal_speed"]
        self.check("LowHop0 permits continuing horizontal walking",
                   baseline_speed > 0.01 and lowhop_speed > 0.01 and
                   math.isfinite(lowhop_speed) and horizontal_zero["horizontal_distance"] > 1,
                   {"disabled":horizontal_baseline,"lowhop0":horizontal_zero,"speed_ratio":
                    lowhop_speed/baseline_speed if baseline_speed else None})

        self.jump_enabled(False)
        valid = self.config.read_bytes()
        for argument in ["auto-jump velocity-height -1", "auto-jump velocity-height NaN",
                         "auto-jump velocity-height inf", "auto-jump mode Nope", "auto-jump jump-if Nope"]:
            response = self.meteor(argument)
            self.check("Invalid AutoJump setting is atomic: " + argument,
                       response.startswith("Zoom+AutoJump migration candidate:") and
                       self.config.read_bytes() == valid, response)
        self.configure()
        self.reset_player()
        self.jump_enabled(True)
        self.jumping("Normal Jump still works after LowHop changes", self.sample("jump_after_lowhop", 28))
        self.jump_enabled(False)
        self.ctl("release")
        self.ctl("wait", 40)
        self.flat("Disabling AutoJump restores stationary ground rest", self.sample("disabled_restored", 20))
        self.report["final_config"] = self.saved()
        final = self.status()
        self.check("Final generated config restores disabled source AutoJump defaults",
                   not final["auto_jump_enabled"] and final["mode"] == "Jump" and final["jump_if"] == "Always"
                   and final["velocity_height"] == 0.25 and not self.report["final_config"]["auto_jump_enabled"],
                   {"status": final,"config":self.report["final_config"]})
        self.state("final")

    def execute(self):
        result = super().execute()
        self.report["harness_hashes_after"] = {
            "autojump": zoom_harness.sha256(Path(__file__)),
            "zoom_base": zoom_harness.sha256(Path(zoom_harness.__file__)),
        }
        unchanged = self.report["harness_hashes"] == self.report["harness_hashes_after"]
        self.report["checks"].append({"name": "Exact runtime harnesses remained unchanged", "passed": unchanged})
        self.report["passed"] = bool(self.report["passed"] and unchanged)
        text = json.dumps(self.report, indent=2, ensure_ascii=False)
        (self.scratch / "report.json").write_text(text, encoding="utf8")
        # The inherited pointer is confined to this run's private namespace.
        (self.root / "standard-autojump-runtime-current.json").write_text(
            text, encoding="utf8")
        return 0 if self.report["passed"] else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--client-binary", type=Path, required=True)
    parser.add_argument("--control-binary", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--scratch-root", type=Path, required=True,
                        help="All worlds/settings/reports stay under a fresh child of this directory")
    args = parser.parse_args()
    for name in ["client_binary", "control_binary", "resources"]:
        path = getattr(args, name).resolve(strict=True)
        if not (path.is_dir() if name == "resources" else path.is_file()):
            parser.error(name.replace("_", "-") + " has the wrong path type")
        setattr(args, name, path)
    args.skip_gui = True
    return AutoJumpHarness(args).execute()


if __name__ == "__main__":
    raise SystemExit(main())
