use refractive_swan_datamart::{SqliteDatamart, WarehouseConfig};
use std::fs::File;
use tempfile::TempDir;

/// Helper for provisioning a temporary SQLite warehouse for integration tests.
#[derive(Debug)]
pub struct TempSqliteWarehouse {
    _dir: TempDir,
    config: WarehouseConfig,
}

impl TempSqliteWarehouse {
    /// Create a new temporary warehouse (file-backed SQLite URL).
    pub fn new() -> Self {
        let dir = TempDir::new().expect("temporary datamart dir");
        let db_path = dir.path().join("warehouse.sqlite");
        File::create(&db_path).expect("create sqlite placeholder");
        let url = format!("sqlite://{}", db_path.display());
        let config = WarehouseConfig {
            url,
            schema: None,
            max_connections: 5,
        };
        Self { _dir: dir, config }
    }

    /// Return a clone of the underlying configuration.
    pub fn config(&self) -> WarehouseConfig {
        self.config.clone()
    }

    /// Construct a datamart sink pointed at this temporary database.
    pub fn datamart(&self) -> SqliteDatamart {
        SqliteDatamart::from_config(self.config())
    }

    /// Convenience accessor for the SQLite URL.
    pub fn url(&self) -> &str {
        &self.config.url
    }
}

#[cfg(test)]
mod tests {
    use super::TempSqliteWarehouse;

    #[test]
    fn temp_warehouse_exposes_sqlite_url() {
        let warehouse = TempSqliteWarehouse::new();
        assert!(warehouse.url().starts_with("sqlite://"));
        let config = warehouse.config();
        assert_eq!(config.url, warehouse.url());
    }
}
