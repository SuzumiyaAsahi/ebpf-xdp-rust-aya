use serde::{Deserialize, Serialize};

// package_info 数据库中的数据是以这种形式传入到前端的
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PackageInfo {
    pub source_ip: String,
    pub source_port: i64,
    pub destination_port: i64,
    pub proto_type: String,
}
