@echo off
rem ローカル開発用の SurrealDB サーバーを起動する。
rem ダブルクリック、または `scripts\start-db.bat` で実行できる。
rem 終了するにはこのウィンドウで Ctrl+C。

cd /d "%~dp0\.."
surreal start --user root --pass root --bind 127.0.0.1:8000 "surrealkv://data/dev.db"
