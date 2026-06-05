# Triage Labels

This repo uses the **five canonical triage roles** as GitHub issue labels.

## Label Vocabulary

| Label | Purpose | When applied |
|-------|---------|--------------|
| `needs-triage` | Maintainer needs to evaluate | New issue, not yet reviewed |
| `needs-info` | Waiting on reporter | Issue lacks reproduction steps or context |
| `ready-for-agent` | Fully specified, AI-agent-ready | All context present, agent can pick up |
| `ready-for-human` | Needs human implementation | Agent cannot complete (needs judgement) |
| `wontfix` | Will not be actioned | Issue is out of scope or duplicate |

## Usage

- Skills `triage` and `to-issues` apply these labels automatically
- Labels must exist in the GitHub repo (create them via `gh label create` if missing)
- To override defaults, edit this file and the skills will read from here
