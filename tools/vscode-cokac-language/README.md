# COKAC Language Support (VSCode)

`cokac` 언어 파일(`.cokac`, `*.mod.cokac`)의 기본 편집 지원 확장입니다.

## 포함 기능

- 파일 타입 인식: `.cokac`, `*.mod.cokac`
- 문법 하이라이트 (TextMate)
  - 키워드: `함수`, `변수`, `만약`, `반환`, `시도` 등
  - 상수: `참`, `거짓`, `없음`
  - 문자열/숫자/주석/연산자
  - 일부 내장 함수 강조
- 기본 코드 스니펫 제공

## 로컬 개발(권장)

1. VSCode에서 `tools/vscode-cokac-language` 폴더 열기
2. `F5` 실행 (Extension Development Host)
3. 새 창에서 `.cokac` 파일 열어 하이라이트 확인

## VSIX 설치(선택)

`vsce` 설치 후 패키징:

```bash
cd tools/vscode-cokac-language
npm i -g @vscode/vsce
vsce package
```

생성된 `.vsix`를 VSCode에서 설치:

```bash
code --install-extension cokac-language-0.1.0.vsix
```

## 참고

현재 확장은 하이라이트 중심입니다. 진단/자동완성/정의로 이동은 LSP(Language Server) 추가 구현이 필요합니다.
