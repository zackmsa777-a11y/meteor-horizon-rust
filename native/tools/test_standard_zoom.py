#!/usr/bin/env python3
"""Isolated runtime verification for the normal Meteor Zoom-only .hnmod proof.

Launches only when explicitly executed, with exact frozen binary paths and local
resources. Creates a new seed-1 creative world under a fresh UUID scratch home.
The native command transport is the actual chat screen; `command` is reserved
for authoritative integrated-server fixture commands. No legacy .meteor state.
"""
from __future__ import annotations

import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import re
import statistics
import subprocess
import time
import uuid

STATUS = re.compile(
    r"Zoom (enabled|disabled); requested ([^,]+), current ([0-9.eE+-]+) "
    r"\(native API clamps visible factor to 10\); smoothing (true|false);"
)
PREFIXES = ("Zoom-only .hnmod migration proof:", "Zoom-only migration proof:")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def plain(component):
    """Read rendered text components without treating style values as text."""
    if isinstance(component, str):
        return component
    if isinstance(component, list):
        return "".join(map(plain, component))
    if isinstance(component, dict):
        return plain(component.get("text", "")) + plain(component.get("extra", []))
    return ""


class Harness:
    def __init__(self, args):
        self.args = args
        self.root = args.scratch_root.resolve()
        self.root.mkdir(parents=True, exist_ok=True)
        self.scratch = self.root / ("standard-zoom-runtime-" + uuid.uuid4().hex)
        self.scratch.mkdir()
        self.home = self.scratch / "game"
        self.socket = self.scratch / "control.sock"
        self.env = os.environ.copy()
        folders = {
            "HORIZON_HOME": "game", "HORIZON_LAUNCHER_HOME": "launcher",
            "XDG_DATA_HOME": "data", "XDG_CONFIG_HOME": "config",
            "HORIZON_LAUNCHER_TRASH": "trash", "HORIZON_LAUNCHER_CLIPBOARD": "clipboard",
            "HORIZON_LAUNCHER_DESKTOP": "desktop", "TEMP": "temp", "TMP": "temp",
        }
        for name, relative in folders.items():
            path = self.scratch / relative
            path.mkdir(exist_ok=True)
            self.env[name] = str(path)
        self.env.update(HORIZON_LAUNCHER_NO_SELF_UPDATE="1", HORIZON_NO_AUDIO="1",
                        HORIZON_NO_DISCORD="1", HORIZON_TELEMETRY_URL="off")
        (self.home / "options.txt").write_text(
            "version:5023\npauseOnLostFocus:false\nmaxFps:60\nmouseSensitivity:0.5\n"
            "smoothCamera:false\nfov:70\n", encoding="utf8")
        self.config = self.home / "horizon-mods/.horizon-mod-data/meteor_client/zoom-migration.json"
        self.creationflags = subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0
        self.game = None
        self.log = None
        self.report = {
            "kind": "normal-hnmod-zoom-only", "runtime_approved": True,
            "started_utc": datetime.now(timezone.utc).isoformat(),
            "scratch": str(self.scratch), "checks": [], "states": {}, "commands": [],
            "client_binary": str(args.client_binary), "control_binary": str(args.control_binary),
            "client_sha256": sha256(args.client_binary), "control_sha256": sha256(args.control_binary),
            "resources": str(args.resources), "isolation": {k: self.env[k] for k in folders},
            "transport": {"meteor": "key t -> type /meteor-migration ... -> key enter",
                          "feedback": "messages.styled local rendered components, newest first",
                          "fixture": "command -> integrated-server console; tp @s authoritative"},
            "limitations": ["Zoom-only proof; other retained modules are not integrated.",
                            "Screenshots require manual visible FOV review; no pixel-parity assertion.",
                            "Persistence checks inspect generated local JSON, not a restarted process.",
                            "Cinematic/hide-HUD/show-hands are not integrated by this proof."],
        }

    def ctl(self, *words, timeout=45):
        result = subprocess.run(
            [str(self.args.control_binary), "--socket", str(self.socket), *map(str, words)],
            cwd=self.home, env=self.env, capture_output=True, text=True, encoding="utf8",
            timeout=timeout, creationflags=self.creationflags)
        if result.returncode:
            raise RuntimeError(f"control {words}: {result.stdout} {result.stderr}")
        data = json.loads(result.stdout)
        if data.get("error"):
            raise RuntimeError(f"control {words}: {data}")
        return data

    def check(self, name, condition, detail=None):
        self.report["checks"].append({"name": name, "passed": bool(condition), "detail": detail})
        print(f"{name}: {'PASS' if condition else 'FAIL'}", flush=True)
        if not condition:
            raise AssertionError(name)

    def state(self, name):
        value = self.ctl("state")
        self.report["states"][name] = value
        return value

    def lines(self):
        messages = self.ctl("messages", 100)
        if not isinstance(messages.get("styled"), list):
            raise RuntimeError("messages has no styled local-message list")
        return [plain(component) for component in messages["styled"]]

    def meteor(self, arguments):
        """Use the player chat path; console command/chat shortcuts bypass hooks."""
        before = Counter(line for line in self.lines() if line.startswith(PREFIXES))
        self.check_playing()
        self.ctl("key", "t", "tap")
        if self.ctl("state").get("screen") != "chat":
            raise RuntimeError("T did not open the actual chat screen")
        self.ctl("type", "/meteor-migration " + arguments)
        self.ctl("key", "enter", "tap")
        deadline = time.monotonic() + 8
        while True:
            lines = [line for line in self.lines() if line.startswith(PREFIXES)]
            after = Counter(lines)
            fresh = [line for line in lines if after[line] > before[line]]
            if fresh:
                response = fresh[0]
                self.report["commands"].append({"command": "/meteor-migration " + arguments,
                                                "response": response})
                self.check_playing()
                return response
            if time.monotonic() >= deadline:
                raise TimeoutError(f"No fresh native local-chat response to {arguments!r}: {lines}")
            self.ctl("wait", 1)

    def status(self):
        text = self.meteor("status")
        match = STATUS.search(text)
        if not match:
            raise RuntimeError("Native status schema mismatch: " + text)
        enabled, requested, current, smooth = match.groups()
        return {"enabled": enabled == "enabled", "requested": float(requested),
                "current": float(current), "smooth": smooth == "true", "text": text}

    def check_playing(self):
        value = self.ctl("state")
        if value.get("screen") != "playing" or not value.get("player"):
            raise RuntimeError("Expected fresh-world playing screen: " + json.dumps(value))
        return value

    def saved(self):
        if not self.config.is_file():
            raise RuntimeError("Generated native JSON was not created: " + str(self.config))
        return json.loads(self.config.read_text(encoding="utf-8-sig"))

    def settle(self):
        self.ctl("wait", 12)
        return self.status()

    def pose(self):
        x, y, z = self.origin
        self.ctl("command", f"tp @s {x+0.5} {y} {z+0.5}")
        self.ctl("wait", 4)
        self.ctl("look", 0, 0)
        self.ctl("wait", 3)

    def mouse_response(self, label):
        samples = []
        for _ in range(3):
            self.ctl("look", 0, 0)
            before = self.check_playing()["player"]["yaw"]
            moved = self.ctl("mouse-move", 80, 0)
            after = self.check_playing()["player"]["yaw"]
            delta = (after - before + 180) % 360 - 180
            samples.append({"before": before, "after": after, "delta": delta, "motion": moved})
        self.report.setdefault("mouse_samples", {})[label] = samples
        response = statistics.median(sample["delta"] for sample in samples)
        self.check(label + " mouse motion produces finite yaw", math.isfinite(response) and response > 0.1, samples)
        return response

    def screenshot(self, label):
        path = self.scratch / (label + ".png")
        self.ctl("screenshot", path)
        self.check(label + " screenshot captured", path.is_file() and path.stat().st_size > 0)
        self.report.setdefault("screenshots", {})[label] = str(path)

    def gui_registration(self):
        if self.args.skip_gui:
            self.report["gui"] = {"skipped": "explicit --skip-gui"}
            return
        try:
            self.ctl("options", "mods")
            # Options pauses game ticks. UI control still runs every frame;
            # waiting for simulation ticks here would time out by design.
            ui = self.ctl("ui")
            self.report["mods_ui"] = ui
            widgets = ui.get("widgets")
            if not isinstance(widgets, list):
                self.report["gui"] = {"skipped": "unknown ui schema", "reply": ui}
                return
            labels = [str(widget.get("label", "")) for widget in widgets]
            self.check("Native Meteor row registers in actual Options Mods UI",
                       any("Meteor Rust" in label and "Zoom migration proof" in label for label in labels), labels)
            self.screenshot("native-mods-row")
            self.report["gui"] = {"registered": True}
        except (RuntimeError, subprocess.TimeoutExpired) as error:
            self.report["gui"] = {"skipped": "optional GUI control unavailable", "error": repr(error)}
        finally:
            for _ in range(5):
                if self.ctl("state").get("screen") == "playing":
                    break
                self.ctl("key", "escape", "tap")
            self.check_playing()

    def run_checks(self):
        self.ctl("release")
        self.ctl("command", "difficulty peaceful")
        self.ctl("command", "gamemode creative")
        self.ctl("command", "time set 6000")
        initial = self.state("initial")
        self.check("Fresh generated world is creative", initial.get("gamemode") == "creative", initial.get("gamemode"))
        p = initial["player"]["position"]
        x, y, z = (math.floor(v) for v in p)
        self.origin = (x, y, z)
        # Fixed blocks in this generated world only, for visible FOV review.
        self.ctl("command", f"fill {x-8} {y-1} {z-8} {x+8} {y-1} {z+18} minecraft:stone")
        self.ctl("command", f"fill {x-8} {y} {z-8} {x+8} {y+9} {z+18} minecraft:air")
        self.ctl("command", f"fill {x-5} {y} {z+16} {x+5} {y+6} {z+16} minecraft:white_concrete")
        self.ctl("command", f"fill {x-5} {y} {z+16} {x-4} {y+6} {z+16} minecraft:red_concrete")
        self.ctl("command", f"fill {x+4} {y} {z+16} {x+5} {y+6} {z+16} minecraft:lime_concrete")
        self.ctl("command", f"fill {x} {y+1} {z+16} {x} {y+3} {z+16} minecraft:diamond_block")
        self.pose()
        self.report["fixture_origin"] = self.origin
        baseline = self.status()
        self.check("Zoom defaults disabled with factor6 and smoothingtrue",
                   not baseline["enabled"] and baseline["requested"] == 6 and
                   baseline["current"] == 1 and baseline["smooth"], baseline)
        self.screenshot("before")
        base_motion = self.mouse_response("baseline")
        self.pose()
        activation = self.meteor("zoom toggle")
        zoom = self.settle()
        self.check("Native Zoom reaches factor6", zoom["enabled"] and abs(zoom["current"]-6) < 0.002, zoom)
        self.check("Default smoothing remains enabled", zoom["smooth"], {"activation": activation, "settled": zoom})
        zoom_motion = self.mouse_response("zoom6")
        self.check("Actual mouse yaw sensitivity is divided by3 at factor6",
                   abs(zoom_motion/base_motion-1/3) < 0.035,
                   {"baseline": base_motion, "zoom": zoom_motion, "ratio": zoom_motion/base_motion})
        self.pose()
        self.screenshot("zoom")
        self.ctl("hotbar", 4)
        selected = self.ctl("inventory")["selected"]
        self.ctl("scroll-at", 160, 90, 1)
        scrolled = self.settle()
        self.check("Playing wheel changes actual Zoom factor to7.5",
                   abs(scrolled["current"]-7.5) < 0.002 and scrolled["requested"] == 6, scrolled)
        self.check("Zoom claims wheel before hotbar selection", self.ctl("inventory")["selected"] == selected)
        self.meteor("zoom toggle")
        restored = self.settle()
        self.check("Disable completes fade and restores factor1", not restored["enabled"] and restored["current"] == 1, restored)
        restored_motion = self.mouse_response("restored")
        self.check("Disable restores actual mouse yaw sensitivity",
                   abs(restored_motion/base_motion-1) < 0.035,
                   {"baseline": base_motion, "restored": restored_motion})
        self.pose()
        self.screenshot("restored")
        self.meteor("zoom factor 4")
        self.meteor("zoom smooth false")
        self.meteor("zoom scroll-sensitivity 0.5")
        saved = self.saved()
        self.check("Native local JSON persists factor smoothing sensitivity and disabled state",
                   saved["schema"] == 1 and not saved["enabled"] and saved["zoom"]["zoom"] == 4 and
                   saved["zoom"]["smooth"] is False and saved["zoom"]["scroll_sensitivity"] == 0.5, saved)
        for argument in ["zoom factor NaN", "zoom factor inf", "zoom factor 0", "zoom factor nope",
                         "zoom scroll-sensitivity -1", "zoom scroll-sensitivity NaN", "zoom smooth nope"]:
            before_bytes = self.config.read_bytes()
            response = self.meteor(argument)
            self.check("Invalid setting rejected safely: " + argument,
                       response.startswith("Zoom-only migration proof:") and self.config.read_bytes() == before_bytes,
                       response)
        valid = self.status()
        self.check("Invalid inputs preserve valid runtime state", not valid["enabled"] and
                   valid["requested"] == 4 and valid["current"] == 1 and not valid["smooth"], valid)
        self.meteor("zoom toggle")
        nonsmooth = self.settle()
        self.check("Smoothingfalse factor4 reaches native output", nonsmooth["enabled"] and
                   nonsmooth["current"] == 4 and not nonsmooth["smooth"], nonsmooth)
        motion4 = self.mouse_response("zoom4_nonsmooth")
        self.check("Actual mouse sensitivity follows new factor4", abs(motion4/base_motion-0.5) < 0.035,
                   {"baseline": base_motion, "zoom4": motion4})
        self.meteor("zoom toggle")
        final = self.settle()
        self.check("Final disable restores nonsmooth factor1", not final["enabled"] and final["current"] == 1, final)
        self.gui_registration()
        self.report["final_config"] = self.saved()
        self.state("final")

    def execute(self):
        try:
            self.log = (self.scratch / "process.log").open("w", encoding="utf8")
            command = [str(self.args.client_binary), "--resources", str(self.args.resources),
                       "--world", "MeteorStandardZoomTest", "--seed", "1", "--creative", "--test-terrain",
                       "--render-distance", "2", "--offscreen", "--width", "960", "--height", "540",
                       "--control-socket", str(self.socket)]
            self.report["launch"] = command
            self.game = subprocess.Popen(command, cwd=self.home, env=self.env, stdout=self.log,
                                         stderr=self.log, creationflags=self.creationflags)
            self.report["pid"] = self.game.pid
            print(f"Isolated normal .hnmod Zoom PID {self.game.pid}; {self.scratch}", flush=True)
            deadline, progress = time.monotonic()+180, 0
            while True:
                if self.game.poll() is not None:
                    raise RuntimeError(f"Client exited during startup: {self.game.returncode}")
                try:
                    current = self.ctl("state", timeout=5)
                    if current.get("player") and current.get("screen") == "playing":
                        break
                except (RuntimeError, subprocess.TimeoutExpired, json.JSONDecodeError):
                    pass
                if time.monotonic() > deadline:
                    raise TimeoutError("Fresh test world did not load within180seconds")
                if time.monotonic()-progress > 20:
                    print("Waiting for generated singleplayer world...", flush=True)
                    progress = time.monotonic()
                time.sleep(1)
            self.run_checks()
            self.report["checks_completed"] = True
        except Exception as error:
            self.report["error"] = repr(error)
            print(f"Runtime verification failed: {error}", flush=True)
            if self.game is not None and self.game.poll() is None:
                try:
                    self.report["failure_state"] = self.ctl("state", timeout=5)
                    self.ctl("screenshot", self.scratch / "failure.png", timeout=15)
                    self.report["failure_messages"] = self.ctl("messages", 100, timeout=5)
                except Exception as diagnostic:
                    self.report["diagnostic_error"] = repr(diagnostic)
        finally:
            if self.game is not None and self.game.poll() is None:
                try:
                    self.ctl("release", timeout=5)
                    self.ctl("quit", timeout=15)
                    self.report["graceful_quit_requested"] = True
                except Exception as error:
                    self.report["quit_error"] = repr(error)
                try:
                    self.game.wait(timeout=30)
                except subprocess.TimeoutExpired:
                    self.report["forced_shutdown"] = True
                    self.game.terminate()
                    try:
                        self.game.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        self.game.kill()
                        self.game.wait(timeout=10)
            self.report["exit_code"] = self.game.returncode if self.game is not None else None
            if self.log is not None:
                self.log.close()
            self.report["client_sha256_after"] = sha256(self.args.client_binary)
            self.report["control_sha256_after"] = sha256(self.args.control_binary)
            self.report["checks"].append({"name": "Graceful own-client shutdown exits0", "passed":
                self.report.get("graceful_quit_requested", False) and not self.report.get("forced_shutdown", False)
                and self.report["exit_code"] == 0})
            self.report["checks"].append({"name": "Exact frozen binaries remained unchanged", "passed":
                self.report["client_sha256"] == self.report["client_sha256_after"] and
                self.report["control_sha256"] == self.report["control_sha256_after"]})
            self.report["passed"] = bool(self.report.get("checks_completed") and
                all(check["passed"] for check in self.report["checks"]))
            self.report["finished_utc"] = datetime.now(timezone.utc).isoformat()
            text = json.dumps(self.report, indent=2, ensure_ascii=False)
            (self.scratch / "report.json").write_text(text, encoding="utf8")
            (self.root / "standard-zoom-runtime-current.json").write_text(text, encoding="utf8")
            print(f"Report: {self.scratch / 'report.json'}; passed={self.report['passed']}", flush=True)
        return 0 if self.report["passed"] else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--client-binary", type=Path, required=True, help="Exact frozen proof client executable")
    parser.add_argument("--control-binary", type=Path, required=True, help="Exact frozen control CLI executable")
    parser.add_argument("--resources", type=Path, required=True, help="Prepared local resources; never distributed")
    parser.add_argument("--scratch-root", type=Path, required=True, help="Separate scratch directory for fresh test homes and reports")
    parser.add_argument("--skip-gui", action="store_true", help="Skip optional native Options Mods row inspection")
    args = parser.parse_args()
    for name in ["client_binary", "control_binary", "resources"]:
        path = getattr(args, name).resolve(strict=True)
        if (path.is_dir() if name == "resources" else path.is_file()) is False:
            parser.error(name.replace("_", "-") + " has the wrong path type")
        setattr(args, name, path)
    return Harness(args).execute()


if __name__ == "__main__":
    raise SystemExit(main())
