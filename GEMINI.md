Rust製テキストエディタ
完了後cargo testで検証する
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

Ctrl+Cのバグ修正

- Press Ctrl+C again to quit. というメッセージが表示されない
- 1回押した後に別のキーを押して、またCtrl+Cすると1回目なのに終了してしまう
