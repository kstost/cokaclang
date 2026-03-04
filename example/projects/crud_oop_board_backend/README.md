# CRUD OOP Board Backend + Frontend (COKAC)

`crud_oop_board_backend`는 기존 CRUD 게시판과 동일한 API/프론트 기능을 제공하되,
핵심 호출 흐름을 객체지향 방식(`클래스생성`, `인스턴스생성`, `메서드호출`)으로 구성한 예제입니다.

절차형(비-OOP) 기준 구현 예제:

- `../crud_board_backend/`

## 특징

- 포트 `3000`에서 동작
- 서버가 프론트엔드 정적 파일도 함께 서빙
- 저장소는 MySQL 사용
- 저장소/라우터를 OOP 인스턴스로 생성해 사용

## OOP 적용 포인트

- `src/store.mod.cokac`
  - `저장소_클래스_생성`, `저장소_인스턴스_생성`
  - 저장소 CRUD를 메서드 기반으로 노출
- `src/router.mod.cokac`
  - `라우터_클래스_생성`, `라우터_인스턴스_생성`
  - 요청 처리를 라우터 메서드로 위임
- `src/main.cokac`
  - 저장소/라우터 인스턴스를 생성하고 `메서드호출`로 연결

## 실행

```bash
./dist/cokaclang-linux-aarch64 ./example/projects/crud_oop_board_backend/src/main.cokac
```

브라우저 접속:

- `http://127.0.0.1:3000/`

## DB 설정(기본값)

`src/config.mod.cokac` 기본값:

- `COKAC_DB_HOST=localhost`
- `COKAC_DB_PORT=3306`
- `COKAC_DB_USER=cokac`
- `COKAC_DB_PASSWORD=cokac`
- `COKAC_DB_NAME=cokac`
- `COKAC_MYSQL_CMD=mysql`

필요시 실행 전에 환경변수로 오버라이드할 수 있습니다.

## API

- `OPTIONS /*`
- `GET /health`
- `GET /stats`
- `GET /posts?페이지=1&크기=10&검색=&작성자=&상태=`
- `POST /posts`
- `GET /posts/{id}`
- `POST /posts/{id}/like`
- `PUT /posts/{id}`
- `PATCH /posts/{id}`
- `DELETE /posts/{id}`
