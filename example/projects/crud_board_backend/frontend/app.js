const 요소 = {
  목록탭버튼: document.getElementById("목록탭버튼"),
  글쓰기탭버튼: document.getElementById("글쓰기탭버튼"),
  목록화면: document.getElementById("목록화면"),
  상세화면: document.getElementById("상세화면"),
  작성화면: document.getElementById("작성화면"),
  상태문구: document.getElementById("상태문구"),

  검색어: document.getElementById("검색어"),
  상태필터: document.getElementById("상태필터"),
  검색버튼: document.getElementById("검색버튼"),
  게시글표본문: document.getElementById("게시글표본문"),
  이전페이지버튼: document.getElementById("이전페이지버튼"),
  다음페이지버튼: document.getElementById("다음페이지버튼"),
  페이지정보: document.getElementById("페이지정보"),

  상세제목: document.getElementById("상세제목"),
  상세메타: document.getElementById("상세메타"),
  상세내용: document.getElementById("상세내용"),
  상세태그: document.getElementById("상세태그"),
  좋아요버튼: document.getElementById("좋아요버튼"),
  수정모드버튼: document.getElementById("수정모드버튼"),
  삭제버튼: document.getElementById("삭제버튼"),
  목록복귀버튼: document.getElementById("목록복귀버튼"),

  폼제목: document.getElementById("폼제목"),
  입력제목: document.getElementById("입력제목"),
  입력작성자: document.getElementById("입력작성자"),
  입력상태: document.getElementById("입력상태"),
  입력태그: document.getElementById("입력태그"),
  입력내용: document.getElementById("입력내용"),
  저장버튼: document.getElementById("저장버튼"),
  취소버튼: document.getElementById("취소버튼")
};

const 상태 = {
  api기본주소: window.location.origin,
  현재페이지: 1,
  페이지크기: 10,
  전체개수: 0,
  현재목록: [],
  선택아이디: null,
  수정모드: false
};

function 알림(문구, 오류 = false) {
  요소.상태문구.textContent = 문구;
  요소.상태문구.style.color = 오류 ? "#b91c1c" : "#0f766e";
}

function 태그파싱(문자열값) {
  return 문자열값
    .split(",")
    .map((v) => v.trim())
    .filter((v) => v.length > 0);
}

function 날짜짧게(문자열값) {
  if (!문자열값) return "-";
  return 문자열값;
}

function 화면전환(화면) {
  요소.목록화면.classList.add("숨김");
  요소.상세화면.classList.add("숨김");
  요소.작성화면.classList.add("숨김");

  요소.목록탭버튼.classList.remove("탭활성");
  요소.글쓰기탭버튼.classList.remove("탭활성");

  if (화면 === "목록") {
    요소.목록화면.classList.remove("숨김");
    요소.목록탭버튼.classList.add("탭활성");
  }
  if (화면 === "상세") {
    요소.상세화면.classList.remove("숨김");
    요소.목록탭버튼.classList.add("탭활성");
  }
  if (화면 === "작성") {
    요소.작성화면.classList.remove("숨김");
    요소.글쓰기탭버튼.classList.add("탭활성");
  }
}

async function API요청(경로, 메서드 = "GET", 본문 = null) {
  const 설정 = {
    method: 메서드,
    headers: { "Content-Type": "application/json" }
  };
  if (본문) 설정.body = JSON.stringify(본문);

  const 응답 = await fetch(`${상태.api기본주소}${경로}`, 설정);
  let 데이터 = null;
  try {
    데이터 = await 응답.json();
  } catch (_) {
    데이터 = null;
  }

  if (!응답.ok) {
    throw new Error((데이터 && 데이터.메시지) || `요청 실패 (${응답.status})`);
  }

  return 데이터;
}

function 목록렌더(목록) {
  요소.게시글표본문.innerHTML = "";

  if (!목록 || 목록.length === 0) {
    요소.게시글표본문.innerHTML = '<tr><td colspan="7">게시글이 없습니다.</td></tr>';
    return;
  }

  목록.forEach((글) => {
    const 행 = document.createElement("tr");
    행.innerHTML = `
      <td>${글.아이디}</td>
      <td class="제목셀">${(글.제목 || "").replace(/</g, "&lt;")}</td>
      <td>${글.작성자 || "-"}</td>
      <td>${글.상태 || "-"}</td>
      <td>${글.조회수 ?? 0}</td>
      <td>${글.좋아요수 ?? 0}</td>
      <td>${날짜짧게(글.수정시각)}</td>
    `;
    행.addEventListener("click", () => 상세열기(글.아이디));
    요소.게시글표본문.appendChild(행);
  });
}

function 페이지정보렌더() {
  const 총페이지 = Math.max(1, Math.ceil(상태.전체개수 / 상태.페이지크기));
  요소.페이지정보.textContent = `${상태.현재페이지} / ${총페이지}`;
  요소.이전페이지버튼.disabled = 상태.현재페이지 <= 1;
  요소.다음페이지버튼.disabled = 상태.현재페이지 >= 총페이지;
}

async function 목록불러오기() {
  const 검색 = encodeURIComponent(요소.검색어.value.trim());
  const 상태값 = encodeURIComponent(요소.상태필터.value);
  const 경로 = `/posts?페이지=${상태.현재페이지}&크기=${상태.페이지크기}&검색=${검색}&작성자=&상태=${상태값}`;

  const 데이터 = await API요청(경로);
  상태.현재목록 = 데이터.목록 || [];
  상태.전체개수 = 데이터.전체개수 || 0;

  목록렌더(상태.현재목록);
  페이지정보렌더();
  알림(`게시글 ${상태.전체개수}건`);
}

function 상세렌더(글) {
  요소.상세제목.textContent = 글.제목 || "(제목 없음)";
  요소.상세메타.textContent = `작성자 ${글.작성자 || "-"} | 상태 ${글.상태 || "-"} | 조회수 ${글.조회수 || 0} | 좋아요 ${글.좋아요수 || 0} | 수정 ${글.수정시각 || "-"}`;
  요소.상세내용.textContent = 글.내용 || "";

  요소.상세태그.innerHTML = "";
  const 태그배열 = Array.isArray(글.태그) ? 글.태그 : [];
  태그배열.forEach((태그) => {
    const span = document.createElement("span");
    span.className = "태그";
    span.textContent = `#${태그}`;
    요소.상세태그.appendChild(span);
  });
}

async function 상세열기(아이디) {
  const 데이터 = await API요청(`/posts/${아이디}`);
  상태.선택아이디 = 아이디;
  상세렌더(데이터.게시글);
  화면전환("상세");
  await 목록불러오기();
}

function 폼초기화() {
  요소.입력제목.value = "";
  요소.입력작성자.value = "";
  요소.입력상태.value = "게시";
  요소.입력태그.value = "";
  요소.입력내용.value = "";
}

function 수정폼채우기(글) {
  요소.입력제목.value = 글.제목 || "";
  요소.입력작성자.value = 글.작성자 || "";
  요소.입력상태.value = 글.상태 || "게시";
  요소.입력태그.value = Array.isArray(글.태그) ? 글.태그.join(",") : "";
  요소.입력내용.value = 글.내용 || "";
}

function 폼본문생성() {
  const 제목 = 요소.입력제목.value.trim();
  const 작성자 = 요소.입력작성자.value.trim();
  const 내용 = 요소.입력내용.value.trim();
  if (!제목 || !작성자 || !내용) {
    throw new Error("제목, 작성자, 내용은 필수입니다.");
  }

  return {
    제목,
    작성자,
    내용,
    상태: 요소.입력상태.value,
    태그: 태그파싱(요소.입력태그.value)
  };
}

async function 저장하기() {
  const 본문 = 폼본문생성();

  if (상태.수정모드 && 상태.선택아이디) {
    await API요청(`/posts/${상태.선택아이디}`, "PUT", 본문);
    알림("게시글을 수정했습니다.");
    await 상세열기(상태.선택아이디);
    return;
  }

  const 생성결과 = await API요청("/posts", "POST", 본문);
  알림("게시글을 등록했습니다.");
  상태.수정모드 = false;
  폼초기화();
  await 상세열기(생성결과.게시글.아이디);
}

async function 좋아요() {
  if (!상태.선택아이디) return;
  await API요청(`/posts/${상태.선택아이디}/like`, "POST");
  await 상세열기(상태.선택아이디);
  알림("좋아요를 반영했습니다.");
}

async function 삭제하기() {
  if (!상태.선택아이디) return;
  const 확인 = window.confirm("정말 삭제하시겠습니까?");
  if (!확인) return;

  await API요청(`/posts/${상태.선택아이디}`, "DELETE");
  상태.선택아이디 = null;
  알림("삭제했습니다.");
  화면전환("목록");
  await 목록불러오기();
}

async function 안전실행(작업) {
  try {
    await 작업();
  } catch (오류) {
    알림(오류.message || String(오류), true);
  }
}

요소.목록탭버튼.addEventListener("click", () => {
  상태.수정모드 = false;
  화면전환("목록");
  안전실행(목록불러오기);
});

요소.글쓰기탭버튼.addEventListener("click", () => {
  상태.수정모드 = false;
  상태.선택아이디 = null;
  요소.폼제목.textContent = "글쓰기";
  폼초기화();
  화면전환("작성");
});

요소.검색버튼.addEventListener("click", () => {
  상태.현재페이지 = 1;
  안전실행(목록불러오기);
});

요소.이전페이지버튼.addEventListener("click", () => {
  if (상태.현재페이지 <= 1) return;
  상태.현재페이지 -= 1;
  안전실행(목록불러오기);
});

요소.다음페이지버튼.addEventListener("click", () => {
  const 총페이지 = Math.max(1, Math.ceil(상태.전체개수 / 상태.페이지크기));
  if (상태.현재페이지 >= 총페이지) return;
  상태.현재페이지 += 1;
  안전실행(목록불러오기);
});

요소.목록복귀버튼.addEventListener("click", () => {
  화면전환("목록");
  안전실행(목록불러오기);
});

요소.수정모드버튼.addEventListener("click", async () => {
  if (!상태.선택아이디) return;
  await 안전실행(async () => {
    const 데이터 = await API요청(`/posts/${상태.선택아이디}`);
    상태.수정모드 = true;
    요소.폼제목.textContent = `글수정 #${상태.선택아이디}`;
    수정폼채우기(데이터.게시글);
    화면전환("작성");
  });
});

요소.좋아요버튼.addEventListener("click", () => 안전실행(좋아요));
요소.삭제버튼.addEventListener("click", () => 안전실행(삭제하기));
요소.저장버튼.addEventListener("click", () => 안전실행(저장하기));
요소.취소버튼.addEventListener("click", () => {
  화면전환(상태.선택아이디 ? "상세" : "목록");
});

(async () => {
  화면전환("목록");
  await 안전실행(async () => {
    await API요청("/health");
    await 목록불러오기();
  });
})();
