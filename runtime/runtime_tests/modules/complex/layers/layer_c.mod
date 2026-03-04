함수 합(a, b) { 반환 a + b; }
함수 곱(a, b) { 반환 a * b; }
함수 makeCounter(seed) {
    변수 s = seed;
    함수 inc(v) { s = s + v; 반환 s; }
    반환 inc;
}
내보내기("합", 합);
내보내기("곱", 곱);
내보내기("makeCounter", makeCounter);
