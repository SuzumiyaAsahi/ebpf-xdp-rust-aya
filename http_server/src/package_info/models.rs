use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PackageInfo {
    pub source_ip: String,
    pub source_port: i64,
    pub destination_port: i64,
    pub proto_type: String,
}
