Rust製テキストエディタ
完了後 cargo test --test '*' で検証する
ユーザーに動作確認をしてもらうときは cargo build でビルドを通す
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

> documentのjoin_line_with_next, insert_newlineを
> document.modify(x, y, "added_text", "deleted_text", false/* isUndo */) に統一したいです。
上記、完了しました

documentのinsert_newlineを削除し、
editorからはdocument.modifyを直接呼ぶようにしてください