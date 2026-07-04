---
id: EP-XXX
status: not-started
depends_on: []
verify: scripts/verify.sh
---

# EP-XXX — <Title>

## 1. Purpose / Big Picture
<Why this plan exists and what capability it delivers, in 2–4 sentences. How it
fits the ADAD architecture and which roadmap phase it serves.>

## 2. Scope
<Exactly what this plan will implement. Bullet list of concrete deliverables.>

## 3. Non-goals
<What this plan will NOT do. Anything here is forbidden in this plan's sessions.>

## 4. Context and Orientation
<Where in the repo this work lives, which crates/modules are involved, and any
constraint that shapes the approach (static musl, no network egress in tests,
etc.).>

## 5. Files to Read First
<Exact paths the agent must read before editing anything.>

## 6. Files to Change
<Exact paths expected to be created or modified. Final review compares
`git diff --name-only` to this list.>

## 7. Interfaces and Contracts
<Function signatures, CLI flags, config keys, file formats, or JSON shapes this
plan defines or depends on. Name them exactly.>

## 8. Milestones
<Ordered M1, M2, ... Each with Goal / Files to read / Files to change / Exact
edits / Validation command / Expected result / Recovery.>

## 9. Concrete Steps
<Step-by-step implementation detail supporting the milestones.>

## 10. Validation and Acceptance
<Objective acceptance criteria: command + expected result for each.>

## 11. Idempotence and Recovery
<How to safely re-run this plan; how to recover from a half-applied milestone.>

## 12. Progress
- [ ] M1
- [ ] M2
- [ ] verify + status set to complete

## 13. Surprises & Discoveries
<Facts learned, failed hypotheses (do not repeat them), environment quirks.>

## 14. Decision Log
<Dated entries: decision, alternatives, why. Dependencies added. Assumptions.>

## 15. Outcomes & Retrospective
<What shipped, what changed vs. plan, remaining risks. Filled at completion.>
