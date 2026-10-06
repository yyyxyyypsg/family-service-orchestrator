# 家庭服务事件编排器 · Family Service Event Orchestrator

GOSIM Agentic App 黑客松 2026 参赛作品 · 队伍 `agent aigc` · 赛道「购物与物流」 · 应用 id `dev.aster.fso`

**意图 → 事实 → 方案 → 确认 → 执行 → 核验**：粘贴一条空调配送/安装通知，
应用解析出**带原文引用**的事实，识别配送延期与时间冲突，给出可确认的改期与复查提醒方案；
确认后更新本地服务时间线、登记售后复查并记录审计。支持部分成功重试、重复操作幂等拦截、
重规划封顶与规则兜底。

**初赛定位**：纯本地规则引擎。不联网、不调用设备 AI、不连接任何外部平台，所有事实可回溯到粘贴原文。
仓库同时提供一个并行的 **Rust/Makepad 原生宿主扩展**，用于接入 OctoSense 的宿主模块和
Octos 代理通道；它不改动已经签名的 `bundle/` 提交物。

## 原生宿主扩展

`native/` 是可独立编译的 Rust crate，复用同一套解析器、状态机和事实引用规则：

- 独立窗口：`family-orchestrator-native`，便于开发和录制演示；
- 宿主模块：`FAMILY_ORCHESTRATOR_MODULE`，实现 `AppModule`，由 OctoSense shell
  在隔离实例中承载；
- AI 服务：`current_state`、`parse_notice`、`confirm_plan`，经 Makepad AI services
  bus 暴露给宿主；
- Octos 边界：应用不自启 kernel、不持有 host token、不直连 socket。宿主注册后，Octos
  peer、审批和审计由 OctoSense shell 负责。

原生扩展还支持多类型服务案件（空调、家具、家政、维修）、准备清单、后续通知合并、用户验收、
售后复查和问题记录。对应 Octos 工具为 `merge_notice`、`create_checklist`、`set_recheck`、
`mark_accepted`、`record_service_issue`；这些动作仍只写入本地案件和审计，不声称修改外部平台。

构建与测试命令见 [`native/README.md`](native/README.md)。提交时仍以 `bundle/` 的签名版本
为初赛基线；原生版本是后续宿主集成路线。

---

## 运行（复现步骤）

前置：Rust stable、Python 3.9+、图形会话，以及官方工作区布局（本仓库的兄弟目录需有
`OctoSense-App-Hub`、`makepad`、`octoscript`、`octoscript-makepad`）。

```sh
# 1. 准备原生运行时与工具（首次，约 3 GB / 需联网）
cd <workspace>/OctoScript-App-Design-Flow
python tools/setup-native.py

# 2. 构建 card-host 与 hub
cd <workspace>/OctoSense-App-Hub
cargo build --release -p octosense-card-host -p octosense-app-hub

# 3. 指向产物（Windows 上产物是 .exe，doctor 需要这两个变量）
export OCTO_HUB=<workspace>/OctoSense-App-Hub/target/release/hub.exe
export OCTO_CARD_HOST=<workspace>/OctoSense-App-Hub/target/release/card-host.exe

# 4. 运行（无头：窗口不出现，远程桥照常工作）
cd <workspace>/OctoScript-App-Design-Flow
python tools/octo run <this-repo>/bundle --port 8142 --hidden --detach

# 5. 操作与截图
curl -s "127.0.0.1:8142/snap"                     # 控件树
curl -s "127.0.0.1:8142/click?x=52&y=202&wait=1"  # 点击（窗口坐标）
python tools/octo shot 8142 out.png               # 真截图
curl -s 127.0.0.1:8142/quit                       # 结束（必须）
```

已验证平台：**Windows**（412x892 card-host 窗口，真实截图来自该环境）。官方文档标注
macOS 为已验证平台，Windows 为未验证；本作品在 Windows 上完整跑通（构建、准入、无头运行、
远程桥交互、持久化）。

### 演示流程（也是演示视频内容）

1. 点「填入完整样例」→「解析通知」：事实卡列出 `字段 = 值` 与**原文依据**；状态 `AwaitingConfirm`
2. 点方案卡选择 A/B（本地日程候选）
3. 点「确认执行」：状态 `Following`，本地时间线变 done，审计新增两行
4. 再次点「确认执行」：**幂等拦截**，只写一条警告审计，状态不变
5. 重新解析后点「模拟提醒失败」：状态 `Partial`（改期已写入、提醒未创建）
6. 再点「确认执行」：**只补未完成动作**，回到 `Following`
7. 退出重启：状态、时间线与审计从 `state.json` 完整恢复

界面底部的「调试事件」控制台（22 事件直注 + 样例填充）是**演示与评审复现工具**，
用于驱动全部 17 状态；正式产品版本会隐藏该区域（见 `bundle/listing.json` 描述）。

## 目录

| 路径 | 内容 |
| --- | --- |
| `bundle/` | **提交物**：`main.splash`、`manifest.json`（已签名）、`listing.json`、`assets/icon.svg`、`screenshots/` |
| `BRIEF.md` | 需求简述：界面、动作、数据、状态、能力与理由 |
| `OUTER_LOOP_REVIEW.md` | OctoLoop 黑板：外环任务、内环 ACK 与验证级别、外环复验记录 |
| `build/` | 审核包（`review.json`）、七问答案（`REVIEW-ANSWERS.md`）、运行手册、演示视频、团队成员名单 |
| `drive.py` | 远程桥驱动脚本：`state` / `click <文字>` / `seq <标签,…>` / `shot <out.png>` |
| `AGENTS.md`、`CLAUDE.md`、`GEMINI.md` | 给编码 Agent 的规则（来自官方模板） |

## 能力与隔离

- 仅申请 `storage`（时间线/审计/事实/方案持久化到应用 jail 的 `state.json`，重启恢复已验证）
- 不声明任何主机、不使用 `net`、无 `agent`/`tools.json`、无密钥、无密码框、无登录表单
- 准入检查：`dev.aster.fso 0.1.0 — PASSED`（已签名）

## 开发方式（OctoLoop 双环）

外环负责目标、任务拆分与独立复验（`OUTER_LOOP_REVIEW.md` 中的 T1/T4/T5 均由其复验通过）；
内环只做当前编号任务，完成后追加 `ACK(done|wontdo|blocked)` 并声明验证级别
（`verified` / `partially-verified` / `unverified`），只提交不推送。

## 许可证

Apache-2.0（见 `LICENSE`）。
