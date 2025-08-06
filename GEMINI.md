Rust製テキストエディタ
replace callせずになるべくfindとsedで一気にやる
完了後cargo testで検証する
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

ファイル最後の行を選択している状態でcontrol wを押すとdmacsが終了するのを直す
