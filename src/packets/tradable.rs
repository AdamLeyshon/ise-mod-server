#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct Tradable {
    #[prost(string, tag="1")]
    pub thing_def: std::string::String,
    #[prost(string, tag="5")]
    pub item_code: std::string::String,
    #[prost(int32, tag="10")]
    pub quality: i32,
    #[prost(int32, tag="15")]
    pub quantity: i32,
    #[prost(bool, tag="20")]
    pub minified: bool,
    #[prost(float, tag="25")]
    pub base_value: f32,
    #[prost(float, tag="30")]
    pub we_buy_at: f32,
    #[prost(float, tag="35")]
    pub we_sell_at: f32,
    #[prost(string, tag="40")]
    pub stuff: std::string::String,
    #[prost(float, tag="45")]
    pub weight: f32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(Serialize, Deserialize)]
pub struct ColonyTradable {
    #[prost(string, tag="1")]
    pub thing_def: std::string::String,
    #[prost(int32, tag="10")]
    pub quality: i32,
    #[prost(bool, tag="20")]
    pub minified: bool,
    #[prost(float, tag="25")]
    pub base_value: f32,
    #[prost(float, tag="30")]
    pub weight: f32,
    #[prost(string, tag="35")]
    pub stuff: std::string::String,
}
