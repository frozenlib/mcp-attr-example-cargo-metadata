use std::path::PathBuf;
use std::sync::Mutex;

use cargo_metadata::{Metadata, MetadataCommand, Package};
use mcp_attr::server::{McpServer, mcp_server, serve_stdio};
use mcp_attr::{ErrorCode, Result, bail_public};
use serde::Serialize;

#[tokio::main]
async fn main() -> Result<()> {
    serve_stdio(CargoMetadataServer(Mutex::new(ServerData::new()))).await?;
    Ok(())
}

struct CargoMetadataServer(Mutex<ServerData>);

struct ServerData {
    metadata: Option<Metadata>,
}

impl ServerData {
    fn new() -> Self {
        Self { metadata: None }
    }

    fn get_metadata(&mut self, manifest_path: PathBuf) -> Result<&Metadata> {
        if self.metadata.is_none() {
            let mut cmd = MetadataCommand::new();
            cmd.manifest_path(manifest_path);
            match cmd.exec() {
                Ok(metadata) => self.metadata = Some(metadata),
                Err(e) => bail_public!(
                    ErrorCode::INTERNAL_ERROR,
                    "Failed to get cargo metadata: {}",
                    e
                ),
            }
        }
        Ok(self.metadata.as_ref().unwrap())
    }
}

#[derive(Serialize)]
struct PackageInfo {
    name: String,
    version: String,
    authors: Vec<String>,
    description: Option<String>,
    repository: Option<String>,
    license: Option<String>,
    dependencies: Vec<DependencyInfo>,
}

#[derive(Serialize)]
struct DependencyInfo {
    name: String,
    version: String,
    optional: bool,
    features: Vec<String>,
}

#[mcp_server]
impl McpServer for CargoMetadataServer {
    /// Cargo Metadata MCP Server
    ///
    /// このサーバーはCargoプロジェクトのメタデータ情報を提供します。
    /// プロジェクトの依存関係、パッケージ情報、ビルドターゲットなどを取得できます。
    #[prompt]
    async fn cargo_metadata_prompt(&self) -> Result<&str> {
        Ok(
            "Cargo Metadataサーバーへようこそ！このサーバーを使用して、Cargoプロジェクトのメタデータ情報を取得できます。",
        )
    }

    /// プロジェクトのメタデータを取得します
    ///
    /// 指定されたCargoプロジェクトのメタデータを取得します。
    /// manifest_pathには、Cargo.tomlファイルへの絶対パスを指定します。
    #[tool]
    async fn get_metadata(&self, manifest_path: String) -> Result<String> {
        let mut state = self.0.lock().unwrap();
        let metadata = state.get_metadata(PathBuf::from(manifest_path))?;

        match serde_json::to_string_pretty(metadata) {
            Ok(json) => Ok(json),
            Err(e) => bail_public!(
                ErrorCode::INTERNAL_ERROR,
                "Failed to serialize metadata: {}",
                e
            ),
        }
    }

    /// プロジェクトのパッケージ情報を取得します
    ///
    /// 指定されたCargoプロジェクトのパッケージ情報を取得します。
    /// manifest_pathには、Cargo.tomlファイルへの絶対パスを指定します。
    #[tool]
    async fn get_package_info(&self, manifest_path: String) -> Result<String> {
        let mut state = self.0.lock().unwrap();
        let metadata = state.get_metadata(PathBuf::from(manifest_path))?;

        let root_package = match metadata.root_package() {
            Some(pkg) => pkg,
            None => bail_public!(ErrorCode::INTERNAL_ERROR, "No root package found"),
        };

        let dependencies = get_dependencies(root_package, metadata);

        let package_info = PackageInfo {
            name: root_package.name.clone(),
            version: root_package.version.to_string(),
            authors: root_package.authors.clone(),
            description: root_package.description.clone(),
            repository: root_package.repository.clone(),
            license: root_package.license.clone(),
            dependencies,
        };

        match serde_json::to_string_pretty(&package_info) {
            Ok(json) => Ok(json),
            Err(e) => bail_public!(
                ErrorCode::INTERNAL_ERROR,
                "Failed to serialize package info: {}",
                e
            ),
        }
    }

    /// プロジェクトの依存関係リストを取得します
    ///
    /// 指定されたCargoプロジェクトの依存関係リストを取得します。
    /// manifest_pathには、Cargo.tomlファイルへの絶対パスを指定します。
    #[tool]
    async fn get_dependencies(&self, manifest_path: String) -> Result<String> {
        let mut state = self.0.lock().unwrap();
        let metadata = state.get_metadata(PathBuf::from(manifest_path))?;

        let root_package = match metadata.root_package() {
            Some(pkg) => pkg,
            None => bail_public!(ErrorCode::INTERNAL_ERROR, "No root package found"),
        };

        let dependencies = get_dependencies(root_package, metadata);

        match serde_json::to_string_pretty(&dependencies) {
            Ok(json) => Ok(json),
            Err(e) => bail_public!(
                ErrorCode::INTERNAL_ERROR,
                "Failed to serialize dependencies: {}",
                e
            ),
        }
    }

    /// プロジェクトのビルドターゲットを取得します
    ///
    /// 指定されたCargoプロジェクトのビルドターゲットを取得します。
    /// manifest_pathには、Cargo.tomlファイルへの絶対パスを指定します。
    #[tool]
    async fn get_targets(&self, manifest_path: String) -> Result<String> {
        let mut state = self.0.lock().unwrap();
        let metadata = state.get_metadata(PathBuf::from(manifest_path))?;

        let root_package = match metadata.root_package() {
            Some(pkg) => pkg,
            None => bail_public!(ErrorCode::INTERNAL_ERROR, "No root package found"),
        };

        match serde_json::to_string_pretty(&root_package.targets) {
            Ok(json) => Ok(json),
            Err(e) => bail_public!(
                ErrorCode::INTERNAL_ERROR,
                "Failed to serialize targets: {}",
                e
            ),
        }
    }

    /// プロジェクトのワークスペース情報を取得します
    ///
    /// 指定されたCargoプロジェクトのワークスペース情報を取得します。
    /// manifest_pathには、Cargo.tomlファイルへの絶対パスを指定します。
    #[tool]
    async fn get_workspace_info(&self, manifest_path: String) -> Result<String> {
        let mut state = self.0.lock().unwrap();
        let metadata = state.get_metadata(PathBuf::from(manifest_path))?;

        let workspace_members = metadata
            .workspace_members
            .iter()
            .filter_map(|id| metadata.packages.iter().find(|p| p.id == *id))
            .collect::<Vec<&Package>>();

        match serde_json::to_string_pretty(&workspace_members) {
            Ok(json) => Ok(json),
            Err(e) => bail_public!(
                ErrorCode::INTERNAL_ERROR,
                "Failed to serialize workspace members: {}",
                e
            ),
        }
    }

    /// プロジェクトのフィーチャー情報を取得します
    ///
    /// 指定されたCargoプロジェクトのフィーチャー情報を取得します。
    /// manifest_pathには、Cargo.tomlファイルへの絶対パスを指定します。
    #[tool]
    async fn get_features(&self, manifest_path: String) -> Result<String> {
        let mut state = self.0.lock().unwrap();
        let metadata = state.get_metadata(PathBuf::from(manifest_path))?;

        let root_package = match metadata.root_package() {
            Some(pkg) => pkg,
            None => bail_public!(ErrorCode::INTERNAL_ERROR, "No root package found"),
        };

        match serde_json::to_string_pretty(&root_package.features) {
            Ok(json) => Ok(json),
            Err(e) => bail_public!(
                ErrorCode::INTERNAL_ERROR,
                "Failed to serialize features: {}",
                e
            ),
        }
    }
}

fn get_dependencies(package: &Package, metadata: &Metadata) -> Vec<DependencyInfo> {
    package
        .dependencies
        .iter()
        .map(|dep| {
            let resolved_package = metadata.packages.iter().find(|p| p.name == dep.name);

            let version = resolved_package
                .map(|p| p.version.to_string())
                .unwrap_or_else(|| dep.req.to_string());

            DependencyInfo {
                name: dep.name.clone(),
                version,
                optional: dep.optional,
                features: dep.features.clone(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_metadata_with_invalid_path() {
        let mut server_data = ServerData::new();
        let result = server_data.get_metadata(PathBuf::from("non_existent_path/Cargo.toml"));

        assert!(result.is_err());
        // エラーが発生することのみを確認
    }
}
