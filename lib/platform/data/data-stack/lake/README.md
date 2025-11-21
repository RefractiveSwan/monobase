# dfps_datalake

**Conceptual location:** `lib/platform/data/lake`  
**Current physical location:** Not yet implemented  
**Scope:** Parquet/Delta lake snapshots for node exports and hub ingestion

This directory will host the **data lake abstraction** for exporting node-local warehouse snapshots and enabling hub ingestion with differential privacy compliance.

---

## Purpose

`dfps_datalake` provides:

1. **LakeConfig**: Path, file format, retention, partition columns
2. **LakeWriter**: Export warehouse snapshots (Parquet/Delta)
3. **LakeReader**: Read partitioned datasets for hub ingestion
4. **DP integration**: Apply DP noise before export (via `dfps_compliance`)

**Goal**: Enable secure node → hub data sharing while preserving privacy.

---

## Trait Design

### LakeConfig

```rust
/// Configuration for data lake storage.
#[derive(Clone, Debug)]
pub struct LakeConfig {
    /// Storage path (local filesystem or cloud URI)
    pub path: String,
    
    /// File format
    pub format: LakeFormat,
    
    /// Partition columns (e.g., ["year", "month", "day"])
    pub partition_by: Vec<String>,
    
    /// Retention policy (days)
    pub retention_days: Option<u32>,
    
    /// Compression codec
    pub compression: CompressionCodec,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LakeFormat {
    Parquet,
    Delta,
    CSV,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompressionCodec {
    None,
    Snappy,
    Gzip,
    Zstd,
}
```

### LakeWriter Trait

```rust
/// Trait for writing warehouse snapshots to the lake.
#[async_trait]
pub trait LakeWriter: Send + Sync {
    /// Write a snapshot of the warehouse to the lake.
    async fn write_snapshot(
        &self,
        snapshot: &WarehouseSnapshot,
        metadata: &SnapshotMetadata,
    ) -> Result<WriteSummary, LakeError>;
    
    /// Write a single partition.
    async fn write_partition(
        &self,
        partition: &Partition,
        data: &[u8],
    ) -> Result<(), LakeError>;
}

#[derive(Clone, Debug)]
pub struct WarehouseSnapshot {
    pub fact_rows: Vec<serde_json::Value>,
    pub dim_rows: HashMap<String, Vec<serde_json::Value>>,
}

#[derive(Clone, Debug)]
pub struct SnapshotMetadata {
    pub node_id: MeshNodeId,
    pub snapshot_timestamp: chrono::DateTime<chrono::Utc>,
    pub row_count: u64,
    pub dp_applied: bool,
    pub epsilon: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct WriteSummary {
    pub bytes_written: u64,
    pub partitions_created: u32,
    pub duration_ms: u64,
}
```

### LakeReader Trait

```rust
/// Trait for reading lake datasets.
#[async_trait]
pub trait LakeReader: Send + Sync {
    /// List available snapshots (for hub ingestion).
    async fn list_snapshots(&self, filters: &SnapshotFilters) -> Result<Vec<SnapshotMetadata>, LakeError>;
    
    /// Read a specific snapshot.
    async fn read_snapshot(&self, metadata: &SnapshotMetadata) -> Result<WarehouseSnapshot, LakeError>;
    
    /// Read a single partition.
    async fn read_partition(&self, partition: &Partition) -> Result<Vec<u8>, LakeError>;
}

#[derive(Clone, Debug, Default)]
pub struct SnapshotFilters {
    pub node_ids: Option<Vec<MeshNodeId>>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub dp_required: bool,
}
```

### LakeError

```rust
#[derive(Debug, Error)]
pub enum LakeError {
    #[error("lake disabled")]
    Disabled,
    
    #[error("write failed: {source}")]
    WriteFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("read failed: {source}")]
    ReadFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("DP noise not applied")]
    DPNotApplied,
}
```

---

## Use Cases

### Node Snapshot Export

Nodes periodically export warehouse snapshots with DP noise:

```rust
// In dfps_mesh_node
pub async fn export_daily_snapshot(
    &self,
    lake_writer: &dyn LakeWriter,
    policy: &dfps_compliance::Policy,
) -> Result<WriteSummary, NodeError> {
    // Query warehouse for today's facts
    let fact_rows = self.datamart.query_fact_range(today()).await?;
    
    // Apply DP noise (Laplace mechanism)
    let noised_rows = policy.apply_dp_noise(&fact_rows, epsilon: 1.0)?;
    
    let snapshot = WarehouseSnapshot {
        fact_rows: noised_rows,
        dim_rows: HashMap::new(), // Dims don't need DP
    };
    
    let metadata = SnapshotMetadata {
        node_id: self.node_id.clone(),
        snapshot_timestamp: chrono::Utc::now(),
        row_count: snapshot.fact_rows.len() as u64,
        dp_applied: true,
        epsilon: Some(1.0),
    };
    
    lake_writer.write_snapshot(&snapshot, &metadata).await
}
```

### Hub Ingestion

Hub reads node snapshots and ingests into reporting warehouse:

```rust
// In dfps_mesh_hub
pub async fn ingest_node_snapshots(
    &self,
    lake_reader: &dyn LakeReader,
    governance: &dfps_mesh_governance::GovernancePolicy,
) -> Result<IngestSummary, HubError> {
    let filters = SnapshotFilters {
        node_ids: None, // All nodes
        start_date: Some(yesterday()),
        end_date: Some(today()),
        dp_required: true, // Only ingest DP-noised data
    };
    
    let snapshots = lake_reader.list_snapshots(&filters).await?;
    
    for metadata in snapshots {
        // Check governance policy
        if !governance.allow_ingest(&metadata)? {
            continue; // Skip denied snapshots
        }
        
        let snapshot = lake_reader.read_snapshot(&metadata).await?;
        
        // Load into hub reporting warehouse
        self.reporting_warehouse.load_batch(&snapshot.fact_rows).await?;
    }
    
    Ok(IngestSummary { snapshots_ingested: snapshots.len() })
}
```

---

## File Organization

### Parquet Layout

```
data_lake/
  snapshots/
    node_id=node-a/
      year=2025/
        month=11/
          day=21/
            fact_service_request_part_0.parquet
            fact_service_request_part_1.parquet
    node_id=node-b/
      year=2025/
        month=11/
          day=21/
            fact_service_request_part_0.parquet
```

**Partitioning strategy**:
- `node_id`: Enables per-node filtering
- `year/month/day`: Time-based retention and queries

### Delta Lake Layout

Delta Lake adds transaction log and ACID guarantees:

```
data_lake/
  delta_table/
    _delta_log/
      00000000000000000000.json
      00000000000000000001.json
    part-00000-*.parquet
    part-00001-*.parquet
```

---

## Differential Privacy Integration

Before writing snapshots, nodes apply DP noise:

```rust
use dfps_compliance::Policy;

pub async fn write_dp_snapshot(
    &self,
    warehouse: &dyn WarehouseAnalytics,
    lake_writer: &dyn LakeWriter,
    policy: &Policy,
) -> Result<WriteSummary, LakeError> {
    // Raw warehouse query
    let raw_rows = warehouse.query_fact_range(date_range).await?;
    
    // Apply DP (Laplace mechanism)
    let noised_rows = policy.apply_dp_noise(&raw_rows, epsilon: 1.0)?;
    
    // Verify DP was applied
    if !noised_rows.iter().all(|r| r.dp_applied) {
        return Err(LakeError::DPNotApplied);
    }
    
    let snapshot = WarehouseSnapshot {
        fact_rows: noised_rows,
        dim_rows: HashMap::new(),
    };
    
    let metadata = SnapshotMetadata {
        node_id: self.node_id.clone(),
        snapshot_timestamp: chrono::Utc::now(),
        row_count: snapshot.fact_rows.len() as u64,
        dp_applied: true,
        epsilon: Some(1.0),
    };
    
    lake_writer.write_snapshot(&snapshot, &metadata).await
}
```

---

## Migration Plan (MESH-025)

### Phase 1: Design Only (current)

- This README documents the planned abstraction
- No implementation yet

### Phase 2: Implement Parquet Writer/Reader

- Create `lib/platform/data/lake/src/parquet.rs`
- Use `arrow` and `parquet` crates for I/O
- Implement `LakeWriter` and `LakeReader` traits

### Phase 3: DP Integration

- Wire `dfps_compliance` for noise application
- Add governance checks before hub ingestion

### Phase 4: Delta Lake Support

- Add Delta Lake backend with transaction log
- Support time-travel queries for compliance audits

---

## Testing

### Unit Tests

- `LakeConfig` validation
- Partition path generation

### Integration Tests

- Write Parquet snapshot to temp directory
- Read snapshot and verify roundtrip
- DP noise verification

### End-to-End Tests

- Node exports snapshot → hub ingests → reporting warehouse updated
- Governance denies non-DP snapshots

---

## References

- **Warehouse**: `lib/platform/data/warehouse/README.md`
- **Compliance**: `lib/platform/compliance` (existing)
- **Mesh Governance**: `lib/platform/mesh/governance/README.md` (to be created)
- **Mesh Layout**: `docs/system-design/base/mesh-data-plane-layout.md`
