# 交接说明 · 给 Codex / GPT-6（外环复验方）

> 生成：2026-10-01 · 交接方：ZCode 内环（GLM 5.3）· 接收方：Codex（外环，负责独立复验与裁定）
> 本文件是自包含交接件：拿到本文件即可复现全部结论，不需要额外口头背景。

---

## 0. 一句话状态

GOSIM Agentic App 黑客松 2026 参赛作品《家庭服务事件编排器》（app id `dev.aster.fso`）**已完成开发、验收、签名与初赛提交**；内环任务 T1/T4/T5 均已由外环复验通过；初赛材料齐备（仅成员名单其余成员、群内三问待人环）。

## 1. 如何取得全部东西

| 方式 | 位置 |
| --- | --- |
| **公开仓库（推荐，可直接 clone）** | https://github.com/yyyxyyypsg/family-service-orchestrator |
| 提交版本 | tag `v0.1.0`；commit 见 `git rev-parse v0.1.0`（本地为 37a8eb3 之后的最新 main） |
| **本地工作副本** | `C:\Users\lsy\Desktop\课程设计\agenticapp-hackathon\apps\family-orchestrator` |
| 官方运行时工作区（构建工具） | `C:\Users\lsy\Desktop\课程设计\agenticapp-hackathon\`（含 OctoSense-App-Hub / OctoScript-App-Design-Flow / makepad / octoscript / octoscript-makepad） |
| 初赛提交记录 | https://github.com/gosimfoundation/hackathon-agenticapp26/issues/13 （comment 5931865738） |
| 参赛主题登记（用户本人此前提交） | 同仓库 issue #5（队伍名 agent aigc / 成员 id Aster / 赛道购物与物流） |

> ⚠️ **不要把发布者私钥交给任何 agent 或任何人**。它位于 `C:\Users\lsy\octosense-keys\publisher.key`（仓库外），复验**不需要**私钥；只用公钥即可（见 §3）。

## 2. 作品是什么

OctoSense **脚本应用**（`bundle/main.splash`，单文件 Splash 程序），非卡片应用、无原生代码。

**做什么**：粘贴一条空调配送/安装通知 → 解析出**带原文引用**的事实（订单号/配送日期/安装时间/地址）→ 识别配送延期与时间冲突 → 生成可确认的改派方案（本地日程候选 A/B）→ 用户确认后更新**本地服务时间线**、登记售后复查、写审计 → 支持部分成功重试、重复操作幂等拦截、重规划封顶（Replan→Unstable@3）、规则兜底。

**不做什么**（重要边界）：不联网、不声明任何 host、不调用设备 AI/助手（设备上也不可用）、不声称修改任何外部物流/安装/支付系统。全部动作落在本地时间线与 `state.json`。

**形态来源**：从已通过 21 项测试的 JS 原型（`C:\Users\lsy\Desktop\codex项目文件\family-service-orchestrator\`，含 `domain.js`/`parser.js`/`store.js`/`app.js`）逐行移植到 Splash，语义对齐由外环复验过。

## 3. 已完成的验证（及如何自己重跑）

环境准备（Windows；产物为 `.exe`，需显式指向）：

```sh
export OCTO_HUB="C:/Users/lsy/Desktop/课程设计/agenticapp-hackathon/OctoSense-App-Hub/target/release/hub.exe"
export OCTO_CARD_HOST="C:/Users/lsy/Desktop/课程设计/agenticapp-hackathon/OctoSense-App-Hub/target/release/card-host.exe"
```

| 验证 | 命令 | 结果 |
| --- | --- | --- |
| 工具链 | `python tools/octo doctor`（在 OctoScript-App-Design-Flow 下） | 全绿 |
| 准入检查 | `"$OCTO_HUB" check bundle --publisher-key "yyyxyyypsg=33818b0b907191c005de59d83c521c58e23e78635df3e0f7139e3b051040c8cd"` | `dev.aster.fso 0.1.0 — PASSED`，无未签名警告 |
| 无头运行 | `python tools/octo run <repo>/bundle --port 8142 --hidden --detach` | `admitted` + `ready: first frame drawn`，日志 0 个 `[E]` |
| 状态机 7 序列 | `python drive.py 8142 seq "<中文标签>,…"`（序列见 `OUTER_LOOP_REVIEW.md` T1/T5 节） | 全部终态与重规划计数符合预期 |
| 端到端 | 解析→选方案→确认→重复确认拦截→模拟失败→重试→重启恢复 | 全通过；`state.json` 数据级核验 |
| 截图 | `bundle/screenshots/01-main.png`、`02-executed.png`、`03-partial.png` | 真实捕获，人工查验过 |

复验工具：仓库内 `drive.py`（远程桥驱动：`state` / `click <文字>` / `seq <标签,…>` / `shot <out.png>`），无需 Computer Use。

## 4. 外环此前的裁定（已记录在案）

见仓库 `OUTER_LOOP_REVIEW.md`：

- **T1** 状态机移植 —— `verified`（售后序列已按真实状态机校正：`Completed → register_postsale → PostSale → postsale_expired → Idle`；`postsale_handled → Completed` 后直接过期会被正确拒绝）
- **T2/T3** 解析/存储/执行全流程 —— `verified`（外环亲自重放，端口 8151/8152）
- **T4** 发布材料与视觉收尾 —— `verified`
- **T5** 调试事件可驱动性 —— `verified`（`/snap` 31 个按钮，22 事件全部可见可点；外环另修了 `drive.py` 的包含匹配误点问题）
- 基线裁定：课程工作区完整版为**唯一基线**；`family-service-orchestrator-octoscript` 下的精简 scaffold 退出发布路径（见该目录 `CANONICAL_BASE.md`）

## 5. 未完成 / 待决策（如实列出）

| 项 | 状态 | 归属 |
| --- | --- | --- |
| 成员名单其余成员 | `build/TEAM.md` 仅 `Aster = yyyxyyypsg` 确认（与 issue #5 一致），其余待补 | 人环 |
| 群内三问：团队上限 5 还是 6 / 「网页小程序」初赛是否认可 / 决赛演示方式（评审自跑 or 共享屏幕） | 未回 | 人环 |
| App Hub **商店上架**（OctoSense-App-Hub 的 `Submit dev.aster.fso 0.1.0` issue） | **未做**。赛事文档明确「现阶段以公开源码仓库和可运行作品为准，**无需等待 Hub 上架**」；该 issue 属商店路径，首次提交需维护者注册发布者密钥。材料已备：`build/SUBMIT-ISSUE.md` | 待决策 |
| `hub scan` 审核包 | 已生成 `build/review.json`（7 问）；**签名后**对已签名包重跑会报 `publisher key "yyyxyyypsg" is not registered with this hub`（首次发布者的预期状态） | 已知，非缺陷 |

## 6. 需要你知道的偏差与风险（诚实标注）

1. **平台**：官方文档标注已验证平台为 Apple silicon macOS，**Windows 未验证**。本作品在 Windows 上完整跑通（构建/准入/无头运行/远程桥/持久化），但这是我们的实测结论，不是官方保证。
2. **调试控制台常驻**：界面底部保留「调试事件（22 事件直注）+ 样例填充」，用于评审复现全部 17 状态。已在 `listing.json` 描述中声明"正式产品版本将隐藏该区域"。审核建议为 `human-review`（见 `build/REVIEW-ANSWERS.md` 第 7 问）。
3. **`local_time()` 在 card-host 是 UTC**：应用内做 +8 小时偏移以显示产品域时间（审计行时间戳）。
4. **`now_hhmm` 为展示用**：不参与任何判定逻辑。
5. **历史缺陷已修**：`.gitignore` 的 `*.png` 曾把 `bundle/screenshots/` 排除出版本控制（导致仓库里 `listing.json` 引用的截图缺失）；已改为只忽略 `debug-t1*.png`，截图入库，签名仍有效（bundle 字节未变，check 复跑 PASSED）。
6. **`hub keygen --help` 曾误把 `--help` 当输出路径**，在仓库根落过一个私钥文件；**已立即删除**且该密钥从未被使用（对应公钥未登记、未签名任何东西）。正式密钥在仓库外。
7. **移植期的 5 个真实坑**（供你审阅同类移植）：`ok` 是保留字不能作对象字段名；`const` 不是关键字（只有 `let/var/mut`）；regex `exec` 的 `captures[0]` 是**整体匹配**、捕获组从 `[1]` 起、未参与组为 `nil` 且数组长度固定；对象动态键读取缺失键是**错误**而非 nil（集合用数组+线性查找）；跨 shell 层写正则要防转义被吃掉（`\s` 必须 `\\s`）。

## 7. 想请外环做的事

1. **独立复验签名包**：用公钥跑 §3 的 check，确认无未签名警告、`grants` 与 `listing`/源码一致（只有 `storage`，无 host）。
2. **抽验状态机语义**：挑 T1 的 7 条序列里任意 2-3 条用 `drive.py seq` 重放，核对终态与 `replan` 计数（尤其 S3 的重规划封顶 rc 1→2→3 与 `manual_plan` 清零）。
3. **审 `parse_notice` 的真实性**：确认每个事实都带 `source_quote`、缺失字段进 `missing` 而不是被猜测（这是 no-facts 原则的关键，也是初赛清单第 6/7 项）。
4. **裁定下一步**：初赛已提交，是否需要（a）为决赛做强化（如把调试区改为独立 debug 面板以提升商店形态），（b）现在就做 App Hub 商店上架，或（c）等待赛事后续入口公布。

## 8. 权威资料（判断依据，按优先级）

1. 赛事文档：`C:\Users\lsy\Desktop\codex项目文件\hackathon-agenticapp26\docs\app-hub-submission.md`（**作品提交与 OctoSense App Hub**，含"无需等待 Hub 上架"的原文）、`competition-schedule.md`
2. 官方运行时文档：`<workspace>\OctoScript-App-Design-Flow\docs\`（`SCRIPT-API.md`、`PUBLISHING.md`、`AI-SERVICES.zh-CN.md`、`CAPABILITIES.md`）
3. 本仓库内证据：`OUTER_LOOP_REVIEW.md`（任务/ACK/复验全记录）、`BRIEF.md`、`build/REVIEW-ANSWERS.md`、`build/HUMAN-RUNBOOK.md`、`build/review.json`
