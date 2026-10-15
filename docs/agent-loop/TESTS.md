# TESTS — commands actually executed and results

Append-only evidence log. Each entry records the date, purpose, exact command, observed result, and
exit status. Never write a result here that was not actually produced by a command that ran.

## 2026-10-15 — Wave 1 kickoff, context-policy verification, delegation recovery

Environment: Linux `falcon` x86_64 (omarchy), `pi 1.2.0`, pi-herdsman 0.27.0, mise-managed
Node 26.8.2, repo `/home/sulthonuladib/work/relayable`.

| # | Command | Observed result | Exit |
|---|---|---|---|
| 1 | `pi --version` | `1.2.0` | 0 |
| 2 | `cat ~/.pi/agent/settings.json` | global settings include `"compaction": { "enabled": false }` (plus theme/model/package keys) | 0 |
| 3 | `ls -la .pi` (before policy) | project `.pi/` did not exist | 2 |
| 4 | `git status --short --branch` | `## master...origin/master`, clean | 0 |
| 5 | `git worktree list` | `master` + 3 linked research worktrees, all at `1c84bca` | 0 |
| 6 | `git branch -a` | `master`, `research/{ui-reference,protocol-compat,env-audit}`, `origin/master` | 0 |
| 7 | `herdr pane list` | `wW:p1` manager `working`; `wX:p1`, `wY:p1`, `wZ:p1` `agent_status: unknown` with shells in worktree cwds | 0 |
| 8 | `herdr pane process-info --pane wX:p1` | `shell_pid` = foreground `fish` process in the worktree cwd | 0 |
| 9 | `herdr pane read wX:p1 --lines 20 --source recent` | fish prompt at worktree cwd; Herdsman readiness marker echoed | 0 |
| 10 | `cat ~/.pi/agent/pi-herdsman/config.json` | `{"spawnPlacement":"subtree"}` — no `contextRetirement` override | 0 |
| 11 | inspect pi-herdsman 0.27.0 `dist/index.js` | `contextRetirement` default is `true`; `session_before_compact` handler cancels threshold compaction for assigned managed agents | 0 |
| 12 | inspect pi extensions | only `herdr-agent-state.ts` globally; no compaction hooks. `pi-herdsman` registers the context-retirement `session_before_compact` hook | 0 |
| 13 | `git -C <worktree> merge --ff-only master` ×3 (ui-reference, protocol-compat, env-audit) | `Updating 1c84bca..ccfdb7a; Fast-forward` for each; each worktree now contains `.pi/settings.json` | 0 |
| 14 | `node /tmp/verify-settings.mjs <primary + 3 worktree cwds>` (`SettingsManager.create(cwd).getSettings()`) | every path: `trusted: true`, `effectiveCompaction.enabled=false`, `projectCompaction.enabled=false`, `errors: []` | 0 |

### Notes

- The failed `delegate_project` calls persisted assignment records under
  `~/.pi/agent/pi-herdsman/runtime/supervision-v2/...` (`research/ui-reference`,
  `research/protocol-compat`, `research/env-audit`, repoKey `/home/sulthonuladib/work/relayable/.git`).
  This is why recovery must use `resume_project`.
- Stale assignment records also exist for the pre-move repoKey `/home/sulthonuladib/personal/relayable`
  (branches `research/reference-ui`, `research/protocol-matrix`, `audit/build-env`). That checkout no
  longer exists; no live Lead owns them.
- No build or test suite exists yet, so no pass/fail suite results are claimed.
