Rust製テキストエディタ
replace callせずになるべくfindとsedで一気にやる
完了後cargo testで検証する
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

- Control + G など割当てのないキーを押すと特殊な文字が入力されてしまうが、その挙動はなくして、なにもしないようにしたい
- 単純にis_alt_pressedを無視すると、矢印キーによるカーソル移動も無視されてしまう
