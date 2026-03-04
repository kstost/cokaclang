# CRUD Board Backend + Frontend (COKAC)

포트 `3000`에서 동작하는 게시판 CRUD 백엔드이며, 동일 서버가 프론트엔드 정적 파일도 함께 서빙합니다.
저장소는 인메모리/JSON 파일이 아니라 MySQL을 사용합니다.

## 구조

- `src/main.cokac`: 서버 엔트리/의존성 연결
- `src/framework.mod.cokac`: `app.event` 스타일 서버 프레임워크
- `src/config.mod.cokac`: 런타임/DB 설정
- `src/store.mod.cokac`: MySQL 저장소 CRUD
- `src/response.mod.cokac`: JSON/텍스트/CORS 응답 유틸
- `src/router.mod.cokac`: 라우팅/검증/정적 파일 서빙
- `frontend/index.html`: 프론트엔드 화면
- `frontend/styles.css`: 프론트엔드 스타일
- `frontend/app.js`: 프론트엔드 로직(API 연동)

## 사전 준비

- 시스템에 `mysql` CLI가 설치되어 있어야 합니다.
- MySQL 서버 접속이 가능해야 합니다.

환경변수(선택):

- `COKAC_DB_HOST` (기본: `127.0.0.1`)
- `COKAC_DB_PORT` (기본: `3306`)
- `COKAC_DB_USER` (기본: `root`)
- `COKAC_DB_PASSWORD` (기본: 빈값)
- `COKAC_DB_NAME` (기본: `cokac_board`)
- `COKAC_MYSQL_CMD` (기본: `mysql`)

서버 시작 시 DB/테이블을 자동 생성합니다.

## 실행

프로젝트 루트에서:

```bash
./dist/cokaclang-linux-aarch64 example/projects/crud_board_backend/src/main.cokac
```

브라우저 접속:

- `http://127.0.0.1:3000/`

## 정적 파일 라우트

- `GET /` 또는 `GET /index.html`
- `GET /styles.css`
- `GET /app.js`

## API

- `OPTIONS /*` : CORS preflight
- `GET /health` : 헬스 체크
- `GET /stats` : 게시글/조회수/좋아요 통계
- `GET /posts?페이지=1&크기=10&검색=&작성자=&상태=` : 목록 + 필터 + 페이징
- `POST /posts` : 글 생성
- `GET /posts/{id}` : 글 단건 조회(+조회수 증가)
- `POST /posts/{id}/like` : 좋아요 1 증가
- `PUT /posts/{id}` : 전체 수정
- `PATCH /posts/{id}` : 부분 수정
- `DELETE /posts/{id}` : 삭제

요청/응답은 JSON입니다.

## 요청 예시

```bash
curl -s -X POST http://127.0.0.1:3000/posts \
  -H 'Content-Type: application/json' \
  -d '{"제목":"첫 글","내용":"안녕하세요","작성자":"관리자","태그":["공지","테스트"],"상태":"게시"}'
```

## 응답 형태

- 성공 공통: `{"성공":true, ...}`
- 실패 공통: `{"성공":false,"오류":"...","메시지":"..."}`
