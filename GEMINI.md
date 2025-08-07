Rust製テキストエディタ
replace callせずになるべくfindとsedで一気にやる
完了後cargo test --features test で検証する
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

文字入力すると一文字ずつundo historyに追加されるのですが、それでは頻度が高すぎるので、
ある一定時間入力が止まったときだけにしてほしいです。
削除も同様です。
