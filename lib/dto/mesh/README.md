# Mesh DTO Veneer (`refractive_swan_mesh_dto`)

This crate re-exports the mesh/node control-plane DTOs defined in
`lib/domain/meta/contracts/src/mesh.rs`. Mesh runtimes (`refractive_swan_mesh_node`,
`refractive_swan_mesh_hub`, governance services) depend on this veneer so they do not
import the entire contracts crate and can evolve independently from CLI/web
surfaces.

## Contents

`refractive_swan_mesh_dto` currently re-exports:

- `MeshNodeId`, `NodeCapabilities`
- `MeshJobType`, `MeshJobDescriptor`
- `MeshJobStatus`, `MeshJobResult`
- `MeshErrorKind`, `MeshErrorCode`, `MeshError`

Future mesh DTOs should be added to `refractive_swan_contracts::mesh` first, then
exposed here to keep the veneer as a thin shim.
