use super::*;
use crate::error::*;

use async_trait::async_trait;
use util::vnet::net::*;

// RelayAddressGeneratorNone returns the listener with no modifications
pub struct RelayAddressGeneratorNone {
    // Address is passed to Listen/ListenPacket when creating the Relay
    pub address: String,
    pub net: Arc<Net>,
}

#[async_trait(?Send)]
impl RelayAddressGenerator for RelayAddressGeneratorNone {
    // validate confirms that the RelayAddressGenerator is properly initialized
    fn validate(&self) -> Result<()> {
        if self.address.is_empty() {
            Err(Error::ErrListeningAddressInvalid)
        } else {
            Ok(())
        }
    }

    // Allocate a PacketConn (UDP) RelayAddress
    async fn allocate_conn(
        &self,
        use_ipv4: bool,
        requested_port: u16,
    ) -> Result<(Arc<dyn Conn>, SocketAddr)> {
        let addr = self
            .net
            .resolve_addr(use_ipv4, &format!("{}:{}", self.address, requested_port))
            .await?;
        let conn = self.net.bind(addr).await?;
        let relay_addr = conn.local_addr()?;
        Ok((conn, relay_addr))
    }
}
