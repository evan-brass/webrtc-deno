use super::*;

use wasm_bindgen::prelude::*;
use js_sys::{Array, Uint8Array, Object, Reflect};
use wasm_bindgen::JsCast;

use deno_net::Conn;
use deno_net::DatagramConn;

#[async_trait::async_trait(?Send)]
impl ConnTrait for Conn {
	async fn connect(&self, _addr: SocketAddr) -> webrtc_util::Result<()> { unimplemented!() }
	async fn recv(&self, buf: &mut [u8]) -> webrtc_util::Result<usize> {
		// de-frame the TCP stream
		let mut len = [0u8; 2];
		let mut total = 0usize;
		while total < len.len() {
			let Ok(ret) = self.read(&mut len[total..]).await else {
				return Err(NetErr::Other("Failed while reading length".into()));
			};
			if ret.is_null() {
				return Err(NetErr::ErrBufferClosed)
			}
			let read = ret.as_f64().unwrap() as usize;
			total += read;
		}
		let len = u16::from_be_bytes(len) as usize;
		if len > buf.len() {
			return Err(NetErr::ErrBufferShort);
		}

		let mut total = 0usize;
		while total < len {
			let Ok(ret) = self.read(&mut buf[total..]).await else {
				return Err(NetErr::Other("Failed while reading the packet.".into()))
			};
			if ret.is_null() {
				break;
			}
			let read = ret.as_f64().unwrap() as usize;
			total += read;
		}
		Ok(total)
	}
	async fn recv_from(&self, buf: &mut [u8]) -> webrtc_util::Result<(usize, SocketAddr)> {
		let ret = self.recv(buf).await?;
		let sa = self.remoteAddr().await?;
		Ok((ret, sa))
	}
	async fn send(&self, buf: &[u8]) -> webrtc_util::Result<usize> {
		// Send the packet's length:
		let len = (buf.len() as u16).to_be_bytes();
		if let Err(_e) = self.write(&len).await {
			return Err(NetErr::Other("Failed to send the packet len.".into()));
		}
		if let Err(_e) = self.write(buf).await {
			return Err(NetErr::Other("Failed to write the packet data".into()));
		}
		Ok(buf.len())
	}
	async fn send_to(&self, _buf: &[u8], _target: SocketAddr) -> webrtc_util::Result<usize> { unimplemented!() }
	async fn local_addr(&self) -> webrtc_util::Result<SocketAddr> {
		let addr = self.localAddr();
		if let Ok(ip) = addr.hostname().parse::<IpAddr>() {
			Ok(SocketAddr::new(ip, addr.port()))
		} else {
			Err(webrtc_util::Error::ErrLocAddr)
		}
	}
	async fn remote_addr(&self) -> Option<SocketAddr> {
		let addr = self.remoteAddr();
		let ip: IpAddr = addr.hostname().parse().ok()?;
		Some(SocketAddr::new(ip, addr.port()))
	}
	async fn close(&self) -> webrtc_util::Result<()> {
		Conn::close(self);
		Ok(())
	}
}

#[async_trait::async_trait(?Send)]
impl ConnTrait for DatagramConn {
	async fn connect(&self, _addr: SocketAddr) -> webrtc_util::Result<()> { unimplemented!() }
	async fn recv(&self, buf: &mut [u8]) -> webrtc_util::Result<usize> {
		let (ret, _addr) = self.recv_from(buf).await?;
		Ok(ret)
	}
	async fn recv_from(&self, buf: &mut [u8]) -> webrtc_util::Result<(usize, SocketAddr)> {
		let ret = self.receive(Some(buf)).await
			.map_err(|_| webrtc_util::Error::Other("sadge".into()))?;
		let ret = ret.unchecked_into::<Array>();
		let u8a = ret.get(0).unchecked_into::<Uint8Array>();
		let addr = ret.get(1).unchecked_into::<Addr>();
		let ip = addr.hostname().parse::<IpAddr>()
			.map_err(|_| webrtc_util::Error::Other("Failed to parse hostname".into()))?;
		Ok((u8a.byte_length() as usize, SocketAddr::new(ip, addr.port())))
	}
	async fn send(&self, buf: &[u8]) -> webrtc_util::Result<usize> { unimplemented!() }
	async fn send_to(&self, buf: &[u8], target: SocketAddr) -> webrtc_util::Result<usize> {
		let addr = Object::new();
		let _ = Reflect::set(&addr, &JsValue::from_str("hostname"), &JsValue::from(target.ip().to_string()));
		let _ = Reflect::set(&addr, &JsValue::from_str("ip"), &JsValue::from(target.port()));
		let ret = DatagramConn::send(&self, buf, addr).await
			.map_err(|_| webrtc_util::Error::Other("Failed to send".into()))?;
		let ret = ret.unchecked_into_f64() as usize;
		Ok(ret)
	}
	async fn local_addr(&self) -> webrtc_util::Result<SocketAddr> {
		let addr = self.localAddr();
	}
	async fn remote_addr(&self) -> Option<SocketAddr> { unimplemented!() }
	async fn close(&self) -> webrtc_util::Result<()> {
		DatagramConn::close(self);
		Ok(())
	}
}
