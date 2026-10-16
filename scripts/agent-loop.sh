#!/usr/bin/env bash
# Drive the Relayable agent loop as a sequence of *fresh* orchestrator sessions.
#
# Each iteration starts a new OpenCode session (deliberately no --continue), so context is
# never carried between iterations. Durable state is the repository:
# docs/agent-loop/{NEXT,STATUS,WORKLOG,DECISIONS}.md plus the Git graph. See AGENTS.md
# "Context-window policy" and docs/agent-loop/DECISIONS.md DEC-008 / DEC-015.
#
# Usage:
#   scripts/agent-loop.sh [iterations]      # default 1
#
# Environment:
#   LOOP_AGENT=orchestrator   agent to run each iteration
#   LOOP_DRY_RUN=1            print the command instead of running it
#
# The knobs are LOOP_-prefixed deliberately: the ambient environment already exports a
# generic `AGENT` variable, and a bare name would be silently picked up (observed).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
iterations="${1:-1}"
agent="${LOOP_AGENT:-orchestrator}"

prompt='Continue the Relayable project in this repository.

Read full.md completely, then AGENTS.md and docs/agent-loop/{NEXT,STATUS,BACKLOG,TASKS,DECISIONS,TESTS}.md plus the WORKLOG.md tail, docs/architecture/{interface-contract,next-phase-plan,code-organization}.md, and docs/adr/ADR-001..005. Trust repository files over prior conversation.

Then continue at the next incomplete acceptance criterion: inspect reality first (git status, git worktree list, git log), delegate bounded work with explicit path ownership, validate with real commands, integrate, and checkpoint docs/agent-loop/** before stopping.

Scope: Android apps are OUT OF SCOPE (DEC-010) — no SDK, emulator, JDK, or Gradle work; only keep relayable-model/relayable-protocol Android-ready. Do not adopt gpui-kit or libghostty until the user answers DEC-014; UI work proceeds against the existing gpui = "=0.2.2" pin.

If you approach your context limit, finish a safe checkpoint (commit + push + update docs/agent-loop/**) and write a short handoff instead of starting new work.'

cd "$repo_root"

# The loop assumes each iteration starts from a checkpointed tree. Refusing a dirty start is
# cheaper than discovering a half-applied change in a later, context-poor session.
if [[ -n "$(git status --porcelain)" ]]; then
  echo "agent-loop: working tree is dirty; commit or stash before running the loop" >&2
  git status --short >&2
  exit 1
fi

for ((i = 1; i <= iterations; i++)); do
  echo "agent-loop: iteration $i/$iterations (agent=$agent, fresh session)"
  if [[ "${LOOP_DRY_RUN:-0}" == "1" ]]; then
    printf 'opencode run --agent %q %q\n' "$agent" "$prompt"
  else
    opencode run --agent "$agent" "$prompt"
  fi
done
