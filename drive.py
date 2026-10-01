#!/usr/bin/env python3
"""Remote-bridge driver for card-host (Windows-safe).

Usage:
  python drive.py <port> state                 # print badge + replan + last log line
  python drive.py <port> click <button text>   # click the first widget whose text matches
  python drive.py <port> seq <label>,<label>,… # click each, print state after every click
  python drive.py <port> shot <out.png>        # save a screenshot
"""
import json
import sys
import time
import urllib.request

BASE = "http://127.0.0.1:{}"


def get(port, path):
    with urllib.request.urlopen(BASE.format(port) + path, timeout=10) as r:
        return r.read()


def snap(port):
    return json.loads(get(port, "/snap").decode("utf-8"))


def find(port, text):
    widgets = snap(port)["s"]
    # Prefer the exact Button label.  Substring-first matching can click an
    # audit Label such as “提醒已创建，核验通过” instead of the “核验通过” chip.
    for exact_button in (True, False):
        for w in widgets:
            t = w.get("t")
            is_button = w.get("ty") in ("Button", "DesktopButton")
            matches = t == text if exact_button else (t and text in t)
            if t and matches and (is_button if exact_button else w.get("ty") in ("Button", "DesktopButton", "Label", "View")):
                x, y, ww, hh = w["r"]
                if ww > 0 and hh > 0 and w.get("enabled", True):
                    return (int(x + ww / 2), int(y + hh / 2), t)
    return None


def click(port, x, y):
    get(port, f"/click?x={x}&y={y}&wait=1")


def badge(port):
    for w in snap(port)["s"]:
        if w.get("i") == "state_badge":
            return w.get("t", "")
    return "?"


def replan(port):
    for w in snap(port)["s"]:
        if w.get("i") == "replan_label":
            return w.get("t", "")
    return "?"


def main():
    port = sys.argv[1]
    cmd = sys.argv[2]
    if cmd == "state":
        print(badge(port), "|", replan(port))
    elif cmd == "click":
        hit = find(port, sys.argv[3])
        if not hit:
            print(f"NOT FOUND: {sys.argv[3]!r}")
            sys.exit(1)
        click(port, hit[0], hit[1])
        print(f"clicked {hit[2]!r} at {hit[:2]} ->", badge(port))
    elif cmd == "seq":
        for label in sys.argv[3].split(","):
            hit = find(port, label)
            if not hit:
                print(f"NOT FOUND: {label!r}")
                sys.exit(1)
            click(port, hit[0], hit[1])
            time.sleep(0.15)
            print(f"{label:<8} -> {badge(port)} | {replan(port)}")
    elif cmd == "shot":
        out = sys.argv[3]
        data = get(port, "/g?raw=1")
        with open(out, "wb") as f:
            f.write(data)
        print(f"wrote {out} ({len(data)} bytes)")
    else:
        print(__doc__)
        sys.exit(1)


if __name__ == "__main__":
    main()
