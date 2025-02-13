use crate::auth::*;
use crate::error::*;
use crate::relay::*;

use util::Conn;

use std::sync::Arc;
use std::time::Duration;

// ConnConfig is used for UDP listeners
pub struct ConnConfig {
    pub conn: Arc<dyn Conn>,

    // When an allocation is generated the RelayAddressGenerator
    // creates the net.PacketConn and returns the IP/Port it is available at
    pub relay_addr_generator: Box<dyn RelayAddressGenerator>,
}

impl ConnConfig {
    pub fn validate(&self) -> Result<()> {
        self.relay_addr_generator.validate()
    }
}

// ServerConfig configures the Pion TURN Server
pub struct ServerConfig {
    // conn_configs are a list of all the turn listeners
    // Each listener can have custom behavior around the creation of Relays
    pub conn_configs: Vec<ConnConfig>,

    // realm sets the realm for this server
    pub realm: String,

    // auth_handler is a callback used to handle incoming auth requests, allowing users to customize Pion TURN with custom behavior
    pub auth_handler: Arc<dyn AuthHandler>,

    // channel_bind_timeout sets the lifetime of channel binding. Defaults to 10 minutes.
    pub channel_bind_timeout: Duration,
}

impl ServerConfig {
    pub fn validate(&self) -> Result<()> {
        if self.conn_configs.is_empty() {
            return Err(Error::ErrNoAvailableConns);
        }

        for cc in &self.conn_configs {
            cc.validate()?;
        }
        Ok(())
    }
}
