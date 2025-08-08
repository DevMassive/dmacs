Rust製テキストエディタ
完了後 cargo test --test '*' で検証する
ユーザーに動作確認をしてもらうときは cargo build でビルドを通す
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

documentのsplit_line_fromを
document.modify(x, y, "added_text", "deleted_text", false/* isUndo */) に統一したいです。

統一後、テストを通してください。

その後、documentのsplit_line_fromを削除し、
editorからはdocument.modifyを直接呼ぶようにしてください