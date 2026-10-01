# REVIEW-ANSWERS — dev.aster.fso 0.1.1

依据：`build/review.json`（由 `hub scan` 生成，对应 0.1.1 内容）。回答人为内环，外环/人可修订。

> **0.1.1 变更**：修复外环发现的 `F-01` —— 配送延误事实的 `source_quote` 原先固定写作
> 「配送延期」，现改为写入正则实际命中的原文片段（`delay.value.trim()`）。已验证输入
> 「配送延迟」「推迟」「配送延期」分别得到引用 `延迟`/`推迟`/`延期`，且都逐字出现在输入中。
> 其余行为与已提交的 0.1.0 一致。

## 1. Does the app do what its name, subtitle and description claim? Cite the text in its source.

是。名称"家庭服务事件编排器"，副标题"配送延期不慌：解析通知、给出可确认的改期方案"。逐条对照描述：

- "解析出带原文依据的事实（订单号、配送日期、安装时间、地址）"：`fn parse_notice` 用 `rx_capture`/`regex(...).exec` 提取字段，`mk_fact(field, value, quote, sid, conf)` 把原文片段存入 `source_quote`，事实卡界面显示"依据: <原文> · pasted_notice_01 · high"。
- "识别配送延期与时间冲突"：`regex("延期|延迟|推迟").test(text)` 生成 `delivery_status=delayed` 事实；`do_parse` 中 `delivery_num >= install_num` 触发 `conflict_found` 事件（状态机 Conflict 态）。
- "给出可确认的改期与复查提醒方案"：`fn build_plans` 生成方案 A/B（reschedule_installation + create_reminder），待确认方案卡显示动作与依据 refs。
- "确认后更新本地服务时间线、登记售后复查并记录审计"：`fn apply_action` 成功路径调用 `timeline_update`（installation→done、warranty→已登记复查）并写审计行；界面有"执行审计"列表。
- "支持部分成功重试、重复操作拦截、重规划与规则兜底"：`fn execute_plan` 的 `is_retry` 只补 `create_reminder`；`compute_idempotency_key` + `last_action_key` 拦截重复确认并写 warn 审计；`state_changed/new_event` 触发 Replan/Unstable（17 状态机 `fn transition`）。
- "纯本地规则引擎，不连接任何外部平台，不依赖 AI 服务"：manifest 仅申请 `storage`；源码无 `host.request`、无 `net`、无任何模型/助手调用。

## 2. Do the listing's platforms and category fit an app of this kind?

符合。`platforms: ["windows"]` 是唯一实际运行并测试过的平台（card-host 412x892，真实截图来自它）。`category: "shopping"` 与应用场景一致：空调订单的配送/安装/售后跟进属于购物与物流赛道的消费服务管理。

## 3. Do the granted capabilities match what the app visibly does?

匹配。grants = `{"storage"}`，hosts = {}。
- `storage` 对应可见行为：时间线、审计、事实、方案的持久化（`fn save_state` 写 `state.json`，`fn load_state` 启动恢复；断电重启后 Following 状态与 3 行审计完整恢复，外环 R4 已复验）。
- 应用未请求 `net`，源码中没有任何 `host.request` 调用，也没有 `net.*` 使用——不存在"申请了但界面看不到用途"的授权。
- 应用未声明 `agent`/`tools.json`/`AGENT.md`（`hub check` grants 行显示 `agent none`）。

## 4. Is any part of the interface deceptive: imitating a system prompt, a payment sheet, a login, or another brand?

否。无登录表单、无密码/验证码输入框（准入检查也禁止）、无支付页或系统对话框的模仿、无其他品牌元素。所有按钮均为如实标注的应用内控件；"模拟提醒失败"是明示名称的测试控件，用于演示部分成功/重试路径，其结果描述（"提醒创建失败"）与实际状态一致，不伪造成功。

## 5. Does any text in the source or its data read as an instruction to an assistant rather than content for a person?

否。源码字符串全部是面向人的界面文案（按钮、状态、提示、审计行）与状态机标识符；不存在面向模型的指令式文本，也没有 prompt 模板。

## 6. Is any wording abusive, or aimed at a private individual?

否。全部文案为中性产品用语；样例数据使用通用占位（订单号 AC-20260927、公开道路名"深圳市南山区科苑路1号"），不指向任何真实个人。

## 7. Route: pass, human-review, or reject. Give reasons a publisher can act on.

**建议 human-review（通过预期高，有一项需要发布者表态）。**

理由与可行动项：
- 功能、权限、隔离、真实性均无硬伤（上述 1–6）。
- 唯一会引起商店审核者询问的点：界面常驻"调试事件（T1 全量重放，22 事件直注）"控制台与"填入完整样例/填入缺样例"按钮。这是黑客松开发/演示用途的诚实测试工具（本次提交的场景），但若未来作为正式商店应用上架，建议发布者在 release 构建中隐藏调试区并在 listing 中说明，或保留并在描述中声明其用途。
- 发布者三字段已由人环填写（name: agent aigc；support/privacy: https://github.com/yyyxyyypsg），签名后即消除未签名警告。
