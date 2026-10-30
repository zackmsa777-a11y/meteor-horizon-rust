#!/usr/bin/env python3
"""Fresh singleplayer AutoRespawn and BreakDelay checks for the standard .hnmod.

All generated worlds and files are children of the caller's scratch root. The
window of a singleplayer world is a client of its integrated server, so these
checks drive the same player-facing paths as a held mouse button and a death
screen. Block breaking is timed in game ticks (`state.ticks`), not control
steps: one `wait` unit is several ticks.
"""
from __future__ import annotations

import argparse
import json
import math
import re
import time
from pathlib import Path

import test_standard_autojump as autojump_harness
from test_standard_autojump import AutoJumpHarness

zoom_harness = autojump_harness.zoom_harness
DEFAULT_PREFIX = "Zoom+AutoJump+AirJump+HighJump .hnmod migration candidate:"
STATUS = re.compile(r"AutoRespawn (enabled|disabled); BreakDelay (enabled|disabled) "
                    r"\(cooldown (\d+), no-insta-break (true|false)\)")
LINE = 4  # blocks z+2..z+5 are all within creative reach of the fixed pose


class PlayerInteractionsHarness(AutoJumpHarness):
    def __init__(self, args):
        super().__init__(args)
        zoom_harness.PREFIXES = (args.candidate_prefix, args.candidate_prefix.replace(".hnmod ", ""))
        self.report["kind"] = "normal-hnmod-player-interactions"
        self.report["candidate_prefix"] = args.candidate_prefix

    def modules(self):
        match = STATUS.search(self.meteor("status"))
        if not match:
            raise RuntimeError("AutoRespawn/BreakDelay status schema mismatch")
        respawn, delay, cooldown, insta = match.groups()
        return {"auto_respawn": respawn == "enabled", "break_delay": delay == "enabled",
                "cooldown": int(cooldown), "no_insta_break": insta == "true"}

    def toggle(self, command, expected):
        label = {"auto-respawn": "AutoRespawn", "break-delay": "BreakDelay"}[command]
        reply = self.meteor(command + " toggle")
        self.check(f"{label} can be switched {'on' if expected else 'off'} through real chat",
                   f"{label} {'enabled' if expected else 'disabled'}" in reply, reply)

    def hold_break(self, x, y, z, hold_ticks=None):
        """Hold the real left button on the generated stone line; time the clearing in ticks."""
        self.ctl("release")
        self.fixture_fill(f"fill {x} {y+1} {z+2} {x} {y+1} {z+1+LINE} minecraft:stone")
        self.ctl("command", f"tp @s {x+0.5} {y} {z+0.5}")
        self.ctl("look", 0, 0)
        self.ctl("wait", 8)
        start = self.ctl("state")["ticks"]
        self.ctl("mouse", "left", "down")
        elapsed, remaining = 0, LINE
        for _ in range(10):
            self.ctl("wait", 1)
            elapsed = self.ctl("state")["ticks"] - start
            remaining = len(self.ctl("blocks", x, y+1, z+2, x, y+1, z+1+LINE)["blocks"])
            if remaining == 0 or (hold_ticks is not None and elapsed >= hold_ticks):
                break
        self.ctl("mouse", "left", "up")
        self.ctl("wait", 3)
        return {"ticks": elapsed, "remaining": remaining}

    def run_checks(self):
        self.ctl("release")
        self.ctl("command", "difficulty peaceful")
        self.ctl("command", "gamemode creative")
        initial = self.state("initial")
        self.check("Fresh world starts in creative", initial.get("gamemode") == "creative")
        x, y, z = (math.floor(v) for v in initial["player"]["position"])
        self.report["fixture_origin"] = [x, y, z]
        self.ctl("command", f"forceload add {x-5} {z-5} {x+5} {z+18}")
        self.ctl("wait", 20)
        self.fixture_fill(f"fill {x-3} {y-1} {z-3} {x+3} {y-1} {z+16} minecraft:stone")
        self.fixture_fill(f"fill {x-3} {y} {z-3} {x+3} {y+5} {z+16} minecraft:air")

        defaults = self.modules()
        self.check("AutoRespawn and BreakDelay default disabled with source settings",
                   defaults == {"auto_respawn": False, "break_delay": False,
                                "cooldown": 0, "no_insta_break": False}, defaults)
        self.toggle("auto-respawn", True)
        self.toggle("auto-respawn", False)

        # Creative cadence: ordinary Horizon waits five ticks between breaks.
        samples = self.report.setdefault("break_samples", {})
        ordinary = [self.hold_break(x, y, z) for _ in range(2)]
        samples["ordinary"] = ordinary
        self.check("Ordinary held creative break clears the line at the vanilla cadence",
                   all(s["remaining"] == 0 and s["ticks"] >= 12 for s in ordinary), ordinary)
        self.toggle("break-delay", True)
        fast = [self.hold_break(x, y, z) for _ in range(2)]
        samples["enabled"] = fast
        self.check("BreakDelay cooldown 0 clears the same line much faster than ordinary",
                   all(s["remaining"] == 0 for s in fast)
                   and max(s["ticks"] for s in fast) + 6 <= min(s["ticks"] for s in ordinary),
                   {"ordinary": ordinary, "enabled": fast})
        self.toggle("break-delay", False)
        restored = [self.hold_break(x, y, z) for _ in range(2)]
        samples["restored"] = restored
        self.check("Disabling BreakDelay restores the ordinary cadence",
                   all(s["remaining"] == 0 for s in restored)
                   and min(s["ticks"] for s in restored) >= max(s["ticks"] for s in fast) + 6
                   and abs(sum(s["ticks"] for s in restored) - sum(s["ticks"] for s in ordinary)) <= 14,
                   {"ordinary": ordinary, "enabled": fast, "restored": restored})

        # Atomic setting validation and generated JSON persistence.
        before = self.config.read_bytes()
        for bad in ("break-delay cooldown -1", "break-delay cooldown nope", "break-delay no-insta-break maybe"):
            reply = self.meteor(bad)
            self.check("Invalid BreakDelay setting rejected without changing saved config: " + bad,
                       self.config.read_bytes() == before, reply)
        self.meteor("break-delay cooldown 3")
        saved = self.saved()
        self.check("BreakDelay cooldown persists in generated local JSON",
                   saved["break_delay"]["cooldown"] == 3 and not saved["break_delay_enabled"], saved)
        self.meteor("break-delay cooldown 0")

        # Survival no-insta-break: a held break must not finish; clicking still can.
        self.toggle("break-delay", True)
        self.meteor("break-delay no-insta-break true")
        self.ctl("command", "gamemode survival")
        self.ctl("command", "item replace entity @s weapon.mainhand with minecraft:netherite_pickaxe 1")
        self.ctl("release")
        self.fixture_fill(f"fill {x} {y+1} {z+2} {x} {y+1} {z+2} minecraft:stone")
        self.ctl("command", f"tp @s {x+0.5} {y} {z+0.5}")
        self.ctl("look", 0, 0)
        self.ctl("wait", 8)
        self.ctl("mouse", "left", "down")
        self.ctl("wait", 18)
        self.ctl("mouse", "left", "up")
        held_block = self.ctl("blocks", x, y+1, z+2, x, y+1, z+2)
        self.check("No-insta-break prevents a completed held survival break",
                   len(held_block["blocks"]) == 1, held_block)
        self.meteor("break-delay no-insta-break false")
        self.ctl("release")
        self.ctl("wait", 8)
        self.ctl("mouse", "left", "down")
        self.ctl("wait", 18)
        self.ctl("mouse", "left", "up")
        ordinary_block = self.ctl("blocks", x, y+1, z+2, x, y+1, z+2)
        self.check("Survival block breaks again when no-insta-break is disabled",
                   not ordinary_block["blocks"], ordinary_block)
        self.toggle("break-delay", False)
        self.ctl("command", "gamemode creative")

        # AutoRespawn: the real death screen, the host respawn button and one respawn per death.
        self.ctl("command", f"spawnpoint @s {x} {y} {z}")
        self.ctl("command", "gamemode survival")
        self.ctl("command", "kill @s")
        self.ctl("wait", 8)
        dead = self.state("auto_respawn_disabled")
        self.check("Disabled AutoRespawn leaves the death screen open", dead["screen"] == "dead", dead)
        self.ctl("respawn")
        self.ctl("wait", 12)
        manual = self.state("manual_respawn")
        self.check("Manual respawn still works while AutoRespawn is disabled",
                   manual["screen"] == "playing" and manual["player"]["health"] > 0, manual)

        self.toggle("auto-respawn", True)
        self.ctl("command", "kill @s")
        self.ctl("wait", 12)
        auto = self.state("auto_respawn_enabled")
        self.check("AutoRespawn returns an actual death to live singleplayer gameplay",
                   auto["screen"] == "playing" and auto["player"]["health"] > 0, auto)
        self.ctl("wait", 12)
        self.check("AutoRespawn leaves the respawned player alive and in play",
                   self.state("auto_respawn_after")["screen"] == "playing")
        saved = self.saved()
        self.check("AutoRespawn persists in generated local JSON", saved["auto_respawn_enabled"] is True, saved)

        # World lifecycle: settings survive leave/rejoin and AutoRespawn still acts in the new world session.
        world = self.report["launch"][self.report["launch"].index("--world") + 1]
        self.ctl("release")
        self.ctl("disconnect")
        deadline = time.monotonic() + 60
        while self.ctl("state").get("player") is not None:
            if time.monotonic() > deadline:
                raise TimeoutError("Generated world did not disconnect")
            time.sleep(0.2)
        self.ctl("open", world)
        deadline = time.monotonic() + 180
        while True:
            state = self.ctl("state")
            if state.get("player") and state.get("screen") == "playing":
                break
            if time.monotonic() > deadline:
                raise TimeoutError("Generated world did not reopen")
            time.sleep(0.5)
        rejoined = self.modules()
        self.check("AutoRespawn setting survives leaving and rejoining the generated world",
                   rejoined["auto_respawn"] and not rejoined["break_delay"], rejoined)
        self.ctl("command", "gamemode survival")
        self.ctl("command", "kill @s")
        self.ctl("wait", 12)
        again = self.state("auto_respawn_rejoined")
        self.check("AutoRespawn acts again after the world lifecycle reset",
                   again["screen"] == "playing" and again["player"]["health"] > 0, again)
        self.toggle("auto-respawn", False)
        self.ctl("command", "kill @s")
        self.ctl("wait", 8)
        final_dead = self.state("auto_respawn_final_disabled")
        self.check("Disabling AutoRespawn restores the death screen", final_dead["screen"] == "dead", final_dead)
        self.ctl("respawn")
        self.ctl("wait", 12)
        self.report["final_config"] = self.saved()

    def execute(self):
        code = super().execute()
        text = json.dumps(self.report, indent=2, ensure_ascii=False)
        (self.scratch / "report.json").write_text(text, encoding="utf8")
        (self.root / "standard-player-interactions-runtime-current.json").write_text(text, encoding="utf8")
        return code


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("client_binary", "control_binary", "resources", "scratch_root"):
        parser.add_argument("--" + name.replace("_", "-"), type=Path, required=True)
    parser.add_argument("--candidate-prefix", default=DEFAULT_PREFIX)
    args = parser.parse_args()
    for name in ("client_binary", "control_binary", "resources"):
        setattr(args, name, getattr(args, name).resolve(strict=True))
    args.skip_gui = True
    return PlayerInteractionsHarness(args).execute()


if __name__ == "__main__":
    raise SystemExit(main())
