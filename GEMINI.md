Rust製テキストエディタ
replace callせずになるべくfindとsedで一気にやる
完了後cargo testで検証する
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

testで editor.handle_keypress を呼び出しているところは editor.process_input に統一したい。 handle_keypress はprivateにできるのではないか？
