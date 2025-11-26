use std::env;

/// Controls whether mapping requests should call the vector backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorMode {
    Enabled,
    Disabled,
}

impl VectorMode {
    pub fn from_env() -> Self {
        let enabled = env::var("refractive_swan_VECTOR_ENABLED")
            .map(|value| value.eq_ignore_ascii_case("true"))
            .unwrap_or(true);
        if enabled {
            VectorMode::Enabled
        } else {
            VectorMode::Disabled
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            VectorMode::Enabled => "enabled",
            VectorMode::Disabled => "disabled",
        }
    }

    /// Returns the query value sent to `/api/map-bundles`.
    pub fn query_param(&self) -> Option<&'static str> {
        match self {
            VectorMode::Enabled => None,
            VectorMode::Disabled => Some("disabled"),
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            VectorMode::Enabled => VectorMode::Disabled,
            VectorMode::Disabled => VectorMode::Enabled,
        }
    }
}

impl Default for VectorMode {
    fn default() -> Self {
        VectorMode::from_env()
    }
}
