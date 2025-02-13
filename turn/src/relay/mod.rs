pub mod relay_none;
pub mod relay_range;
pub mod relay_static;

use crate::error::Result;

use util::Conn;

use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;

// RelayAddressGenerator is used to generate a RelayAddress when creating an allocation.
// You can use one of the provided ones or provide your own.
#[async_trait(?Send)]
pub trait RelayAddressGenerator {
    // validate confirms that the RelayAddressGenerator is properly initialized
    fn validate(&self) -> Result<()>;

    // Allocate a RelayAddress
    async fn allocate_conn(
        &self,
        use_ipv4: bool,
        requested_port: u16,
    ) -> Result<(Arc<dyn Conn>, SocketAddr)>;
}
