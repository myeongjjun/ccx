# ADR-004: Session Collection Caching Strategy

- **Date**: 2026-03-18
- **Status**: superseded
- **Deciders**: user, Claude Code
- **Supersedes**: none
- **Superseded by**: [ADR-006](2026-03-18-adopt-cache-first-cli-defaults.md)

## Context

현재 `collect`는 one-shot 방식으로 동작한다:

```
pick "cc auth" → osascript 실행 (0.5~2초) → 검색 → 포커스
```

PRD의 핵심 요구사항은 **"검색 → Enter → 즉시 이동"** 이다. osascript 호출 비용(0.5~2초)이 매번 발생하면 이 목표를 달성할 수 없다.

`--cache` 옵션이 존재하지만, 수동 실행에 의존하므로 캐시가 금방 stale해진다 (새로 연 세션이 안 보임).

### 현재 흐름의 문제

| 모드 | 장점 | 단점 |
|------|------|------|
| 매번 수집 (기본) | 항상 최신 | osascript 대기 시간 |
| `--cache` 수동 | 즉시 응답 | stale 데이터, 수동 갱신 필요 |

## Decision

**백그라운드 주기적 캐시 갱신 방식을 채택한다.**

이 제안은 이후 구현 우선순위 재평가로 superseded 되었다. 현재 배포 전 기본 동작은 ADR-006의 cache-first + on-demand refresh 전략을 따른다. launchd 기반 주기적 collector는 향후 다시 채택할 수 있는 확장 옵션으로 남긴다.

런처 프로세스와 수집 프로세스를 분리하여:

1. **Collector (백그라운드)**: 주기적으로(3~5초) osascript 실행 → 캐시 파일 갱신
2. **Launcher (포그라운드)**: 캐시에서 즉시 읽기 → 검색 → 포커스

```
┌────────────────────────────────┐
│  Collector (daemon / timer)    │
│  매 N초마다:                    │
│  osascript → normalize → 캐시  │
└──────────┬─────────────────────┘
           │ 파일 기반 캐시 (JSON)
           │ ~/.cache/ccx/sessions.json
           ▼
┌────────────────────────────────┐
│  Launcher (hotkey 트리거)       │
│  캐시 읽기 → 검색 → 포커스       │
│  (osascript 안 돌림 → <50ms)   │
└────────────────────────────────┘
```

### 구현 방식 선택지

| 방식 | 설명 | 권장 |
|------|------|------|
| **A. launchd plist** | macOS 네이티브 스케줄러, 별도 데몬 불필요 | MVP 권장 |
| B. 자체 daemon | Rust 바이너리가 백그라운드 루프 실행 | 복잡도 높음 |
| C. 런처 시작 시 수집 + 캐시 | 런처 열 때 한 번 수집, 이후 캐시 사용 | 최초 열기 느림 |
| D. fswatch/inotify 이벤트 | iTerm2 세션 변경 감지 시 수집 | macOS에서 세션 변경 이벤트 없음 |

**방식 A (launchd)를 MVP 기본으로 권장한다.**

```xml
<!-- ~/Library/LaunchAgents/com.ccx.collector.plist -->
<plist>
  <dict>
    <key>ProgramArguments</key>
    <array>
      <string>/path/to/ccx</string>
      <string>collect</string>
      <string>--output</string>
      <string>~/.cache/ccx/sessions.json</string>
    </array>
    <key>StartInterval</key>
    <integer>5</integer>
  </dict>
</plist>
```

### 캐시 경로

- `~/.cache/ccx/sessions.json` (XDG) 또는
- `~/Library/Caches/ccx/sessions.json` (macOS 관례)
- Rust에서 `dirs::cache_dir()` 사용

### Staleness 허용 범위

- 세션 목록은 5초 이내 갱신이면 실용적으로 충분
- 새 세션을 열고 5초 후면 검색 가능
- 닫힌 세션이 5초간 남아 있어도 focus 실패 시 graceful 처리

## Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| 현재 유지 (one-shot) | 단순, 항상 최신 | 매번 0.5~2초 대기, PRD 목표 미달 |
| iTerm2 Python API 사용 | 실시간 이벤트 가능 | Python 런타임 의존, Rust 포팅과 충돌 |
| 자체 daemon (방식 B) | 세밀한 제어 | PID 관리, 프로세스 라이프사이클 복잡도 |
| 런처 시작 시 수집 (방식 C) | 데몬 불필요 | 최초 열기 느림 (1~2초), 장시간 열면 stale |

## Consequences

### Positive
- 런처 응답 시간 <50ms 달성 가능 (파일 읽기 + 검색만)
- osascript 비용이 사용자 체감 경로에서 완전 제거
- launchd는 macOS 네이티브 — 추가 의존성 없음, 로그인 시 자동 시작

### Negative
- 데몬 설치/해제 UX 필요 (`ccx install` / `ccx uninstall` 커맨드)
- 캐시 staleness 윈도우 (최대 N초) 존재
- 백그라운드 프로세스가 시스템 리소스 미미하게 소비

### Neutral
- `collect` 커맨드 자체는 그대로 유지 — 데몬이 내부적으로 같은 로직 호출
- `--cache` 플래그도 유지 — 데몬 없이도 수동 워크플로 가능

## Implementation Notes

### MVP 단계

1. `collect --output <path>` 는 이미 구현됨 — 데몬이 이걸 호출
2. 캐시 경로를 CWD → XDG/macOS 표준 경로로 변경 (코드 리뷰 M5 반영)
3. `ccx install` — launchd plist 생성 + `launchctl load`
4. `ccx uninstall` — `launchctl unload` + plist 삭제
5. `pick` 기본 동작을 `--cache` 모드로 변경 (데몬 설치 시)

### 향후 확장

- 캐시 포맷을 JSON → 바이너리(bincode/MessagePack)로 변경하여 파싱 속도 향상
- 파일 기반 → Unix domain socket IPC로 전환 (데몬이 메모리에 인덱스 유지)
- 세션 변경 감지 (diff 기반) — 변경 없으면 파일 쓰기 skip

## References

- [PRD](../../PRD.md) — 14절 성능 요구사항, 7.2절 v1 확장 범위
- [Code Review](./../.agent/code-review-2026-03-18.md) — M5 (cache 경로), H3 (cwd 수집)
- [ADR-003](2026-03-18-use-rust-for-primary-implementation.md) — Rust 기반 구현
