함수 calc(x, y) {
    변수 B = 모듈가져오기("modules/complex/layers/layer_b.mod");
    반환 B.score(x, y);
}
함수 counter(seed) {
    변수 B = 모듈가져오기("modules/complex/layers/layer_b.mod");
    반환 B.newCounter(seed);
}
내보내기("calc", calc);
내보내기("counter", counter);
