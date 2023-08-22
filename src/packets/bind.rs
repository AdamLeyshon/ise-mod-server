#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct BindRequest {
    #[prost(string, tag="1")]
    pub steam_id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct BindReply {
    #[prost(bool, tag="1")]
    pub valid: bool,
    #[prost(string, tag="2")]
    pub bind_id: std::string::String,
    #[prost(enumeration="bind_reply::BindErrorReason", tag="3")]
    pub reason: i32,
}
pub mod bind_reply {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    #[derive(Serialize, Deserialize)]
    pub enum BindErrorReason {
        None = 0,
        Unknown = 1,
        SteamIdInvalid = 2,
        SteamIdBlocked = 3,
    }
}
/// This message is used for two purposes
/// 1. Checking if a bind request is valid
/// 2. Acknowledging a FinalBindId
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct ConfirmBindRequest {
    #[prost(enumeration="BindTypeEnum", tag="1")]
    pub bind_type: i32,
    #[prost(string, tag="2")]
    pub bind_id: std::string::String,
}
/// This message is used for two purposes
/// 1. Answering a bind request with status data.
/// 2. Acknowledging a Client Bind ID
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct ConfirmBindReply {
    #[prost(enumeration="BindTypeEnum", tag="1")]
    pub bind_type: i32,
    /// Lets them know if this Bind Id is still valid.
    #[prost(bool, tag="2")]
    pub is_valid: bool,
    /// Has the Id been accepted?
    #[prost(bool, tag="3")]
    pub bind_complete: bool,
    /// If the user has confirmed the Id, this is set and is the permenant
    /// Id associated with that client.
    /// It must be stored by the client, it is required to authenticate later.
    #[prost(string, tag="4")]
    pub client_bind_id: std::string::String,
    /// Time left until this Bind Id is invalid.
    /// When this reaches 0, ValidBind becomes False. 
    /// Will never be less than 0.
    #[prost(int32, tag="5")]
    pub ttl: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct ClientBindVerifyRequest {
    #[prost(string, tag="1")]
    pub client_bind_id: std::string::String,
    #[prost(string, tag="2")]
    pub steam_id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct ClientBindVerifyReply {
    #[prost(bool, tag="1")]
    pub valid: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(Serialize, Deserialize)]
#[derive(TryFromPrimitive)]
pub enum BindTypeEnum {
    AccountBind = 0,
    ClientBind = 1,
}
