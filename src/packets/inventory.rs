//message MarketConfigRequest {
//}
//
//message MarketConfigReply {
//  // Fixed amount per KG of item to be sold
//  int32 CollectionChargePerKG = 1;
//
//  // Fixed amount per KG of item to be bought
//  int32 DeliveryChargePerKG = 2;
//}
//

#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct InventoryRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    #[prost(string, tag="2")]
    pub colony_id: std::string::String,
    #[prost(bool, tag="3")]
    pub continue_existing_promise: bool,
}
//
//message InventoryBatchRequest {
//  string ClientBindId = 1;
//  string ColonyId = 2;
//  repeated tradable.ColonyTradable items = 3;
//}
//
//message InventoryBatchReply {
//  // The items we have to offer
//  repeated tradable.Tradable Items = 1;
//}

#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct InventoryReply {
    /// The items we have to offer
    #[prost(message, repeated, tag="1")]
    pub items: ::std::vec::Vec<super::tradable::Tradable>,
    /// ID Code of promise, only valid until TTL expires
    /// You must sent this in the order request
    /// If the promise expires before your order is placed
    /// It will be refused and you will have to get the inventory again
    #[prost(string, tag="2")]
    pub inventory_promise_id: std::string::String,
    /// UTC Epoch timestamp of when this offer expires
    #[prost(int64, tag="3")]
    pub inventory_promise_expires: i64,
    /// Fixed amount per KG of item to be sold
    #[prost(int32, tag="4")]
    pub collection_charge_per_kg: i32,
    /// Fixed amount per KG of item to be bought
    #[prost(int32, tag="5")]
    pub delivery_charge_per_kg: i32,
    /// Amount of cash in their account already.
    #[prost(int32, tag="6")]
    pub account_balance: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct ActivatePromiseRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    #[prost(string, tag="2")]
    pub colony_id: std::string::String,
    #[prost(string, tag="3")]
    pub inventory_promise_id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct ActivatePromiseReply {
    #[prost(bool, tag="1")]
    pub success: bool,
    #[prost(string, tag="2")]
    pub inventory_promise_id: std::string::String,
    #[prost(int64, tag="3")]
    pub inventory_promise_expires: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct GeneratePromiseRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    #[prost(string, tag="2")]
    pub colony_id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct GeneratePromiseReply {
    #[prost(string, tag="1")]
    pub inventory_promise_id: std::string::String,
    #[prost(int64, tag="2")]
    pub inventory_promise_expires: i64,
}
