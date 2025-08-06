Rust製テキストエディタ
replace callせずになるべくfindとsedで一気にやる
完了後cargo testで検証する
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

- バージョンをアプデしたい
      Adding isatty v0.1.9 (available: v0.2.0)
      Adding thiserror v1.0.69 (available: v2.0.12)
      Adding unicode-width v0.1.14 (available: v0.2.1)