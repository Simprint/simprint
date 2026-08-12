use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaintenanceType {
    #[serde(rename = "scheduled")]
    Scheduled,
    #[serde(rename = "emergency")]
    Emergency,
    #[serde(rename = "upgrade")]
    Upgrade,
}

impl FromStr for MaintenanceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "scheduled" => Ok(MaintenanceType::Scheduled),
            "emergency" => Ok(MaintenanceType::Emergency),
            "upgrade" => Ok(MaintenanceType::Upgrade),
            _ => Err(format!("无效的维护类型: {}", s)),
        }
    }
}

impl From<MaintenanceType> for String {
    fn from(maintenance_type: MaintenanceType) -> Self {
        match maintenance_type {
            MaintenanceType::Scheduled => "scheduled".to_string(),
            MaintenanceType::Emergency => "emergency".to_string(),
            MaintenanceType::Upgrade => "upgrade".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Maintenance {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub maintenance_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
