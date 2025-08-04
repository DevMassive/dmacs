Rust製テキストエディタ
replace callせずになるべくfindとsedで一気にやる
完了後cargo testで検証する
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

以下の新機能を追加して

- このエディタでは「---」を特別な区切りとして扱う
- 「---」があるばあい、「―――」として表示する
- 「----」とか「--」は表示を変えない。3つの-のみを変える
