# Triage labels

This repo uses the following label vocabulary for the triage state machine:

| Role | Label |
|------|-------|
| Needs evaluation | `needs-triage` |
| Waiting on reporter | `needs-info` |
| Ready for AFK agent | `ready-for-agent` |
| Ready for human | `ready-for-human` |
| Will not fix | `wontfix` |

## Notes

- These are the default label strings; create them in the GitHub repo if they don't exist
- The `triage` skill applies these labels as issues move through the state machine
