# storm-almanac-app — Keeper Directives

CTO command channel. Keeper writes directives, project sessions execute on spawn.

## Keeper Directives

- [ ] Verify project builds locally (`npm install && npm run build`) — confirm baseline functional
- [x] Create DESIGN.md — 7th pillar doc. (2026-04-03, Keeper compliance sweep)

## Tech Debt Backlog (Keeper Audit — 2026-04-10)

### QA-UI & Design Compliance (Low)

- [ ] Fix missing DM Sans font import — `--font-body: 'DM Sans'` defined but never imported, silently falls back to system-ui.
- [ ] Note: Tauri desktop app with separate design lineage. No forge compliance expected.
