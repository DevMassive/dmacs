Rust製テキストエディタ
replace callせずになるべくfindとsedで一気にやる
完了後cargo testで検証する
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

- Control + N によるデリミタ間の移動 move_to_next_delimiter の反対の機能を実装
- Control + P に割り当てる
- 仮に1つ目のデリミタのすぐ下の行のことをページ1ポジションと呼びましょう
- ファイルの先頭はページ0ポジションです
- 現在の Control + N による移動はカーソルのページポジションの移動と捉えられます
- Control + P による移動はその反対です。
- カーソルよりも上にある一番近いページポジションにジャンプする、という機能です
- testも書いて
