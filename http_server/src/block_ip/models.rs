use serde::{Deserialize, Serialize};

// CREATE TABLE blocked_ip (
//     id INTEGER PRIMARY KEY AUTOINCREMENT,
//     ipv4 TEXT NOT NULL
// );
//

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlockedIp {
    pub id: i64,
    pub ipv4: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WriteIp {
    pub ipv4: Option<String>,
}
