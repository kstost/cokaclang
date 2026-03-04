# COKAC VSCode 확장 사용 가이드

이 문서는 `tools/vscode-cokac-language` 확장을 실제로 사용하는 방법을 정리합니다.

## 1) 확장 목적

- `.cokac`, `*.mod.cokac` 파일을 VSCode에서 언어로 인식
- 키워드/문자열/숫자/주석/연산자 기본 하이라이트 제공
- 기본 코드 스니펫 제공

## 2) 가장 빠른 확인 방법 (개발 모드)

> 새 VSCode 창(Extension Development Host)이 뜨는 방식입니다.
> 기존 VSCode 환경을 오염시키지 않기 위한 정상 동작입니다.

1. VSCode에서 아래 폴더를 **루트로 열기**
   - `tools/vscode-cokac-language`
2. `Run and Debug`에서 `Run COKAC Extension` 선택
3. `F5`
4. 새로 뜬 VSCode 창에서 `.cokac` 파일 열어 하이라이트 확인

## 3) 기존 VSCode에 바로 설치해서 사용 (VSIX)

### 3-1. `vsce`가 설치된 경우

```bash
cd /Users/kst/VMIMGS/vmware_ubuntu_shared/cokacdir/tools/vscode-cokac-language
vsce package
code --install-extension cokac-language-0.1.0.vsix
```

### 3-2. `vsce`가 없거나 전역 설치 권한이 없는 경우 (권장)

```bash
cd /Users/kst/VMIMGS/vmware_ubuntu_shared/cokacdir/tools/vscode-cokac-language
npx @vscode/vsce package
code --install-extension cokac-language-0.1.0.vsix
```

## 4) 자주 발생한 오류와 해결

### 오류 A

- 메시지: `You don't have an extension for debugging JSON...`
- 원인: 확장 프로젝트가 아닌 JSON 파일 자체를 디버깅하려고 한 경우
- 해결: `tools/vscode-cokac-language` 폴더를 루트로 열고 `Run COKAC Extension`으로 F5 실행

### 오류 B

- 메시지: `Failed loading extension ... property activationEvents should be omitted if the extension doesn't have a main or browser property`
- 원인: 문법 전용 확장에 `activationEvents`가 들어가 있었음
- 해결: `package.json`에서 `activationEvents` 제거(이미 반영됨)

### 오류 C

- 메시지: `zsh: command not found: vsce`
- 해결: 아래 중 하나 선택
  1. 전역 설치: `npm i -g @vscode/vsce`
  2. 전역 없이: `npx @vscode/vsce package`

### 오류 D

- 메시지: `EACCES: permission denied, mkdir '/usr/lib/node_modules/@vscode'`
- 원인: 전역 npm 설치 권한 부족
- 해결:
  1. `sudo npm i -g @vscode/vsce` 사용
  2. 또는 `npx @vscode/vsce package` 사용(권장)

## 5) 설치 확인

1. VSCode에서 `.cokac` 파일 열기
2. 우측 하단 언어 모드가 `COKAC`인지 확인
3. 키워드(`함수`, `변수`, `만약`, `반환`, `시도` 등) 색상이 적용되는지 확인

## 6) 현재 범위와 다음 단계

현재 확장은 하이라이트/스니펫 중심입니다.

아래 기능은 아직 없음:
- 진단 오류 표시
- 자동완성
- 정의로 이동
- 리네임

이 기능들은 별도 LSP(Language Server) 구현이 필요합니다.
