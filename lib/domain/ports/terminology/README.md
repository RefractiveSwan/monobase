# refractive_swan_terminology_port

**Path:** `code/lib/domain/ports/terminology`

Port crate that defines the terminology lookup trait, config, and shared error types.
Domain crates (mapping, ingestion) depend on these enums/traits, while platform/app
crates implement the trait via concrete HTTP/mock clients.
