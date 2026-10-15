# ADR-005 — Environment-variable namespace

| Field | Value |
|---|---|
| Status | **Proposed — Lead decision; pending Manager acceptance and DECISIONS.md index update (DEC-006 → ADR-005)** |
| Date | 2026-10-15 |
| Deciders | Lead (Relayable / Superlogical) |
| Supersedes | — (new ADR; no prior environment-variable-namespace ADR) |
| Related | DEC-006; `full.md` §0.1, §3, §5.2, §8.1; `docs/agent-loop/DECISIONS.md`; `docs/adr/ADR-002-protocol-schema-and-transport.md` D14; `docs/compatibility/reference-behavior-matrix.md` §7.3, §14 D-08; `docs/architecture/interface-contract.md` |

> **Status note.** This ADR records the Lead's decision that Relayable's environment-variable
> namespace is **`RELAYABLE_*` only**, with no aliases and no dual-setting. It is `Proposed` pending
> Manager acceptance and the DECISIONS.md index update (`DEC-006 → ADR-005`); the decision is binding
> on Relayable implementation only once accepted. Every `REX_*` name below appears **only** as
> documented reference/compatibility evidence for the reference product (see the reference-evidence
> table and the boundary sentence in `## Context`). Nothing in this ADR claims binary or wire-level
> compatibility with the reference implementation. `full.md` §8.1 still requires the decision to be
> **tested**; naming a concrete test location here is a proposal only, and this ADR creates no test
> file (`tests/` does not exist — Phase 0; see `### §8.1 requirement mapping`).

---

## Abstract

`full.md` §8.1 requires Relayable to choose its contextual environment-variable namespace —
`RELAYABLE_*`, reference-named aliases, or both — document the choice in an ADR, test it, and avoid
mixing namespaces accidentally. This ADR chooses **`RELAYABLE_*` only**: `RELAYABLE_SESSION`,
`RELAYABLE_BLOCK`, `RELAYABLE_SERVER`, and `RELAYABLE_CLIENT`. Relayable neither writes nor reads a
reference-named (`REX_*`) variable, now or as a silent fallback. This satisfies the `full.md` §0.1
product-naming rule, preserves the documented user-visible context behavior of the reference product
while using Relayable's own variable names, and avoids implying the binary/wire compatibility that
`full.md` §3 forbids.

---

## Context

### The decision requirement (`full.md` §8.1)

`full.md` §8.1 states the requirement verbatim:

> "Reference documentation currently describes `REX_SESSION`, `REX_BLOCK`, and `REX_SERVER` as context variables. Relayable should choose whether to use `RELAYABLE_*`, support aliases for compatibility, or set both; document the decision in an ADR and test it. Do not mix namespaces accidentally."

That sentence imposes four obligations: (1) make an explicit choice among `RELAYABLE_*`, aliases, or
both; (2) document it in an ADR; (3) test it; and (4) do not mix namespaces accidentally. This ADR
discharges (1) and (2); `### §8.1 requirement mapping` addresses (3) as a proposal; the chosen design
discharges (4) by construction.

§8.1 additionally requires **context-aware defaults when executed inside a managed terminal** and
**explicit host/session/window/block/client targeting**, with **ambiguous targets failing before
mutation** and stable exit codes "documented and tested." Those requirements are the reason Relayable
needs a context mechanism at all; this ADR fixes only the namespace and precedence, not the whole CLI.

### The product-naming rule (`full.md` §0.1)

> "Do not label the product “REX”, “REX Superlogical”, or “Superlogical Terminal” in the new UI, package metadata, application title, README, docs, CLI help, or marketing copy except when describing the reference product during research."

> "The existing reference product in the videos and docs is called REX. Keep that name only in source-evidence notes and compatibility discussions."

`full.md` §0.1 also fixes the preferred public CLI name (`relayable`), the proposed Rust crate prefix
(`relayable-`), and the proposed app title (`Relayable`). Adopting a reference-named variable as
Relayable's primary interface would put the reference product's name on a Relayable user-facing
surface, which §0.1 confines to reference/compatibility evidence.

### The compatibility boundary (`full.md` §3)

> "Do **not** claim binary or wire-level compatibility with the original REX implementation unless the actual protocol is documented and interoperability tests prove it. Replicating documented user-visible behavior and implementing an equivalent native protocol are distinct goals."

`docs/compatibility/reference-behavior-matrix.md` §2 records the same boundary: "No binary or
wire-level compatibility with REX is claimed or supported by this matrix; it records documented
user-visible behavior only." Adopting a reference-named namespace as a first-class surface would
invite the inference of compatibility that §3 forbids. Replicating the documented **user-visible
context behavior** with Relayable's own variable names is the correct reading.

### What the reference product documents (reference evidence only)

`docs/compatibility/reference-behavior-matrix.md` §7.3 records four environment variables as
`verified-official-docs`. These are **reference/compatibility evidence for the reference product, not
Relayable product naming** (AGENTS.md: "Never label the product 'REX' outside
research/compatibility evidence"):

| Reference variable | Label | Documented meaning (verbatim, matrix §7.3) |
|---|---|---|
| `REX_SESSION` | verified-official-docs | "The ID of the session the terminal belongs to" |
| `REX_BLOCK` | verified-official-docs | "The ID of the terminal's own block" |
| `REX_SERVER` | verified-official-docs | "The address of the server that runs the terminal" |
| `REX_CLIENT` | verified-official-docs | "A client. `REX_CLIENT` sets a default." |

Matrix §7.3 also records that the reference "knows where you are from three environment variables it
sets in every terminal" (`REX_SESSION`, `REX_BLOCK`, `REX_SERVER`), while **whether `REX_CLIENT` is set
automatically in terminals is `unknown`** (`not documented in current docs` — "it appears only as a
default source for `-C`"). Matrix §7.3 additionally records "any environment variables other than the
four `REX_*` names" as `unknown`. The `-C, --client` flag row (matrix §7.2) sources `REX_CLIENT` only
as the documented default for `-C`.

**Boundary sentence.** `REX_*` appears in this ADR only as documented reference evidence for the
reference product. Nothing here claims REX binary or wire compatibility (`full.md` §3); the names are
recorded so the Relayable decision can be shown to preserve the documented *user-visible* context
behavior without adopting the reference product's naming.

### What is already deferred to this ADR

- `docs/architecture/interface-contract.md` lists under "What this contract deliberately does not
  decide": *"Environment-variable namespace (`RELAYABLE_*` vs REX aliases vs both) — ADR-005 per
  DOC-001 §14 row D-08."* The interface contract fixes entity identity and ownership but defines **no**
  env-var resolution algorithm; it hands the namespace to this ADR.
- `docs/adr/ADR-002-protocol-schema-and-transport.md` D14 is **advisory only**: *"Advisory only. Not
  decided here; ADR-005/DECISIONS.md owns the record."* Its recommendation (Option A, `RELAYABLE_*`
  only, with a possibly-later read-only `REX_*` fallback behind a compatibility task) is input to this
  ADR, not a decision.
- `docs/compatibility/reference-behavior-matrix.md` §14 row D-08 records the namespace question as
  `unknown`, "**still unverified/open** — DEC-006 is Pending and points at `docs/adr/ADR-005`."
- `docs/agent-loop/DECISIONS.md` records DEC-006 "Env var namespace: `RELAYABLE_*`, REX aliases, or
  both" as *"Recommended by ADR-002 D14 as `RELAYABLE_*`-only; Manager approval pending"*, and lists
  "Env namespace: `RELAYABLE_*` vs REX-compatible aliases vs both (§8.1)" among open questions.

DEC-006 was Pending. This ADR resolves it with the Lead's decision: **`RELAYABLE_*` only.**

### Evidence-handling rule used by this ADR

Following ADR-002's rule: a name whose only support is reference documentation may appear only as
reference/compatibility evidence, with its source section named at the point of use. The four
`REX_*` names above qualify (matrix §7.3). Every other name, ordering, or behavior this ADR adopts is
marked **Relayable-defined (no REX evidence)** inline.

---

## Options Considered

### Option A — `RELAYABLE_*` only (CHOSEN)

- **Description:** Relayable reads and writes only `RELAYABLE_SESSION`, `RELAYABLE_BLOCK`,
  `RELAYABLE_SERVER`, and `RELAYABLE_CLIENT`. No `REX_*` name is ever written or read.
- **Pros:** One unambiguous namespace; satisfies §8.1 "do not mix namespaces accidentally" by
  construction; respects the §0.1 product-naming rule; preserves the documented *user-visible*
  context behavior (a terminal knows its session, block, and server) with Relayable's own names;
  makes no binary/wire-compatibility implication under §3; changes are localized to the context
  resolution layer.
- **Cons:** Users who learned the reference product's variable names must learn a new prefix for the
  same concept; reference CLI snippets that interpolate `$REX_SESSION` do not work unmodified (see
  `## Consequences`).
- **Chosen because:** it is the only option that satisfies §0.1 and §3 while preserving the documented
  context behavior required by §8.1. It is the Lead's decision recorded here.

### Option B — Set both `RELAYABLE_*` and `REX_*` on every managed terminal

- **Description:** Every managed terminal exports both namespaces; Relayable reads `RELAYABLE_*`
  first and falls back to `REX_*`.
- **Pros:** Reference-named scripts that read `$REX_SESSION` keep working without edits; eases
  migration for reference users.
- **Cons:** Two namespaces must be kept coherent; the reference product's name is written onto a
  Relayable user-facing surface; it invites the inference that Relayable is reference-compatible; it
  is precisely the "mix namespaces accidentally" mode §8.1 warns against; it implies a compatibility
  posture §3 forbids absent a documented protocol and passing interoperability tests.
- **Not chosen because:** it conflicts with §0.1 (reference naming on a Relayable surface) and §8.1
  (namespace mixing), and it implies the §3 compatibility claim Relayable cannot make.

### Option C — `REX_*` only

- **Description:** Adopt the reference product's variable names as Relayable's namespace.
- **Pros:** Drop-in for reference-named scripts; least new vocabulary for reference users.
- **Cons:** Directly contradicts §0.1 (reference product name on Relayable CLI/environment surfaces);
  implies compatibility §3 forbids; discards the Relayable-specific naming the §0.1 CLI/crate prefix
  establishes; DOC-001 §6 records the reference target-resolution rules as `unknown`, so relying on
  the reference names would rest on undocumented behavior.
- **Not chosen because:** it violates the product-naming rule and the compatibility boundary.

### Option D — No context environment variables at all (require explicit flags always)

- **Description:** Relayable sets nothing; every command requires explicit host/session/window/block/
  client flags.
- **Pros:** No namespace decision and no ambiguity; nothing to mix; no reference-naming exposure.
- **Cons:** Loses the documented context-aware-defaulting requirement (`full.md` §8.1: "Context-aware
  defaults when executed inside a managed terminal") and sacrifices documented reference user-visible
  behavior (`REX_SESSION`/`REX_BLOCK`/`REX_SERVER` context) without need; degrades the terminal
  workflow that `full.md` §0.2/§2.4 treat as a primary fidelity criterion; makes ordinary commands
  verbose.
- **Not chosen because:** §8.1 requires context-aware defaults and §0.2/§2.4 require fidelity to the
  observed reference workflows; dropping context would be a regression, not a simplification.

---

## Decision

**Status: Accepted — Manager acceptance 2026-10-15 (DEC-006).**

1. **Namespace.** Relayable's environment-variable namespace is **`RELAYABLE_*` only**. The exact names
   are `RELAYABLE_SESSION`, `RELAYABLE_BLOCK`, `RELAYABLE_SERVER`, and `RELAYABLE_CLIENT`. No `REX_*`
   alias is written or read by Relayable. *(Relayable-defined (no REX evidence).)*

2. **Context variables set in every managed terminal.** Relayable sets `RELAYABLE_SESSION`,
   `RELAYABLE_BLOCK`, and `RELAYABLE_SERVER` in every managed terminal it launches, carrying the
   session ID the terminal belongs to, the ID of the terminal's own block, and the address of the
   server that runs the terminal, respectively. This **mirrors the documented reference user-visible
   context** (matrix §7.3) but the environment-variable **names are Relayable's own** — this is
   **Relayable-defined (no REX evidence)**. *(Relayable-defined (no REX evidence).)*

3. **Client variable.** `RELAYABLE_CLIENT` is **Relayable-defined (no REX evidence)** and is the
   default source for the `--client/-C` target when no explicit flag is given. Relayable does **not**
   set it automatically in terminals. The reference product likewise does not document automatic
   setting (matrix §7.3 records whether `REX_CLIENT` is auto-set as `unknown`), so Relayable claims
   nothing about the reference here. *(Relayable-defined (no REX evidence).)*

4. **Precedence.** Explicit CLI target flags always win over environment context. Precedence, highest
   first: **explicit flag/target argument → `RELAYABLE_*` context variable → no default.** The
   precedence ordering itself is **Relayable-defined (no REX evidence)**. *(Relayable-defined (no REX
   evidence).)*

5. **No-context behavior.** When the relevant `RELAYABLE_*` variable is unset and no explicit target
   is given, a command that requires a target **fails** with a documented error and a documented
   stable non-zero exit code. Relayable never guesses a target and never silently falls back to
   another session/block/server. This is **Relayable-defined (no REX evidence)** and is consistent
   with §8.1 ("Ambiguous targets fail before mutation; never guess silently") and with the reference
   evidence that at least one reference command does not default to the current session (matrix §8.3:
   "Unlike other `rex` commands, `rex api call` doesn't default to the session you are in."). The
   specific error text and exit-code number are a CLI-design detail for the command-matrix task, not
   fixed here. *(Relayable-defined (no REX evidence).)*

6. **Environment only, not a protocol surface.** `RELAYABLE_*` variables are read-only inputs to
   Relayable from the terminal environment. They are **not** a protocol or wire surface, are not
   serialized into protocol messages as a namespace, and create no compatibility surface.
   *(Relayable-defined (no REX evidence).)*

7. **Explicitly deferred — no `REX_*` fallback now.** No `REX_*` fallback or alias is added now; if one
   is ever added it must be a separate compatibility task with tests, documented as reference-behavior
   compatibility only and never as REX wire compatibility. *(Relayable-defined (no REX evidence).)*

### §8.1 requirement mapping

| `full.md` §8.1 requirement | This ADR's disposition |
|---|---|
| Choose `RELAYABLE_*`, aliases, or both | Chosen: `RELAYABLE_*` only (Decision 1) |
| Document the decision in an ADR | This ADR (DEC-006) |
| Test the decision | Proposed test (below); not yet created — see caveats |
| Do not mix namespaces accidentally | Decision 1 reads/writes only `RELAYABLE_*`; no alias path exists |
| Context-aware defaults inside a managed terminal | Decisions 2–4 |
| Ambiguous targets fail before mutation; never guess silently | Decision 5 |

### Test plan (proposal only — this ADR creates no test)

`full.md` §8.1 requires the decision be **documented and tested**. This ADR documents it; the test is
**proposed, not added**. The repository is Phase 0 (`AGENTS.md`: "Repository state: **Phase 0** —
spec-only repo at start"), and **no `tests/` directory exists** in this worktree.

- **Proposed location:** a context-resolution/integration test under the spec's `tests/` area. The
  `full.md` repository-layout section lists `tests/{protocol,lifecycle,reconnect,multi-client,terminal-compat,fixtures}`,
  and `docs/architecture/interface-contract.md` repeats: "`tests/{protocol,lifecycle,reconnect,multi-client,terminal-compat,fixtures}` (contract tests per area)". The closest existing planned area is
  `full.md` §13.1's "command/context resolution" test target. One concrete proposed path is
  **`tests/context-resolution/`**; if the test owner prefers the existing planned taxonomy, the
  fallback is the nearest existing planned area (e.g. under `tests/lifecycle/` or
  `tests/multi-client/` as appropriate). Final placement is that owner's call.
- **Proposed coverage:**
  1. `RELAYABLE_*` presence resolution — with `RELAYABLE_SESSION`/`RELAYABLE_BLOCK`/`RELAYABLE_SERVER`
     set, a context-defaulting command resolves the intended target.
  2. Flag-over-env precedence — an explicit flag overrides the corresponding `RELAYABLE_*` value
     (Decision 4).
  3. No-context error — with the relevant variable unset and no flag, the command fails with the
     documented error and stable non-zero exit code, and does not guess (Decision 5).
- **Caveats (explicit):** (a) `tests/` does not exist yet (Phase 0); (b) no env/context-specific test
  area is currently defined; (c) `tests/**` is owned by CORE-001 per `docs/agent-loop/TASKS.md`
  ("owns root workspace + `crates/**` + `tests/**` + `protocol/**`"), so final placement and ownership
  are that task's call; (d) this ADR creates no test file. The test framework/tooling is
  **undecided by this ADR**; no framework is named here as verified.

---

## Consequences

### Positive

- **One unambiguous namespace.** Exactly one set of names (`RELAYABLE_*`) exists in Relayable; the
  §8.1 "do not mix namespaces accidentally" hazard cannot arise.
- **Naming rule respected.** No reference-product name appears on a Relayable user-facing surface
  (`full.md` §0.1); the CLI name, crate prefix, and app title remain Relayable's own.
- **No unsupported compatibility claim.** Because Relayable reads and writes no `REX_*` name, nothing
  in the implementation implies the binary/wire compatibility `full.md` §3 forbids.
- **Documented reference behavior preserved at the user-visible level.** A managed terminal still
  knows the session, block, and server it belongs to, and `-C` still has a context default — the same
  observable ergonomics the reference documents, under Relayable's own names.
- **Safe failure mode.** Missing context produces an explicit error rather than silently wrong
  targeting (Decision 5).

### Negative / costs

- **New vocabulary for reference users.** People who know the reference product's variable names must
  learn a new prefix for the same concepts.
- **Reference snippets do not work unmodified.** Copy-pasted reference CLI snippets that interpolate
  `$REX_SESSION` (or otherwise use a reference-named variable) will not resolve in Relayable
  terminals.
- **No drop-in compatibility with reference scripts.** This is a deliberate, acknowledged cost. It is
  **not** a wire- or binary-compatibility claim and **not** a requirement of this project; preserving
  the documented user-visible behavior (`full.md` §3) is the goal, not script drop-in.

### Mitigations

- Missing context yields an explicit error instead of guessing (Decision 5), so a user who relies on a
  reference-named variable gets a clear failure rather than wrong targeting.
- Any future `REX_*` fallback is deferred and explicitly recorded (Decision 7): it would require a
  separate compatibility task with tests and a documented reference-behavior-compatibility framing.

---

## What Evidence Would Cause Reconsideration

Reconsider (and, if warranted, supersede via a new ADR) if:

1. Actual reference scripts or users are blocked in a documented way, and a compatibility task proves
   a **read-only `REX_*` fallback** is needed — provided the change still makes no binary/wire
   compatibility claim (`full.md` §3).
2. DOC-001 or a vendor source documents **additional or changed reference variable names/behavior**
   (for example, that the reference auto-sets `REX_CLIENT` in terminals, or adds further variables)
   in a way that changes the mapping recorded here (matrix §7.3 currently records both as `unknown`).
3. Evidence shows Relayable **must interoperate with unmodified reference tooling** (a real,
   documented requirement, not a preference).
4. The §8.1 test shows namespaces **mixing in practice** — i.e. an implementation path reads or
   writes both `RELAYABLE_*` and `REX_*` accidentally.
5. `RELAYABLE_*` (specifically one of the four chosen names) is found to **collide with a documented
   name in the target OS/ecosystem** or with an established convention that would cause misbehavior.

What would **not** cause reconsideration: generic dislike of the prefix; a preference for shorter
names; or any argument that rests on claiming reference compatibility that `full.md` §3 forbids.

---

## Evidence

Repository sources read for this ADR, with the sections used:

| Source | Sections used |
|---|---|
| `full.md` | §0.1 product-name rules; §3 compatibility boundary; §5.2 ADR structure; §8.1 CLI design principles; §8.2 Server API; repository-layout section (`tests/…`); §13.1 test coverage |
| `AGENTS.md` | Product-name rule; Phase 0 repository state; path ownership |
| `docs/adr/ADR-002-protocol-schema-and-transport.md` | D14 (advisory recommendation); "Evidence base" evidence-handling rule; "Recommendation for DEC-006" |
| `docs/compatibility/reference-behavior-matrix.md` | §7.2 flags/aliases; §7.3 Environment variables (rows quoted); §8.3 Calling methods, session defaulting, errors; §14 D-07, D-08; §15 Unknowns summary |
| `docs/architecture/interface-contract.md` | "What this contract deliberately does not decide" (env namespace deferred to ADR-005); test-directories listing; entity/ID ownership |
| `docs/agent-loop/DECISIONS.md` | DEC-006 (Pending; resolves to this ADR); open questions list |
| `docs/agent-loop/TASKS.md` | CORE-001 ownership (`crates/**` + `tests/**` + `protocol/**`) |

### Decisive quotes (verbatim)

`full.md` §8.1:

> "Reference documentation currently describes `REX_SESSION`, `REX_BLOCK`, and `REX_SERVER` as context variables. Relayable should choose whether to use `RELAYABLE_*`, support aliases for compatibility, or set both; document the decision in an ADR and test it. Do not mix namespaces accidentally."

`full.md` §0.1:

> "Do not label the product “REX”, “REX Superlogical”, or “Superlogical Terminal” in the new UI, package metadata, application title, README, docs, CLI help, or marketing copy except when describing the reference product during research."

`full.md` §3:

> "Do **not** claim binary or wire-level compatibility with the original REX implementation unless the actual protocol is documented and interoperability tests prove it. Replicating documented user-visible behavior and implementing an equivalent native protocol are distinct goals."

---

## Unverified / open questions

- **Reference variable behavior beyond the four names.** Matrix §7.3 records whether the reference
  auto-sets its client variable and whether it defines any variables beyond the four names as
  `unknown`. No claim here depends on those unknowns.
- **Concrete error text and exit-code number for the no-context failure (Decision 5).** Deferred to
  the CLI command-matrix design; §8.1 requires exit-code semantics to be documented and tested.
- **Test placement, framework, and ownership.** Noted above; `tests/` does not yet exist and is owned
  by CORE-001. This ADR proposes a location and coverage but does not create tests.
- **Whether a read-only reference-named fallback will ever be required.** Decision 7 defers it; the
  reconsideration triggers above define when it would be revisited.
