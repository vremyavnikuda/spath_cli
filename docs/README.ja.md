# spath

![spath banner](../img/1765746905745-019b1eb7-0a53-7269-9766-481e69cf3b4e.png)

Windows PATH セキュリティスキャナーと最適化ツール。

## 問題

スペースを含むがクォートされていないPATHエントリは、特権昇格攻撃に悪用される可能性のあるセキュリティ脆弱性を作成します。

## 解決策

spathはこれらの脆弱性を自動的に検出して修正します。

## インストール

### ダウンロード（最も簡単）

[GitHub Releases](https://github.com/vremyavnikuda/spath_cli/releases)からダウンロード:

- `spath-setup.exe` — インストーラー（PATHに自動追加）
- `spath.exe` — スタンドアロン実行ファイル
- `spath-windows-x64.zip` — ドキュメント付きアーカイブ

### crates.ioから

```bash
cargo install spath-cli
```

### ソースから

```bash
cargo build --release
```

バイナリの場所: `target/release/spath.exe`

## コマンド

### スキャン

PATHのセキュリティ問題を分析します。

```bash
spath scan                    # USER PATHのみをスキャン
spath scan --verbose          # 詳細情報を表示
spath scan --audit            # 監査統計を表示
spath scan --system           # SYSTEM PATHをスキャン（修正には管理者権限が必要）
```

### セキュリティ検証

悪意のあるファイルの存在を確認することで、重大な問題が実際に悪用可能かどうかをチェックします。

```bash
spath verify                  # USER PATHのセキュリティを検証
spath verify --system         # SYSTEM PATHのセキュリティを検証
```

このコマンドは、`C:\Program.exe`のような悪意のあるファイルを探すことで、クォートされていないスペース付きパスが実際に悪用される可能性があるかどうかをチェックします。

### 修正

USER PATHの問題を修正します（管理者権限不要）。

```bash
spath fix --dry-run           # 変更を適用せずにプレビュー
spath fix                     # USER PATHに修正を適用
spath fix --delicate          # 変更前に確認を求める
```

### 分析

SYSTEMとUSER PATHの両方を分析します。

```bash
spath analyze
```

### クリーンアップ

重複パスを削除してPATHを最適化します。

```bash
spath clean --dry-run         # クリーンアップをプレビュー
spath clean                   # USER PATHをクリーンアップ
spath clean --system          # SYSTEM PATHをクリーンアップ（管理者権限必要）
spath clean --delicate        # 確認を求める
```

### バックアップ管理

```bash
spath backup                  # 現在のPATHのバックアップを作成
spath list-backups            # 利用可能なすべてのバックアップを一覧表示
spath restore <ファイル>      # バックアップから復元
spath restore <ファイル> --delicate  # 確認付きで復元
```

## 問題の種類

**CRITICAL**: システムディレクトリ（例：`C:\Program Files`）内のクォートされていないスペース付きパス - 悪用される可能性のある潜在的なセキュリティ脆弱性

**WARNING**: 存在しないパス、相対パス、または存在しないクォートされていないスペース付きパス

**INFO**: 適切にクォートされたパスまたは軽微な問題に関する情報メッセージ

## セキュリティ検証

`verify`コマンドは以下を区別します：
- **潜在的リスク**: 脆弱なパスだが悪用ファイルは検出されていない（現時点では安全）
- **実際の脅威**: 脆弱性を悪用する可能性のある悪意のあるファイルが見つかった（即座の対応が必要）

例：`C:\Program Files\App\bin`がクォートなしでPATHにある場合、ツールは以下をチェックします：
- `C:\Program.exe`
- `C:\Program.com`
- `C:\Program.bat`
- `C:\Program.cmd`

## ワークフロー

### 基本ワークフロー
1. スキャン: `spath scan --audit`
2. 検証: `spath verify`（実際の脅威をチェック）
3. バックアップ: `spath backup`
4. USER PATH修正: `spath fix`
5. 重複削除: `spath clean`
6. 必要に応じて復元: `spath restore <ファイル>`

### 高度なワークフロー（SYSTEM PATHを含む）
1. SYSTEMスキャン: `spath scan --system`
2. SYSTEM検証: `spath verify --system`（エクスプロイトをチェック）
3. 安全であれば、SYSTEM PATHの修正を検討（管理者権限が必要）

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

MIT License - [LICENSE](../LICENSE)ファイルを参照

## 変更履歴

バージョン履歴とリリースノートについては、[CHANGELOG.md](../CHANGELOG.md)を参照してください。
