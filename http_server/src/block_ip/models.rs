use serde::{Deserialize, Serialize};

// 传给前端的数据应该是以这种形式返回的
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlockedIp {
    pub id: i64,
    pub ipv4: String,
}

// 前端传给后端的数据应该是这样发送的
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WriteIp {
    pub ipv4: Option<String>,
}
