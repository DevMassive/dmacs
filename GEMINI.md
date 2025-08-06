Rust製テキストエディタ
replace callせずになるべくfindとsedで一気にやる
完了後cargo testで検証する
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

範囲選択してcut/copyする機能を追加して

- control + spaceでカーソル位置にmarkerをおく
- contorl + W でmarkerからカーソル位置までをcut (killと同じでcontrol + Yで貼り付け)
- option + W でcopy
- markerからカーソル位置まではハイライト表示
- contorl + G でmarkerを解除

# 制約事項

まずtestsから書いて
