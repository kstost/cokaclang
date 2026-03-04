함수 make(seed) {
    변수 s = seed;
    함수 inc(v) { s = s + v; 반환 s; }
    반환 inc;
}
내보내기("make", make);
