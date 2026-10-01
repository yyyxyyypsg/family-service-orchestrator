# Submit dev.aster.fso 0.1.0

**App id**: `dev.aster.fso` · **version** `0.1.0`
**Name**: 家庭服务事件编排器（Family Service Event Orchestrator）
**Category**: shopping · **Platforms tested**: Windows
**License**: Apache-2.0

## Source

- Repo: https://github.com/yyyxyyypsg/family-service-orchestrator
- Tag: `v0.1.0`
- Commit: `46a4a20c70b1233240da8e36b0a2d02fc03fd8ff`
- Bundle path: `bundle/`
- Entry: `main.splash` (script app)

## Publisher

- Name: `agent aigc`
- Publisher id: `yyyxyyypsg`
- Public key: `33818b0b907191c005de59d83c521c58e23e78635df3e0f7139e3b051040c8cd`

## hub check output

```
dev.aster.fso 0.1.0 — PASSED
  grants: capabilities {"storage"}, hosts {}, storage 16777216 bytes, agent none
```

(signed; `hub check --publisher-key yyyxyyypsg=33818b0b…` reports no unsigned warning)

## What it does

Paste an air-conditioner delivery / installation notice; the app parses facts **with the
original quoted text as evidence** (order id, delivery date, installation time, address),
detects the delivery delay and the time conflict, and proposes confirmable reschedule +
follow-up reminder plans from local schedule data. On confirmation it updates a local
service timeline, registers the after-sales review and writes an audit row. It supports
partial-success retry, duplicate-action interception (idempotency key), replanning with a
cap (Replan → Unstable at 3) and a rules fallback. Pure local rules engine: no network,
no device AI, no external platform is contacted; every fact traces back to the pasted text.

**Not claimed**: the app never claims to modify any external logistics/installation/payment
system. All actions land in the local timeline and local storage.

## Capabilities

`storage` only (timeline, audit, facts and plans persist to `state.json` in the app jail;
verified surviving a restart). No hosts declared, no `net`, no `agent`, no `tools.json`.
`card-host` runs it with `capabilities {"storage"}`.

## Materials

- `build/review.json` — hub scan packet (7 questions)
- `build/REVIEW-ANSWERS.md` — the 7 answers, honest; Q7 route: **human-review** (the app
  keeps a demo control console — sample-fill and 22-event injection — for hackathon
  reproduction; the listing description states this and notes it would be hidden in a
  product build)
- `build/HUMAN-RUNBOOK.md` — signing and submission steps actually used
- `build/TEAM.md` — roster
- Demo video: `build/video/demo-dev.aster.fso.mp4` (2 min 33 s, captured frame-by-frame
  from a live `card-host` session driven over the native remote bridge)
- Screenshots: `bundle/screenshots/01-main.png`, `02-executed.png`, `03-partial.png`
  (real captures, looked at; named in `listing.json`)

## Notes for the maintainer

- First submission for this publisher: the publisher key is not yet in the hub catalog, so
  `hub scan` on the signed bundle reports `publisher key "yyyxyyypsg" is not registered
  with this hub`. The attached packet was produced before signing; the signed bytes are in
  the tag above.
- The bundle contains no secrets, no password field, no login form, no AI call.
