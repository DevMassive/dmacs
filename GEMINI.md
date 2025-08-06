Rust製テキストエディタ
replace callせずになるべくfindとsedで一気にやる
完了後cargo testで検証する
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

1. ステータスバーの高さ=2をどこかに固定値で定義したい
2. ステータスバーとその水平線が表示されている行はカーソルが移動できないようにしたい。
   いまはwindowの一番下までカーソルが移動できるしテキストの編集もできる（テキスト自体はステータスバーに隠れて見えない）。
