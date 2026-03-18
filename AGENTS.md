# AGENTS.md

> ACP guide for AI agents.

## ⚠️ Critical Rule

**agent-context/ 파일 직접 수정 금지.**
**반드시 ACP Skills 사용.**

위반 시 context 동기화 깨짐.

---

## Agent Context Pack (ACP) v1.0

### Directory Structure

| Directory | Access |
|-----------|--------|
| `/agent-context/decisions/` | **READ-ONLY** → `/acp-decision` |
| `/agent-context/constraints/` | **READ-ONLY** → `/acp-constraint` |
| `/.agent/` | **VIA SKILL** → `/handoff`, `/takeover` |

### Session Workflow

| Phase | Action |
|-------|--------|
| **Start** | Read constraints/ → Read decisions/ → `/takeover` |
| **During** | `/acp-decision`, `/acp-constraint` |
| **End** | `/handoff` |

### ACP Skills

| Skill | When |
|-------|------|
| `/acp-decision` | 아키텍처 결정 |
| `/acp-constraint` | 제약 추가 |
| `/handoff` | 세션 종료 |
| `/takeover` | 세션 시작 |

### Agent Notes

- **Codex**: Auto-reads AGENTS.md
- **Claude Code**: Auto-loaded via CLAUDE.md → AGENTS.md symlink

<!-- ACP:TEMPLATE_END -->
