// Code generated by jtd-codegen for Rust v0.2.1

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ApiConfigDataApi {
    #[serde(rename = "force_offline")]
    pub force_offline: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiConfigDataDelivery {
    #[serde(rename = "collect_cost_per_kg")]
    pub collect_cost_per_kg: u32,

    #[serde(rename = "delivery_cost_per_kg")]
    pub delivery_cost_per_kg: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ApiConfigDataInventory {
    #[serde(rename = "vote_age_threshold")]
    pub vote_age_threshold: u32,

    #[serde(rename = "vote_promotion_threshold")]
    pub vote_promotion_threshold: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ApiConfigDataMaintenance {
    #[serde(rename = "start_time")]
    pub start_time: u32,
}

#[derive(Serialize, Deserialize, FromSqlRow, AsExpression, Debug, Clone, Default)]
pub struct ApiConfigData {
    #[serde(rename = "api")]
    pub api: ApiConfigDataApi,

    #[serde(rename = "delivery")]
    pub delivery: ApiConfigDataDelivery,

    #[serde(rename = "inventory")]
    pub inventory: ApiConfigDataInventory,

    #[serde(rename = "maintenance")]
    pub maintenance: ApiConfigDataMaintenance,
}
