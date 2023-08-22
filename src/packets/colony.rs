#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct ColonyData {
    #[prost(string, tag="1")]
    pub colony_id: std::string::String,
    #[prost(string, tag="2")]
    pub name: std::string::String,
    #[prost(string, tag="3")]
    pub faction_name: std::string::String,
    #[prost(int32, tag="9")]
    pub map_id: i32,
    #[prost(int32, tag="4")]
    pub tick: i32,
    /// This is a fuse, once blown, it cannot be unset.
    #[prost(bool, tag="5")]
    pub used_dev_mode: bool,
    #[prost(string, tag="6")]
    pub game_version: std::string::String,
    #[prost(enumeration="PlatformEnum", tag="7")]
    pub platform: i32,
    #[prost(int64, tag="8")]
    pub create_date: i64,
    /// The world seed
    #[prost(string, tag="10")]
    pub seed: std::string::String,
    ///X,Y of colony on map
    #[prost(string, tag="11")]
    pub location: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct ColonyCreateRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    /// Certain fields from ColonyData will be ignored
    /// during creation of the colony such as ColonyId and CreateDate.
    #[prost(message, optional, tag="2")]
    pub data: ::std::option::Option<ColonyData>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct ColonyUpdateRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    /// Certain fields from ColonyData will be ignored
    /// during update of the colony such as ColonyId, Platform and CreateDate.
    #[prost(message, optional, tag="3")]
    pub data: ::std::option::Option<ColonyData>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct ColonyGetRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    #[prost(string, tag="2")]
    pub colony_id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct ColonyModsSetRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    #[prost(string, tag="2")]
    pub colony_id: std::string::String,
    #[prost(string, repeated, tag="3")]
    pub mod_name: ::std::vec::Vec<std::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct ColonyModsSetReply {
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct ColonyTradableSetRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    #[prost(string, tag="2")]
    pub colony_id: std::string::String,
    #[prost(message, repeated, tag="3")]
    pub item: ::std::vec::Vec<super::tradable::ColonyTradable>,
    #[prost(bool, tag="10")]
    pub final_packet: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct ColonyTradableSetReply {
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(Serialize, Deserialize)]
#[derive(TryFromPrimitive)]
pub enum PlatformEnum {
    Unknown = 0,
    Windows = 1,
    Linux = 2,
    Mac = 3,
}
