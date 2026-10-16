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
#   AGENT=orchestrator   agent to run each iteration
#   DRY_RUN=1            print the command instead of running it
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
iterations="${1:-1}"
agent="${AGENT:-orchestrator}"

prompt='Continue the Relayable project in this repository.

Read full.md completely, then AGENTS.md and docs/agent-loop/{NEXT,STATUS,BACKLOG,TASKS,DECISIONS,TESTS}.md plus the WORKLOG.md tail, docs/architecture/{interface-contract,next-phase-plan,code-organization}.md, and docs/adr/ADR-001..005. Trust repository files over prior conversation.

Then continue at the next incomplete acceptance criterion: inspect reality first (git status, git worktree list, git log), delegate bounded work with explicit path ownership, validate with real commands, integrate, and checkpoint docs/agent-loop/** before stopping. If you approach your context limit, finish a safe checkpoint and write the handoff instead of starting new work.'

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
  if [[ "${DRY_RUN:-0}" == "1" ]]; then
    printf 'opencode run --agent %q %q\n' "$agent" "$prompt"
  else
    opencode run --agent "$agent" "$prompt"
  fi
done
