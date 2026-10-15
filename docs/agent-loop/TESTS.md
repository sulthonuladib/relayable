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

## 2026-10-15 — Herdr agent-provisioning blocker diagnosis (pane busy)

Symptom: every programmatic agent launch fails at `stage: agent_start` with Herdr
`agent_pane_busy` / `agent target pane <id> is not an available shell`. This affects Manager→Lead
resume, Lead→Agent `delegate_agent`, and any Herdsman-spawned writer agent.

| # | Command / action | Observed result | Exit |
|---|---|---|---|
| 15 | `herdr pane process-info --pane wX:p1` | foreground processes `[fish, bash(defunct)]` while Herdr reports busy | 0 |
| 16 | `herdr agent start probe --kind pi --pane wX:p1 --timeout 4000 -- --version` | `agent_pane_busy` | 1 |
| 17 | `herdr pane send-keys wX:p1 Enter`, then the same probe | `timeout waiting for agent startup` (= pane accepted; `pi --version` launched) | 1 |
| 18 | `herdr pane run wY:p1 "echo __TIMING__"`, probes every 5 s for 50 s | `agent_pane_busy` for ≥50 s | 1 |
| 19 | A/B test in throwaway tab `wZ:tD`: `pane run` marker with plugin hook active | `agent_pane_busy`; process-info `[fish, bash, bash]` | 1 |
| 20 | A/B test: `functions -e _har_preexec _har_precmd`, then `pane run` marker, then `agent start --timeout 4000` | `timeout waiting for agent startup` (= accepted); process-info `[fish]` only | 1 |
| 21 | `herdr tab create ... --env HERDR_PANE_ID=`, marker + probe | Hooks still ran (`[fish, bash, bash]`), busy — Herdr overrides the env var; env simulation inconclusive | 1 |
| 22 | `herdr tab close wZ:tD` / `wZ:tE`; `herdr pane list --workspace wZ` | test tabs removed; workspace shows only `wZ:p1` | 0 |
| 23 | `resume_project` ui-reference / protocol-compat / env-audit with a background pane wake loop (empty `Enter` every 0.25–0.4 s during launch) | all three Leads started; ui-reference and protocol-compat bound as Herdr agents (`working`), env-audit needed a second resume | 0 |

**Root cause:** `~/.config/fish/config.fish` (lines 40–43) sources the `herdr-automatic-rename`
fish hook (`shell/hook.fish`). Its `fish_preexec`/`fish_postexec` handlers spawn background `bash`
(`automatic-rename.sh`) on every command/prompt. Herdr's `agent start` requires the pane to be at
its prompt with only the shell in the foreground; the hook's live `bash` children make it report
`agent_pane_busy`. Herdsman's readiness marker (`pane run`) always triggers the hook immediately
before `agent start`.

**Remediation options:** A — stop sourcing the fish hook in `~/.config/fish/config.fish`
(recommended, needs user approval; cosmetic loss of live per-command tab renaming); B — add a
project-local `managed-lead` overlay granting Leads bash/edit/write so they execute directly
(repo-only, reversible, changes the delegation model); C — update/reinstall the plugin (unproven)
or report upstream. See `WORKLOG.md` Checkpoint 3.
