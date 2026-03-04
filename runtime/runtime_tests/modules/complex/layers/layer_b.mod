함수 score(a, b) {
    변수 C = 모듈가져오기("modules/complex/layers/layer_c.mod");
    반환 C.합(C.곱(a, b), C.합(a, b));
}
함수 newCounter(seed) {
    변수 C = 모듈가져오기("modules/complex/layers/layer_c.mod");
    반환 C.makeCounter(seed);
}
내보내기("score", score);
내보내기("newCounter", newCounter);
