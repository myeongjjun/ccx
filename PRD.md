iTerm2 에이전트 세션 런처 명세서

1. 문서 목적

이 문서는 사용자의 실제 작업 방식과 요구사항을 바탕으로, iTerm2 환경에서 Claude Code(cc), Codex(cx) 세션을 빠르게 찾고 포커스 전환할 수 있는 고품질 세션 런처를 설계하기 위한 제품 명세서다.

이 도구는 새로운 터미널을 만드는 것이 아니라, 기존 iTerm2 위에 얹는 검색·인덱싱·전환 레이어를 목표로 한다.

⸻

2. 배경 / 상황 설명

사용자는 현재 다음과 같은 방식으로 작업하고 있다.
	•	주 터미널은 iTerm2다.
	•	tmux 기반 워크플로는 선호하지 않는다.
	•	Claude Code와 Codex를 여러 세션으로 동시에 띄워 작업한다.
	•	관심사는 여러 에이전트가 한 프로젝트를 병렬 수정하는 오케스트레이션보다, 여러 세션을 쉽게 찾고, 현재 상태를 파악하고, 원하는 세션으로 즉시 이동하는 것에 있다.
	•	iTerm2의 기본 기능인 Open Quickly는 존재하지만, 검색 품질이 충분히 좋지 않고, 특히 영문 검색이나 fuzzy 탐색 경험이 만족스럽지 않다.
	•	따라서 사용자는 cc, cx, pwd path 등을 기준으로 더 정확한 fuzzy 검색을 하고, 검색 결과 중 원하는 세션으로 즉시 포커스 전환할 수 있는 도구를 원한다.

즉, 문제의 본질은 다음과 같다.

“tmux 없이 iTerm2만 쓰면서, Claude Code / Codex 세션을 더 잘 찾고 더 빨리 전환할 수 있는 전용 검색기/런처가 필요하다.”

⸻

3. 문제 정의

현재 iTerm2 기본 기능만으로는 아래 문제가 있다.
	1.	검색 품질 부족
	•	부분 문자열 검색과 fuzzy 탐색이 기대만큼 강하지 않다.
	•	영문 약어, path segment, task 키워드 조합 탐색 품질이 낮다.
	2.	에이전트 세션 특화 정보 부족
	•	Claude Code, Codex 세션을 의미적으로 구분하기 어렵다.
	•	단순히 탭 제목을 찾는 수준을 넘어서기 어렵다.
	3.	전환 UX 제한
	•	원하는 세션을 정확히 찾기까지 시간이 걸린다.
	•	세션 전환 속도와 정확도가 생산성에 직접 영향을 준다.
	4.	사용자 워크플로와 불일치
	•	사용자는 iTerm2를 유지하고 싶으며, tmux로 작업 방식을 바꾸고 싶지 않다.
	•	따라서 별도 터미널 생태계가 아니라 기존 iTerm2를 보완하는 도구가 필요하다.

⸻

4. 제품 목표

이 도구의 목표는 다음 한 줄로 정의할 수 있다.

iTerm2를 버리지 않고, Claude Code / Codex 세션을 고품질 fuzzy 검색으로 빠르게 찾고 즉시 포커스 전환하는 세션 런처

핵심 목표
	•	iTerm2에서 즉시 실행 가능한 런처 UX 제공
	•	Open Quickly와 유사한 오버레이 UI 제공
	•	기존 Open Quickly보다 검색 품질 향상
	•	cc, cx, pwd/path, repo, task 기준 탐색 지원
	•	검색 결과 선택 시 해당 세션으로 즉시 포커스 이동
	•	iTerm2를 대체하지 않고 보완하는 형태 유지

비목표
	•	새로운 터미널 앱 개발
	•	tmux 대체
	•	멀티에이전트 코딩 오케스트레이션 플랫폼 개발
	•	CI/PR/리뷰 자동화까지 포함한 대형 시스템 구축

⸻

5. 사용자 요구사항

필수 요구사항
	1.	iTerm2 같은 터미널에서 바로 동작해야 한다
	•	사용자는 기존 iTerm2를 계속 사용한다.
	•	별도 터미널 앱으로 갈아타는 것은 원하지 않는다.
	2.	Open Quickly 같은 UI를 제공해야 한다
	•	단축키로 즉시 열리는 오버레이 형태
	•	검색창 + 후보 리스트 구조
	•	키보드 중심 조작
	3.	고급 fuzzy finder 기반 검색 품질이 필요하다
	•	부분 문자열보다 나은 fuzzy match
	•	영어 키워드, path 일부, repo명, 약어 등도 잘 검색되어야 한다.
	4.	cc, cx, pwd path를 기준으로 세션을 찾을 수 있어야 한다
	•	예: cc hi
	•	예: cx api
	•	예: path hi
	•	예: repo bug
	5.	검색 결과 중 해당 세션으로 즉시 포커스 전환해야 한다
	•	해당 탭 선택
	•	해당 윈도우를 앞으로 가져오기
	•	해당 세션 활성화

기대 UX 예시
	•	path hi 로 검색하면
	•	cc + /path/hi/
	•	cx + /path/hi/
관련 세션이 상위 후보로 노출된다.
	•	cc auth 로 검색하면 Claude Code 세션 중 auth 관련 작업이 우선 노출된다.
	•	cx api test 로 검색하면 Codex 세션 중 API 테스트 관련 세션이 우선 노출된다.

⸻

6. 제품 컨셉

이 도구는 다음 성격을 가진다.
	•	터미널 대체재가 아니다
	•	세션 찾기/전환 전용 런처다
	•	에이전트 세션에 최적화된 Open Quickly 대체재다

한 줄로 표현하면:

“Claude Code / Codex 세션에 최적화된 iTerm2 Open Quickly replacement”

⸻

7. 기능 범위

7.1 MVP 범위

MVP에서는 아래 기능만 우선 구현한다.
	1.	iTerm2 세션 목록 수집
	2.	각 세션의 최소 메타데이터 인덱싱
	3.	fuzzy 검색
	4.	선택한 세션으로 포커스 이동

MVP에서 수집할 메타데이터
	•	session_id
	•	window_id
	•	tab_id
	•	title
	•	badge
	•	cwd 또는 현재 path
	•	foreground_command
	•	last_active_at
	•	session_type (cc, cx, other)

⸻

7.2 v1 확장 범위
	•	최근 활성 세션 가중치
	•	repo 이름 추출
	•	cc, cx 타입 필터 강화
	•	글로벌 hotkey
	•	세션 목록 캐시

7.3 v2 확장 범위
	•	최근 출력 일부 미리보기
	•	waiting / busy / idle 상태 추정
	•	즐겨찾기 세션
	•	최근 검색 이력
	•	고급 검색 문법

7.4 v3 확장 범위
	•	Claude/Codex 실행 wrapper 연동
	•	세션 상태 메타데이터 강화
	•	메뉴바 모드
	•	Raycast / Alfred 연동

⸻

8. 데이터 모델

SessionRecord

interface SessionRecord {
  sessionId: string
  windowId: string
  tabId: string
  title: string
  badge?: string
  cwd?: string
  repoName?: string
  foregroundCommand?: string
  sessionType: 'cc' | 'cx' | 'other'
  lastActiveAt?: number
  scoreHints?: {
    exactType?: boolean
    exactRepo?: boolean
    exactPathSegment?: boolean
    recentlyActive?: boolean
  }
}


⸻

9. 세션 타입 판별 규칙

이 툴의 정확도에서 가장 중요한 부분은 세션 타입 인식이다.

1차: 추론 기반 판별

다음 정보로 cc, cx, other를 추론한다.
	•	foreground command
	•	process name
	•	title
	•	badge

예시:
	•	claude, claude-code 계열 → cc
	•	codex 계열 → cx
	•	그 외 → other

2차: 명시 기반 판별

정확도를 높이기 위해, 가능하면 Claude/Codex 실행 시 명시적 표식을 남긴다.

예시:
	•	title prefix: CC | repo | task, CX | repo | task
	•	badge: cc, cx
	•	env: AGENT_KIND=cc, AGENT_KIND=cx

권장한다.

⸻

10. 검색 설계

10.1 검색 대상 필드
	•	session_type
	•	cwd
	•	repo_name
	•	title
	•	badge
	•	foreground_command

10.2 토큰화 규칙

검색 품질 향상을 위해 다음 방식으로 색인한다.
	•	공백 기준 토큰화
	•	path는 / 기준 segment 분해
	•	repo/title은 -, _, . 기준 분해
	•	basename(repo명) 별도 추출

예시:
	•	/Users/me/work/path-history → users, me, work, path, history, path-history
	•	auth-api_server → auth, api, server, auth-api_server

10.3 검색 문법

기본은 자연 검색으로 한다.

예:
	•	cc hi
	•	cx api
	•	path hi
	•	repo bug
	•	review

고급 사용자용 얇은 문법은 선택적으로 지원한다.
	•	p:foo → path 우선
	•	r:api → repo 우선
	•	t:bug → title 우선

단, 초기 버전에서는 문법보다 자연 검색 품질을 우선한다.

⸻

11. 랭킹 / 스코어링 규칙

단순 fuzzy score만으로는 부족하므로, 규칙 기반 보정을 반드시 넣는다.

기본 점수 요소
	•	fuzzy match score
	•	exact match 여부
	•	prefix match 여부
	•	path segment match 여부
	•	repo basename match 여부
	•	session type match 여부
	•	recently active bonus

우선순위 예시
	1.	정확한 타입 일치 (cc, cx)
	2.	path segment 일치
	3.	repo basename 일치
	4.	title/badge 일치
	5.	최근 활성 세션 bonus

예시

질의: path hi

후보:
	•	cc | /path/hi/feature-x
	•	cx | /path/hi/bugfix
	•	cc | /work/highlight
	•	cx | /repo/history-api

이 경우:
	•	/path/hi/ segment 정확 일치가 가장 높은 점수
	•	highlight, history는 fuzzy 매칭으로만 점수 부여

⸻

12. 포커스 전환 UX

검색 결과에서 항목을 선택하면 다음 동작이 즉시 수행되어야 한다.
	1.	해당 탭 선택
	2.	해당 윈도우를 전면으로 가져오기
	3.	해당 세션 활성화

UX 원칙
	•	사용자는 “검색 → Enter → 즉시 이동”을 기대한다.
	•	전환 속도는 검색 품질만큼 중요하다.
	•	이 도구는 검색기이면서 동시에 jump tool이다.

⸻

13. UI 명세

기본 UI 형태
	•	글로벌 hotkey로 호출
	•	상단 또는 중앙 오버레이
	•	한 줄 입력창
	•	아래 결과 리스트 5~10개 노출
	•	방향키/Ctrl+n,Ctrl+p 이동
	•	Enter로 포커스 이동
	•	Esc로 종료

후보 표시 예시

CC  repo-a   /Users/me/work/repo-a/feature/auth   auth fix
CX  repo-b   /Users/me/work/repo-b/tests/api      api test
CC  repo-c   /Users/me/work/repo-c/refactor       review

표시 항목
	•	세션 타입 (CC, CX, OT)
	•	repo명
	•	path
	•	title 또는 task 요약
	•	필요 시 최근 활성 시각

⸻

14. 성능 요구사항
	•	세션 목록 갱신은 빠르고 가벼워야 한다.
	•	인덱스 구축에 과도한 비용이 들면 안 된다.
	•	초기 버전에서는 화면 전체 출력 파싱보다 title/path/type 중심으로 가볍게 유지한다.
	•	포커스 전환은 체감상 즉시 이뤄져야 한다.

권장 원칙
	•	실시간 전체 텍스트 OCR/파싱은 하지 않는다.
	•	우선은 title + cwd + type + recent active 중심으로 설계한다.
	•	필요 시 후속 버전에서 preview를 추가한다.

⸻

15. 기술 방향

권장 구현 철학
	•	새 터미널 개발이 아님
	•	iTerm2 세션 조회 + fuzzy 검색 + activate에 집중
	•	가능한 한 범위를 작게 유지

구현 방향 후보
	1.	macOS 네이티브 런처 앱
	2.	백그라운드 인덱서 + 오버레이 UI
	3.	iTerm2 스크립팅/API 연동
	4.	로컬 인메모리 인덱스 + fuzzy 라이브러리

구현에서 중요한 점
	•	iTerm2 세션 정보를 안정적으로 읽을 수 있어야 한다.
	•	세션 활성화가 확실하게 동작해야 한다.
	•	fuzzy 품질은 일반 문자열 검색보다 확실히 좋아야 한다.

⸻

16. 사용자 실행 규칙 권장안

도구 정확도를 높이기 위해, 사용자는 Claude/Codex 세션 실행 시 다음 규칙을 지키는 것이 좋다.

권장 title 규칙
	•	CC | repo | task
	•	CX | repo | task

권장 badge 규칙
	•	cc
	•	cx
	•	wait
	•	review

기대 효과
	•	타입 판별 정확도 상승
	•	검색 결과 품질 향상
	•	Open Quickly와 병행 사용 시에도 효과

⸻

17. 대표 사용 시나리오

시나리오 1: Claude 세션 빠르게 찾기

사용자가 cc auth를 입력한다.

기대 결과:
	•	Claude Code 세션만 우선 필터링
	•	auth 관련 repo/path/title 세션이 상단 노출
	•	Enter 시 해당 세션으로 즉시 이동

시나리오 2: Codex 세션 중 특정 path 찾기

사용자가 cx api를 입력한다.

기대 결과:
	•	Codex 세션 중 API 관련 작업 세션 상단 노출

시나리오 3: path 기준 탐색

사용자가 path hi를 입력한다.

기대 결과:
	•	/path/hi/ segment를 포함한 세션 우선 노출
	•	cc + /path/hi/, cx + /path/hi/ 세션이 상위 후보에 표시

시나리오 4: 그냥 repo/task 검색

사용자가 review를 입력한다.

기대 결과:
	•	title, badge, repo, path 전체에서 review와 관련된 세션이 점수순으로 노출

⸻

18. MVP 성공 기준

다음이 되면 MVP는 성공한 것으로 본다.
	1.	iTerm2 세션을 안정적으로 수집할 수 있다.
	2.	cc, cx, path, repo, task 기준 검색이 잘 된다.
	3.	Open Quickly보다 사용자가 체감하는 검색 만족도가 높다.
	4.	원하는 세션으로 빠르게 포커스 전환된다.
	5.	사용자가 tmux 없이도 세션 관리 스트레스를 줄일 수 있다.

⸻

19. 향후 발전 방향

이 툴은 이후 다음 방향으로 확장 가능하다.
	•	waiting/busy/idle 상태 추정
	•	최근 출력 미리보기
	•	세션 우선순위 추천
	•	메뉴바 연동
	•	Raycast/Alfred extension 제공
	•	Claude/Codex wrapper와 직접 연계
	•	팀 공유용 세션 상태 보드

다만 초기에는 반드시 세션 검색 + 포커스 전환에 집중한다.

⸻

20. 최종 요약

이 프로젝트는 다음 문제를 해결하기 위한 것이다.

“tmux 없이 iTerm2만 쓰는 환경에서, Claude Code / Codex 세션을 Open Quickly보다 더 잘 찾고 더 빠르게 전환하고 싶다.”

따라서 이 도구는:
	•	iTerm2를 유지하면서
	•	Open Quickly와 유사한 얇은 UI를 제공하고
	•	고급 fuzzy 인덱싱으로 검색 품질을 높이며
	•	cc, cx, pwd path, repo, task 기준으로 세션을 찾고
	•	원하는 세션으로 즉시 포커스 이동하는

에이전트 세션 전용 런처로 정의된다.
