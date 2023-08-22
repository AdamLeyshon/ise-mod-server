/// The request message containing the user's name.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct HelloRequest {
    #[prost(string, tag="1")]
    pub client_version: std::string::String,
}
/// The response message containing the server version
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct HelloReply {
    #[prost(string, tag="1")]
    pub server_version: std::string::String,
}
