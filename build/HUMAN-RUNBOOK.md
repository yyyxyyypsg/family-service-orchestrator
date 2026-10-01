# HUMAN-RUNBOOK — dev.aster.fso 0.1.0 签名与提交（Aster 专用）

> 全部命令已在 2026-10-01 由内环核对过路径与参数。内环只在你说"go"后代跑带外效果的步骤
> （打 tag、开 issue）；keygen/sign 建议你亲手执行（密钥不经过任何 agent 会话）。
> 前提：bundle 以**当前工作树**为准（与提交 `10bc914` 字节一致，含外环的 yyyxyyypsg 修正），
> `octo check` 已 PASSED（仅未签名警告）。

## 0. 环境（一次性，新开终端需要）

```bat
set OCTO_HUB=C:\Users\lsy\Desktop\课程设计\agenticapp-hackathon\OctoSense-App-Hub\target\release\hub.exe
set OCTO_CARD_HOST=C:\Users\lsy\Desktop\课程设计\agenticapp-hackathon\OctoSense-App-Hub\target\release\card-host.exe
set BUNDLE=C:\Users\lsy\Desktop\课程设计\agenticapp-hackathon\apps\family-orchestrator\bundle
```

（Git Bash 用 `source env.sh`，env.sh 在 `agenticapp-hackathon\`。）

## 1. 生成发布者密钥（一次，放在任何仓库之外）

```bat
%OCTO_HUB% keygen
```

按它打印的说明保存密钥文件（建议 `C:\Users\lsy\octosense-keys\`，不要放进任何仓库）。
记下输出里的 **publisher-id** 和**公钥 hex**（提交 issue 要用公钥，私钥谁都不给）。

## 2. 签名 + 复核

```bat
%OCTO_HUB% sign-manifest %BUNDLE% --key <密钥文件路径> --key-id <publisher-id>
%OCTO_HUB% check %BUNDLE% --publisher-key <publisher-id>=<公钥hex>
```

预期：第二行输出 `dev.aster.fso 0.1.0 — PASSED` 且**无** unsigned 警告。
签名后任何文件改动都要重新 stamp + 签名。

## 3. 固化提交 + 打 tag

```bat
cd C:\Users\lsy\Desktop\课程设计\agenticapp-hackathon\apps\family-orchestrator
git add -A
git commit -m "release: dev.aster.fso 0.1.0 signed bundle"
git tag v0.1.0
```

## 4. 推送到公开仓库（GitHub: yyyxyyypsg 名下，仓库建议名 family-service-orchestrator）

```bat
git remote add origin https://github.com/yyyxyyypsg/family-service-orchestrator.git
git push -u origin main --tags
```

## 5. 开提交 Issue（OctoSense-App-Hub 仓库，或按赛方指定渠道）

标题：`Submit dev.aster.fso 0.1.0`

正文模板（<> 处替换为实际值）：

```
Repo: https://github.com/yyyxyyypsg/family-service-orchestrator
Tag: v0.1.0
Commit: <git rev-parse HEAD 的输出>
Bundle path: bundle/
Publisher: agent aigc
Publisher key: <publisher-id>=<公钥hex>

hub check output:
<粘贴 %OCTO_HUB% check %BUNDLE% --publisher-key 的完整输出>

hub scan: build/review.json + build/REVIEW-ANSWERS.md（七问已答，见仓库）

What it does: 粘贴空调配送/安装通知，解析带原文依据的事实，识别延期与时间冲突，
给出可确认的改期+复查提醒方案，确认后更新本地时间线并审计；支持部分成功重试、
重复拦截、重规划与规则兜底。纯本地规则引擎，仅 storage 权限。
Platforms tested: Windows（card-host + 远程桥，真实截图 bundle/screenshots/01-03）
```

> 赛方备选：比赛提交走主办方指定仓库/表单时，同样附上 tag、commit、check 输出、
> scan 答案、演示视频（2-3 分钟，另录）与两张关键截图。

## 6. 黑客松初赛清单对照（8 项）

| # | 项 | 状态 |
|---|---|---|
| 1 | 可运行原型+启动说明 | ✓ card-host 可跑；启动说明=本文件 §0 + README（待补一页） |
| 2 | Apache 2.0 固定版本源码 | ✓ LICENSE 已含；推送后即公开 |
| 3 | 2-3 分钟演示视频 | ✗ 待录（脚本要点见下） |
| 4 | 两张关键截图 | ✓ bundle/screenshots/01、02（03 备用） |
| 5 | 简短需求说明 | ✓ BRIEF.md |
| 6 | 数据来源与限制 | ✓ BRIEF.md「数据（Data）」节 + REVIEW-ANSWERS §1 |
| 7 | 一次操作+可核对结果+一个失败状态 | ✓ 解析→确认→时间线变更（state.json 可查）；Partial 失败态 |
| 8 | 已报名成员名单 | ✗ 待人环填（Aster 等，注意与群内报名一致） |

演示视频脚本建议（≤3 分钟）：粘贴完整样例 → 解析（事实卡+依据）→ 冲突与方案 A/B →
确认执行（时间线变 done + 审计）→ 重复确认拦截 → 模拟失败（Partial）→ 重试 →
重启恢复 → 结尾一句"纯规则引擎，事实全部可回溯原文"。
