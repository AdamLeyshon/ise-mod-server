use semver_parser::version::{parse, Version};

use crate::packets::hello::HelloRequest;

pub trait ClientVersion {
    fn parse_version(self) -> Result<Version, ()>;
}

impl ClientVersion for HelloRequest {
    fn parse_version(self) -> Result<Version, ()> {
        parse(&*self.client_version).map_err(|_| ())
    }
}
