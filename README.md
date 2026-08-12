# anko-bot

A GitHub App that keeps GitHub Issues automatically in sync with `TODO` / `BUG` / `DEPR` comments in your source code, without writing anything back into your repository.

On every push, anko-bot scans changed source for structured tags like `// TODO(parser): rewrite error recovery`, opens a GitHub issue for new ones, and closes issues for tags that have since disappeared from the code.

## How it works

- Each tag's identity is derived from its kind, labels, message and file, so moving a TODO within a file doesn't trigger a fake close/reopen.
- Anko respects `.gitignore` plus a small hardcoded denylist (`.git`, `node_modules`, `target`, `vendor`, `dist`, `build`).
- Manually closing an anko-bot issue will never silently reopen the issue again, even if the underlying comment is still in the code.

## Architecture

| Crate | Responsibility |
|---|---|
| `scanner` | Pure scan/hash/diff logic, unit tested. |
| `github` | GitHub App auth and Issues API client, with octocrab. |
| `store` | Firestore lookup. |
| `server` | axum webhook receiver, HMAC verification, runs scan → diff → sync. |

**Infra:** Docker → Artifact Registry → Cloud Run, with secrets in Secret Manager (private key mounted as a file, webhook secret injected as an env var) and a GitHub Actions pipeline using Workload Identity Federation.

## Known Limitations

- No fuzzy matching for edited/moved tags. A reworded TODO closes the old issue and opens a new one.
- No custom config (include/exclude paths, custom labels), `.gitignore` only.
- An issue closed by anko-bot and then manually *reopened* on GitHub isn't picked back up by sync.
