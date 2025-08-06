Rust製テキストエディタ
replace callせずになるべくfindとsedで一気にやる
完了後cargo testで検証する
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

- dmacsの終了が正常に行われないとき以降terminalの改行表示がおかしくなる。
