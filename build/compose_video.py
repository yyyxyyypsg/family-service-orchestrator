#!/usr/bin/env python3
"""Compose demo video: real captured frames + caption bars + title/end cards."""
import os
import subprocess
from PIL import Image, ImageDraw, ImageFont

APP = r"C:\Users\lsy\Desktop\课程设计\agenticapp-hackathon\apps\family-orchestrator"
FRAMES = os.path.join(APP, "build", "frames")
OUTDIR = os.path.join(APP, "build", "video")
W, H = 618, 1338
BAR = 110
TH = H + BAR  # total 1448

FONT_PATH = r"C:\Windows\Fonts\msyh.ttc"
FONT_PATH_B = r"C:\Windows\Fonts\msyhbd.ttc"

def font(size, bold=False):
    path = FONT_PATH_B if bold and os.path.exists(FONT_PATH_B) else FONT_PATH
    return ImageFont.truetype(path, size)

# (frame file, duration s, caption or None for title/end special)
scenes = [
    (None,             8,  None),  # title card
    ("01-pasted.png", 15, "① 粘贴订单 / 配送 / 安装通知"),
    ("02-parsed.png", 20, "② 解析出带原文依据的事实 · 识别配送延期与时间冲突"),
    ("03-plan-a.png", 14, "③ 确认方案 A · 周日 15:00（本地日程候选）"),
    ("04-executed.png", 20, "④ 确认执行 · 更新本地时间线 · 登记复查 · 写入审计"),
    ("05-duplicate.png", 15, "⑤ 重复操作被幂等拦截 · 审计留痕"),
    ("06-partial.png", 18, "⑥ 模拟提醒失败 · 部分成功（改期已写入，提醒待重试）"),
    ("07-retried.png", 15, "⑦ 重试只补未完成动作 · 全部核验通过"),
    ("08-restored.png", 18, "⑧ 重启后状态与审计完整恢复（storage 持久化）"),
    (None,            10, None),  # end card
]

def caption_frame(png, text, step):
    img = Image.open(os.path.join(FRAMES, png)).convert("RGB")
    assert img.size == (W, H), img.size
    canvas = Image.new("RGB", (W, TH), (12, 24, 20))
    canvas.paste(img, (0, 0))
    d = ImageDraw.Draw(canvas)
    d.rectangle([0, H, W, TH], fill=(12, 24, 20))
    if step is not None:
        d.rectangle([16, H + 25, 16 + 60, H + 85], fill=(52, 199, 89))
        d.text((30, H + 33), str(step), font=font(36, bold=True), fill=(255, 255, 255))
        tx = 92
    else:
        tx = 20
    # shrink text to fit
    size = 30
    while size > 18:
        f = font(size)
        if d.textlength(text, font=f) <= W - tx - 16:
            break
        size -= 2
    d.text((tx, H + 38), text, font=font(size), fill=(255, 255, 255))
    return canvas

def title_card():
    img = Image.new("RGB", (W, TH), (12, 24, 20))
    d = ImageDraw.Draw(img)
    d.text((W // 2, 300), "家庭服务事件编排器", font=font(52, bold=True), fill=(255, 255, 255), anchor="mm")
    d.text((W // 2, 400), "dev.aster.fso · v0.1.0", font=font(30), fill=(160, 200, 180), anchor="mm")
    d.text((W // 2, 500), "GOSIM Agentic App 黑客松 · agent aigc", font=font(28), fill=(160, 200, 180), anchor="mm")
    d.text((W // 2, 620), "意图 → 事实 → 方案 → 确认 → 执行 → 核验", font=font(30, bold=True), fill=(52, 199, 89), anchor="mm")
    d.text((W // 2, 700), "17 状态机 · 防编造三层校验 · 幂等审计 · 失败重规划", font=font(26), fill=(220, 220, 220), anchor="mm")
    d.text((W // 2, 780), "纯本地规则引擎 · 不依赖设备 AI · 数据不出设备", font=font(26), fill=(220, 220, 220), anchor="mm")
    d.text((W // 2, 1300), "以下画面为 card-host 真实运行，经原生远程桥驱动逐帧捕获", font=font(22), fill=(120, 150, 140), anchor="mm")
    return img

def end_card():
    img = Image.new("RGB", (W, TH), (12, 24, 20))
    d = ImageDraw.Draw(img)
    d.text((W // 2, 350), "纯本地规则引擎", font=font(44, bold=True), fill=(255, 255, 255), anchor="mm")
    d.text((W // 2, 470), "事实全部可回溯原文 · 不依赖设备 AI", font=font(30), fill=(200, 200, 200), anchor="mm")
    d.text((W // 2, 590), "hub check: PASSED", font=font(34, bold=True), fill=(52, 199, 89), anchor="mm")
    d.text((W // 2, 700), "数据来源：用户粘贴文本 + 本地日程示例", font=font(28), fill=(200, 200, 200), anchor="mm")
    d.text((W // 2, 820), "声明：本应用不读取短信/聊天记录，不连接外部平台", font=font(26), fill=(150, 150, 150), anchor="mm")
    d.text((W // 2, 940), "agent aigc · yyyxyyypsg · 2026-10", font=font(26), fill=(150, 150, 150), anchor="mm")
    return img

os.makedirs(OUTDIR, exist_ok=True)

# write captioned stills
stills = []
step = 0
for png, dur, cap in scenes:
    if png is None and cap is None and "end" not in [s[0] for s in stills]:
        pass
for i, (png, dur, cap) in enumerate(scenes):
    if png is None:
        img = end_card() if i == len(scenes) - 1 else title_card()
        name = "card-title" if i == 0 else "card-end"
    else:
        step += 1
        img = caption_frame(png, cap, step)
        name = "scene-" + png.replace(".png", "")
    p = os.path.join(OUTDIR, name + ".png")
    img.save(p)
    stills.append((p, dur))
    print("rendered", name, dur, "s")

# concat list for ffmpeg (each still repeated to its duration via one entry + -loop? use concat with duration)
lst = os.path.join(OUTDIR, "list.txt")
total = 0
with open(lst, "w", encoding="utf-8", newline="\n") as f:
    for p, dur in stills:
        f.write("file '" + p.replace("\\", "/") + "'\n")
        f.write("duration " + str(dur) + "\n")
        total += dur
    f.write("file '" + stills[-1][0].replace("\\", "/") + "'\n")  # concat needs last file again
print("total duration:", total, "s")

import imageio_ffmpeg
ff = imageio_ffmpeg.get_ffmpeg_exe()
out = os.path.join(APP, "build", "video", "demo-dev.aster.fso.mp4")
cmd = [ff, "-y", "-f", "concat", "-safe", "0", "-i", lst,
       "-vf", "fps=15,format=yuv420p", "-c:v", "libx264", "-crf", "26",
       "-movflags", "+faststart", out]
r = subprocess.run(cmd, capture_output=True, text=True)
print("ffmpeg exit:", r.returncode)
if r.returncode != 0:
    print(r.stderr[-1200:])
else:
    print("video:", out, os.path.getsize(out) // 1024, "KB")
