# `/task`コマンド実装 TODOリスト

## 1. エディタの状態管理の追加
*   [ ] `src/editor.rs`:
    *   [ ] `EditorMode` enumの定義 (`Normal`, `TaskSelection`, `Search`)
    *   [ ] `Editor` structに`mode: EditorMode`フィールドを追加
    *   [ ] `Editor` structに`task_list: Vec<String>`フィールドを追加（表示するタスクのリスト）
    *   [ ] `Editor` structに`selected_task_index: Option<usize>`フィールドを追加（現在選択されているタスクのインデックス）
    *   [ ] `Editor` structに`task_display_offset: usize`フィールドを追加（タスクリストのスクロール用）

## 2. `/task`コマンドのトリガーとタスク検索
*   [ ] `src/editor.rs`の`insert_newline`関数内で、コマンド実行ロジックを拡張。
*   [ ] 入力された行が`/task`コマンドと一致する場合、タスク選択モードに移行する処理を追加。
*   [ ] `Document`から現在のカーソル位置より下の未チェックタスク (`- [ ] `) を検索し、`task_list`に格納する関数を実装。

## 3. 入力処理の変更 (`src/editor/input.rs`)
*   [ ] `handle_key_event`のような関数内で、`editor.mode`に基づいて入力処理を分岐。
*   [ ] `EditorMode::TaskSelection`の場合のキーハンドリングロジックを実装:
    *   [ ] 上下矢印キーでの`selected_task_index`の更新。
    *   [ ] SPACEキーでのタスク移動ロジック（`Document`の変更、`ActionDiff`の生成、`commit`）。
    *   [ ] ESC/ENTERキーでのモード終了ロジック。

## 4. 描画処理の変更 (`src/editor/ui.rs`)
*   [ ] `draw`関数内で`editor.mode`をチェック。
*   [ ] `EditorMode::TaskSelection`の場合の描画ロジックを実装:
    *   [ ] タスク選択UIの表示領域（例: `TASK_UI_HEIGHT`定数）を定義。
    *   [ ] `task_list`の内容を`window.mvaddstr`で描画。
    *   [ ] `selected_task_index`の行を`A_REVERSE`でハイライト。
    *   [ ] タスクリストのスクロール（`task_display_offset`）を考慮した描画。
    *   [ ] メインドキュメントの描画開始行を`TASK_UI_HEIGHT`分だけ下にずらす。
    *   [ ] `scroll()`関数内の`visible_content_height`の計算も`TASK_UI_HEIGHT`を考慮するように変更。

## 5. タスク移動ロジックの実装
*   [ ] 選択されたタスクを元の位置から削除し、現在のカーソル位置に挿入するロジックを実装。
*   [ ] `ActionDiff`と`commit`を使用して、Undo/Redoが可能なようにする。

## 6. テスト
*   [ ] 新しいモードへの移行、キー入力による選択、タスク移動、モード終了の各シナリオに対するテストを追加。
*   [ ] `cargo test`で既存のテストが壊れていないことを確認。