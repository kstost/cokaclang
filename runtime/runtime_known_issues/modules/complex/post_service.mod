함수 총점(posts) {
    변수 수학 = 모듈가져오기("modules/complex/math.mod");
    변수 acc = 0;
    변수 likes = 0;
    변수 comments = 0;
    동안 (변수 i = 0; i < 길이(posts); i = i + 1) {
        likes = 길이(posts[i].likes);
        comments = 길이(posts[i].comments);
        acc = 수학.합(acc, 수학.합(수학.곱(likes, 10), comments));
    }
    반환 acc;
}

함수 누적기(초기값) {
    변수 수학 = 모듈가져오기("modules/complex/math.mod");
    변수 s = 초기값;
    함수 add(v) {
        s = 수학.합(s, v);
        반환 s;
    }
    반환 add;
}

내보내기("총점", 총점);
내보내기("누적기", 누적기);
