#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct OrderItem {
    #[prost(string, tag="1")]
    pub item_code: std::string::String,
    #[prost(int32, tag="2")]
    pub quantity: i32,
    /// Value between 0-100 representing the
    /// Remaining percentage hitpoints of this item
    #[prost(float, tag="3")]
    pub health: f32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct DeliveryItem {
    #[prost(string, tag="1")]
    pub item_code: std::string::String,
    #[prost(string, tag="2")]
    pub thing_def: std::string::String,
    #[prost(int32, tag="3")]
    pub quantity: i32,
    #[prost(int32, tag="4")]
    pub quality: i32,
    #[prost(string, tag="5")]
    pub stuff: std::string::String,
    #[prost(bool, tag="6")]
    pub minified: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct OrderRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    #[prost(string, tag="2")]
    pub colony_id: std::string::String,
    #[prost(int32, tag="3")]
    pub colony_tick: i32,
    #[prost(message, repeated, tag="4")]
    pub want_to_sell: ::std::vec::Vec<OrderItem>,
    #[prost(message, repeated, tag="5")]
    pub want_to_buy: ::std::vec::Vec<OrderItem>,
    #[prost(string, tag="6")]
    pub inventory_promise_id: std::string::String,
    #[prost(enumeration="super::common::CurrencyEnum", tag="7")]
    pub currency: i32,
    #[prost(int32, tag="8")]
    pub additional_funds: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct OrderStatusRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    #[prost(string, tag="2")]
    pub colony_id: std::string::String,
    #[prost(string, tag="3")]
    pub order_id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct OrderManifestRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    #[prost(string, tag="2")]
    pub colony_id: std::string::String,
    #[prost(string, tag="3")]
    pub order_id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct OrderListRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    #[prost(string, tag="2")]
    pub colony_id: std::string::String,
    #[prost(bool, tag="3")]
    pub any: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct OrderUpdateRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    #[prost(string, tag="2")]
    pub colony_id: std::string::String,
    #[prost(string, tag="3")]
    pub order_id: std::string::String,
    #[prost(enumeration="OrderStatusEnum", tag="4")]
    pub status: i32,
    #[prost(int32, tag="5")]
    pub colony_tick: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct OrderStatusReply {
    #[prost(string, tag="1")]
    pub order_id: std::string::String,
    #[prost(enumeration="OrderStatusEnum", tag="2")]
    pub status: i32,
    #[prost(int32, tag="3")]
    pub delivery_tick: i32,
    #[prost(int32, tag="4")]
    pub placed_tick: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct OrderReply {
    #[prost(message, optional, tag="1")]
    pub data: ::std::option::Option<OrderStatusReply>,
    #[prost(enumeration="OrderRequestStatus", tag="2")]
    pub status: i32,
    #[prost(message, repeated, tag="3")]
    pub unavailable: ::std::vec::Vec<OrderItem>,
    #[prost(int32, tag="4")]
    pub refunded: i32,
    #[prost(int32, tag="5")]
    pub balance: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct OrderListReply {
    #[prost(message, repeated, tag="1")]
    pub orders: ::std::vec::Vec<OrderStatusReply>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct OrderManifestReply {
    #[prost(message, repeated, tag="1")]
    pub items: ::std::vec::Vec<DeliveryItem>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(Serialize, Deserialize)]
#[derive(TryFromPrimitive)]
pub enum OrderStatusEnum {
    Placed = 0,
    OutForDelivery = 1,
    Delivered = 2,
    Failed = 3,
    Reversed = 4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(Serialize, Deserialize)]
pub enum OrderRequestStatus {
    Rejected = 0,
    AcceptedAll = 1,
    AcceptedPartial = 2,
}
