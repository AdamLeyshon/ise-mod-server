#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct BankDataReply {
    #[prost(map="int32, int32", tag="1")]
    pub balance: ::std::collections::HashMap<i32, i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct BankGetRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    #[prost(string, tag="2")]
    pub colony_id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct BankWithdrawRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    #[prost(string, tag="2")]
    pub colony_id: std::string::String,
    #[prost(enumeration="super::common::CurrencyEnum", tag="3")]
    pub currency: i32,
    #[prost(int32, tag="4")]
    pub amount: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct BankWithdrawReply {
    #[prost(message, optional, tag="1")]
    pub data: ::std::option::Option<super::order::OrderStatusReply>,
    #[prost(enumeration="super::order::OrderRequestStatus", tag="2")]
    pub status: i32,
    #[prost(int32, tag="5")]
    pub balance: i32,
}
