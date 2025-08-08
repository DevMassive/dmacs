Rust製テキストエディタ
完了後 cargo test --test '*' で検証する
ユーザーに動作確認をしてもらうときは cargo build でビルドを通す
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

documentのlineをmergeしたり分けたりする関数がいくつも種類があるのが嫌です。
他にも行を編集する関数がありすぎです。
document.modify(x, y, "added_text", "deleted_text", false/* isUndo */) だけがあれば十分に思います。
この関数はx,yを起点として文字追加と削除を行います。isUndoがtrueのときはadded_textを消してdeleted_textを追加します。
返り値はこの変更後の新しいカーソルのポジションです。
