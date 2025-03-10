# Cargo Metadata MCP Server

このプロジェクトは、[Model Context Protocol (MCP)](https://modelcontextprotocol.io/)を使用して、Cargo プロジェクトのメタデータ情報を提供するサーバーを実装しています。

## 機能

この MCP サーバーは以下の機能を提供します：

- プロジェクトのメタデータ情報の取得
- パッケージ情報の取得
- 依存関係リストの取得
- ビルドターゲットの取得
- ワークスペース情報の取得
- フィーチャー情報の取得

## 使い方

### ビルド

```bash
cargo build --release
```

### 実行

```bash
cargo run
```

または、ビルド済みのバイナリを直接実行することもできます：

```bash
./target/release/mcp-attr-example-cargo-metadata
```

### MCP クライアントとの連携

このサーバーを MCP クライアント（例：Claude Desktop）と連携するには、クライアントの設定ファイルに以下のように追加します：

```json
{
  "mcpServers": {
    "cargo-metadata": {
      "command": "path/to/mcp-attr-example-cargo-metadata"
    }
  }
}
```

## 提供されるツール

このサーバーは以下のツールを提供します：

1. `get_metadata` - プロジェクトのメタデータを取得します
2. `get_package_info` - プロジェクトのパッケージ情報を取得します
3. `get_dependencies` - プロジェクトの依存関係リストを取得します
4. `get_targets` - プロジェクトのビルドターゲットを取得します
5. `get_workspace_info` - プロジェクトのワークスペース情報を取得します
6. `get_features` - プロジェクトのフィーチャー情報を取得します

各ツールは、オプションで `manifest_path` パラメータを受け取ります。指定しない場合は、カレントディレクトリの Cargo.toml ファイルが使用されます。

## 技術的な詳細

このプロジェクトは以下の技術を使用しています：

- [mcp-attr](https://crates.io/crates/mcp-attr) - MCP サーバーを宣言的に記述するための Rust クレート
- [cargo_metadata](https://crates.io/crates/cargo_metadata) - Cargo プロジェクトのメタデータを取得するための Rust クレート
- [tokio](https://crates.io/crates/tokio) - 非同期ランタイム
- [serde](https://crates.io/crates/serde) - シリアライズ/デシリアライズフレームワーク

## ライセンス

MIT または Apache-2.0
