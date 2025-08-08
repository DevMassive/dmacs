Rust製テキストエディタ
完了後 cargo test --test '*' で検証する
ユーザーに動作確認をしてもらうときは cargo build でビルドを通す
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

documentのremove_lineを
document.modify(x, y, "added_text", "deleted_text", false/* isUndo */) に統一したいです。

統一後、テストを通してください。

その後、documentのremove_lineを削除し、
editorからはdocument.modifyを直接呼ぶようにしてください