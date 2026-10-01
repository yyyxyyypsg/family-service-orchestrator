# OUTER_LOOP_REVIEW — 家庭服务事件编排器（Splash 移植）

> OctoLoop 黑板。外环追加编号任务（含验收标准）；内环完成后追加 ACK 行，声明验证到什么程度，附证据。
> 规则：内环只提交不推送；结果不符合目标时外环退回、内环继续；人环（Aster）负责定目标、批范围、最终验收。

---

## T1 · 状态机移植（domain.js → Splash）

**目标**：17 状态 / 22 事件转移表逐行移植进 `bundle/main.splash`，语义与 JS 完全一致（含重规划计数规则）。

**验收标准**（等价于 `test/domain.test.js` 七个用例，经远程桥驱动验证）：
1. happy：paste_notice→parsed→conflict_found→plan_ready→user_confirmed→all_actions_succeeded→verify_ok ⇒ `Following`
2. partial：…→partial_actions_succeeded ⇒ `Partial`，且 Partial+plan_ready ⇒ `AwaitingConfirm`
3. 重规划封顶：…→state_changed⇒Replan(rc1)→replan_conflict⇒Replan(rc2)→replan_conflict⇒Unstable(rc3)→manual_plan⇒AwaitingConfirm(rc0)
4. 补充信息：paste_notice→need_info→parsed ⇒ `Parsing`；need_info→parse_failed ⇒ `ParseFailed`
5. 非法转移：Idle+user_confirmed ⇒ 拒绝并记录，状态不变
6. 售后：…→service_done⇒Completed→register_postsale⇒PostSale→postsale_handled⇒Completed→postsale_expired⇒Idle
7. Following+new_event 二轮后 replan_conflict ⇒ Unstable(3)，manual_plan 清零

**退回条件**：任一序列终态/计数与预期不符。

---

（内环 ACK 追加在对应任务下方）
