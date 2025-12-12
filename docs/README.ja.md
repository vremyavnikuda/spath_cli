# spath

Windows PATH セキュリティスキャナーと最適化ツール。

## 問題

スペースを含むがクォートされていないPATHエントリは、特権昇格攻撃に悪用される可能性のあるセキュリティ脆弱性を作成します。

## 解決策

spathはこれらの脆弱性を自動的に検出して修正します。

## インストール

```bash
cargo build --release
```

バイナリの場所: `target/release/spath.exe`

## コマンド

### スキャン

PATHのセキュリティ問題を分析します。

```bash
spath scan
spath scan --verbose
spath scan --audit
```

### 修正

USER PATHの問題を修正します（管理者権限不要）。

```bash
spath fix --dry-run
spath fix
spath fix --delicate
```

### 分析

SYSTEMとUSER PATHの両方を分析します。

```bash
spath analyze
```

### クリーンアップ

重複パスを削除してPATHを最適化します。

```bash
spath clean --dry-run
spath clean
spath clean --system
spath clean --delicate
```

### バックアップ管理

```bash
spath backup
spath list-backups
spath restore <バックアップファイル>
spath restore <バックアップファイル> --delicate
```

## 問題の種類

**CRITICAL**: クォートされていないスペース付きパス - セキュリティ脆弱性

**WARNING**: 存在しないパスまたは相対パス

**INFO**: 情報メッセージ

## ワークフロー

1. スキャン: `spath scan --audit`
2. 分析: `spath analyze`
3. バックアップ: `spath backup`
4. USER PATH修正: `spath fix`
5. 重複削除: `spath clean`
6. 必要に応じて復元: `spath restore <ファイル>`

## 要件

- Windows 10以降
- Rust 1.70+（ソースからビルドする場合）

## オプション

- `--dry-run` または `-d` - 変更を適用せずにプレビュー
- `--delicate` - 変更を適用する前に確認を求める
- `--system` または `-s` - SYSTEM PATH操作を含める（管理者権限必要）
- `--verbose` または `-v` - 詳細情報を表示
- `--audit` または `-a` - 詳細な監査レポートを表示

## 注意事項

- USER PATHの変更には管理者権限は不要です
- SYSTEM PATHの変更には管理者権限が必要です
- 変更前に自動バックアップが作成されます
- PATH変更を適用するにはアプリケーションを再起動してください
- 確認プロンプト付きの追加の安全性には`--delicate`を使用してください

## ライセンス

MIT License
