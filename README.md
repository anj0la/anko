
# anko

<p align="center">
  <img src="images/anko-logo.png" width="256" alt="Anko logo">
</p>

<div align="center">

[![Build](https://github.com/anj0la/anko/actions/workflows/deploy.yml/badge.svg)](https://github.com/anj0la/anko/actions/workflows/deploy.yml)
[![License](https://img.shields.io/github/license/anj0la/anko)](LICENSE)
[![AI Usage Disclosed](https://img.shields.io/badge/AI%20Usage-Disclosed-blue)](docs/development.md)

</div>

A GitHub App that keeps GitHub Issues automatically in sync with `TODO` / `BUG` / `DEPR` comments in your source code, without writing anything back into your repository.

On every push, anko-bot scans changed source for structured tags like `// TODO(parser): rewrite error recovery`, opens a GitHub issue for new ones, and closes issues for tags that have since disappeared from the code.

## Demo

<img width="800" height="450" alt="anko_demo" src="https://github.com/user-attachments/assets/efc78fd7-75d5-476f-b4d9-6b06203b84d9" />

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

## Development

This project was intentionally developed as an experiment in AI-assisted
development. See [Development Notes](docs/development.md) for more about
how AI was used during development.
