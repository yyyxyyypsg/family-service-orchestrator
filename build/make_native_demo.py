from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
import subprocess, imageio_ffmpeg

root = Path(r'C:\Users\lsy\Desktop\课程设计\agenticapp-hackathon\apps\family-orchestrator')
indir = root / 'build' / 'video' / 'native'
outdir = root / 'build' / 'video'
frames = outdir / 'native-frames'
frames.mkdir(parents=True, exist_ok=True)
font_paths = [r'C:\Windows\Fonts\msyh.ttc', r'C:\Windows\Fonts\simhei.ttf', r'C:\Windows\Fonts\arial.ttf']
font_path = next((p for p in font_paths if Path(p).exists()), None)
font = ImageFont.truetype(font_path, 34) if font_path else ImageFont.load_default()
small = ImageFont.truetype(font_path, 24) if font_path else ImageFont.load_default()

def card(name, title, subtitle=''):
    im = Image.new('RGB',(1400,900),(11,24,42)); d=ImageDraw.Draw(im)
    d.rectangle((0,0,1400,18),fill=(49,180,173)); d.rectangle((0,882,1400,900),fill=(49,180,173))
    d.text((90,300),title,font=ImageFont.truetype(font_path,56) if font_path else font,fill=(242,247,250))
    if subtitle: d.text((94,390),subtitle,font=font,fill=(169,207,214))
    d.text((94,790),'Rust · Makepad · OctoSense · Octos peer',font=small,fill=(130,158,177))
    im.save(frames/name)

def scene(src, name, caption, detail):
    im=Image.open(indir/src).convert('RGB')
    d=ImageDraw.Draw(im)
    d.rectangle((0,785,1400,900),fill=(8,20,32))
    d.text((55,806),caption,font=font,fill=(244,248,250))
    d.text((55,854),detail,font=small,fill=(166,204,212))
    im.save(frames/name)

card('00-title.png','家庭服务事件编排器','OctoSense 原生宿主扩展演示')
scene('01-idle.png','01-idle.png','1  原生模块已加载','OctoSense Desktop · family-orchestrator')
scene('02-notice.png','02-notice.png','2  输入家庭配送与安装通知','通知内容保留在应用本地，等待解析')
scene('03-parsed.png','03-parsed.png','3  解析事实并生成方案','每条事实都保留可回溯的原文引用')
scene('04-following.png','04-following.png','4  用户确认后执行','更新本地服务时间线，并登记审计记录')
card('05-peer.png','Octos peer 已接通','system agent → peer_send_input → parse_notice\n返回 facts=4 · missing=0 · AwaitingConfirm')
card('06-end.png','谢谢观看','家庭服务事件编排器 · agent aigc')

order=[('00-title.png',4),('01-idle.png',5),('02-notice.png',6),('03-parsed.png',8),('05-peer.png',6),('04-following.png',8),('06-end.png',4)]
listfile=outdir/'native-list.txt'
with listfile.open('w',encoding='utf-8') as f:
    for n,dur in order:
        f.write("file '%s'\n" % (frames/n).as_posix())
        f.write('duration %s\n' % dur)
    f.write("file '%s'\n" % (frames/order[-1][0]).as_posix())
video=outdir/'demo-native-host-extension.mp4'
ff=imageio_ffmpeg.get_ffmpeg_exe()
cmd=[ff,'-y','-f','concat','-safe','0','-i',str(listfile),'-vf','scale=1280:-2,format=yuv420p','-r','30','-c:v','libx264','-preset','medium','-crf','20','-movflags','+faststart',str(video)]
subprocess.run(cmd,check=True)
print(video)
print(video.stat().st_size)
