Rust製テキストエディタ
replace callせずになるべくfindとsedで一気にやる
完了後cargo testで検証する
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

以下のバグを修正して

- 編集済みのときだけ表示されるはずのステータスバーの「*」が常に表示されている
- 新規ファイルでも既存ファイルでも開いたときから表示されてしまっている
- ファイルの最後に改行があると発生するバグのようだ