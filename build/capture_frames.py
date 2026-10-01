#!/usr/bin/env python3
"""Capture real-state frames for the demo video via the card-host remote bridge."""
import json
import os
import shutil
import subprocess
import sys
import time
import urllib.request
import urllib.parse

PORT = 8144
BASE = f"http://127.0.0.1:{PORT}"
FLOW = r"C:\Users\lsy\Desktop\课程设计\agenticapp-hackathon\OctoScript-App-Design-Flow"
WS = r"C:\Users\lsy\Desktop\课程设计\agenticapp-hackathon"
APP = os.path.join(WS, "apps", "family-orchestrator")
OUT = os.path.join(APP, "build", "frames")
os.environ["OCTO_HUB"] = os.path.join(WS, "OctoSense-App-Hub", "target", "release", "hub.exe")
os.environ["OCTO_CARD_HOST"] = os.path.join(WS, "OctoSense-App-Hub", "target", "release", "card-host.exe")

def get(path):
    with urllib.request.urlopen(BASE + path, timeout=15) as r:
        return r.read()

def snap():
    return json.loads(get("/snap").decode("utf-8"))

def click_text(text):
    for w in snap()["s"]:
        t = w.get("t")
        if t and text in t and w.get("ty") in ("Button", "GestureView", "Label", "RoundedView"):
            x, y, ww, hh = w["r"]
            if ww > 0 and hh > 0:
                get(f"/click?x={int(x+ww/2)}&y={int(y+hh/2)}&wait=1")
                time.sleep(0.35)
                return True
    raise RuntimeError("button not found: " + text)

def shot(name):
    data = get("/g?raw=1")
    os.makedirs(OUT, exist_ok=True)
    p = os.path.join(OUT, name + ".png")
    with open(p, "wb") as f:
        f.write(data)
    print("captured", name, len(data), "bytes")

def quit_app():
    try: get("/quit")
    except Exception: pass
    time.sleep(2)

def start_app():
    subprocess.run(["python", "tools/octo", "run", os.path.join(APP, "bundle"),
                    "--port", str(PORT), "--hidden", "--detach"],
                   cwd=FLOW, capture_output=True, timeout=120)
    time.sleep(1.5)

def type_notice():
    for w in snap()["s"]:
        if w.get("i") == "input_area":
            x, y, ww, hh = w["r"]
            get(f"/click?x={int(x+ww/2)}&y={int(y+hh/2)}&wait=1")
            txt = urllib.parse.quote("订单号：AC-20260927\n配送预计：9月28日（配送延期）\n安装预约：9月27日 15:00\n安装地址：深圳市南山区科苑路1号")
            get(f"/t?t={txt}&wait=1")
            time.sleep(0.35)
            return
    raise RuntimeError("notice input not found")

# ---- fresh session ----
subprocess.run(["taskkill", "/IM", "card-host.exe", "/F"], capture_output=True)
time.sleep(2)
shutil.rmtree(os.path.join(APP, ".local-state"), ignore_errors=True)
start_app()

shot("00-idle")                       # idle, clean
type_notice()
shot("01-pasted")                     # notice typed
click_text("解析通知")
shot("02-parsed")                     # facts + plans + AwaitingConfirm
click_text("方案 A")
shot("03-plan-a")                     # plan A explicitly chosen (msg shows selection)
click_text("确认执行")
shot("04-executed")                   # Following + timeline + audit
click_text("确认执行")
shot("05-duplicate")                  # duplicate intercepted

# fresh re-parse for partial path
click_text("重置全部")
type_notice()
click_text("解析通知")
click_text("模拟提醒失败")
shot("06-partial")                    # Partial
click_text("确认执行")
shot("07-retried")                    # Following again

# restart persistence
quit_app()
start_app()
shot("08-restored")                   # restored from state.json

quit_app()
print("ALL FRAMES CAPTURED")
