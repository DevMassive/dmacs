Rust製テキストエディタ
replace callせずになるべくfindとsedで一気にやる
完了後cargo testで検証する
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

ui.scrollのテストに垂直方向のスクロールのテストが抜けているので追加してほしい。
いまの実装に合わせたテストにして、実装を変えてはいけません。
