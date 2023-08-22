use prost_build::Config;

fn main() {
    let mut config = Config::new();
    config.out_dir("./src/packets");
    config.type_attribute(".", "#[derive(Serialize, Deserialize)]");
    config.type_attribute("BindTypeEnum", "#[derive(TryFromPrimitive)]");
    config.type_attribute("PlatformEnum", "#[derive(TryFromPrimitive)]");
    config.type_attribute("OrderStatusEnum", "#[derive(TryFromPrimitive)]");
    config.type_attribute("CurrencyEnum", "#[derive(TryFromPrimitive, EnumIter)]");
    config
        .compile_protos(&["./proto/common.proto"], &["./proto"])
        .unwrap();
    config
        .compile_protos(&["./proto/hello.proto"], &["./proto"])
        .unwrap();
    config
        .compile_protos(&["./proto/bind.proto"], &["./proto"])
        .unwrap();
    config
        .compile_protos(&["./proto/colony.proto"], &["./proto"])
        .unwrap();
    config
        .compile_protos(&["./proto/bank.proto"], &["./proto"])
        .unwrap();
    config
        .compile_protos(&["./proto/tradable.proto"], &["./proto"])
        .unwrap();
    config
        .compile_protos(&["./proto/inventory.proto"], &["./proto"])
        .unwrap();
    config
        .compile_protos(&["./proto/order.proto"], &["./proto"])
        .unwrap();
}
