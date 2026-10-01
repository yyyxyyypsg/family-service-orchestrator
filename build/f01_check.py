#!/usr/bin/env python3
"""F-01 regression: delivery_status source_quote must be the exact matched phrase."""
import json
import os
import subprocess
import time
import urllib.parse
import urllib.request

PORT = 8142
BASE = f"http://127.0.0.1:{PORT}"
WS = r"C:\Users\lsy\Desktop\课程设计\agenticapp-hackathon"
APP = os.path.join(WS, "apps", "family-orchestrator")
FLOW = os.path.join(WS, "OctoScript-App-Design-Flow")
STATE = os.path.join(APP, ".local-state", "dev.aster.fso", "state.json")
os.environ["OCTO_HUB"] = os.path.join(WS, "OctoSense-App-Hub", "target", "release", "hub.exe")
os.environ["OCTO_CARD_HOST"] = os.path.join(WS, "OctoSense-App-Hub", "target", "release", "card-host.exe")

def get(path):
    with urllib.request.urlopen(BASE + path, timeout=15) as r:
        return r.read()

def snap():
    return json.loads(get("/snap").decode("utf-8"))

def click_id(wid):
    for w in snap()["s"]:
        if w.get("i") == wid:
            x, y, ww, hh = w["r"]
            get(f"/click?x={int(x+ww/2)}&y={int(y+hh/2)}&wait=1")
            time.sleep(0.3)
            return
    raise RuntimeError("widget not found: " + wid)

def click_text(text):
    for w in snap()["s"]:
        t = w.get("t")
        if t and text in t and w.get("ty") == "Button":
            x, y, ww, hh = w["r"]
            get(f"/click?x={int(x+ww/2)}&y={int(y+hh/2)}&wait=1")
            time.sleep(0.3)
            return
    raise RuntimeError("button not found: " + text)

def type_text(s):
    get("/t?t=" + urllib.parse.quote(s) + "&wait=1")
    time.sleep(0.3)

def delivery_status_fact():
    st = json.load(open(STATE, encoding="utf-8"))
    for f in st["parsed"]["facts"]:
        if f["field"] == "delivery_status":
            return f
    return None

cases = [
    ("订单号：AC-1\n配送预计：9月28日\n安装预约：9月27日 15:00\n配送延迟", "延迟"),
    ("订单号：AC-2\n配送预计：9月28日\n安装预约：9月27日 15:00\n到货推迟", "推迟"),
    ("订单号：AC-3\n配送预计：9月28日（配送延期）\n安装预约：9月27日 15:00", "延期"),
]

# ensure fresh app
subprocess.run(["taskkill", "/IM", "card-host.exe", "/F"], capture_output=True)
time.sleep(2)
subprocess.run(["python", "tools/octo", "run", os.path.join(APP, "bundle"), "--port", str(PORT),
                "--hidden", "--detach"], cwd=FLOW, capture_output=True, timeout=120)
time.sleep(1.5)

ok = True
for notice, expect in cases:
    click_text("重置全部")
    click_id("input_area")
    type_text(notice)
    click_text("解析通知")
    f = delivery_status_fact()
    if f is None:
        print(f"FAIL  {expect}: no delivery_status fact produced")
        ok = False
        continue
    exact = f["source_quote"] == expect
    # quote must also literally occur in the input
    in_input = f["source_quote"] in notice
    print(f"{'PASS' if exact and in_input else 'FAIL'}  输入含「{expect}」-> source_quote={f['source_quote']!r} "
          f"(exact={exact}, occurs_in_input={in_input})")
    ok = ok and exact and in_input

get("/quit")
print("F-01 RESULT:", "FIXED" if ok else "STILL BROKEN")
