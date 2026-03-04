#!/bin/bash
PORT=3000
PID=$(lsof -t -i:$PORT 2>/dev/null)
if [ -n "$PID" ]; then
  kill $PID 2>/dev/null
  sleep 1
  echo "기존 서버(PID $PID) 종료"
fi
echo "http://localhost:$PORT 서버 시작"
npx -y serve -s "$(dirname "$0")" -l $PORT
