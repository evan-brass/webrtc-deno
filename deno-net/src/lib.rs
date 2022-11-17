use std::future::Future;
use std::time::Duration;
use std::net::{
	SocketAddr,
	SocketAddrV4,
	SocketAddrV6,
	IpAddr,
	Ipv4Addr,
	Ipv6Addr,
};

use wasm_bindgen::prelude::*;
use js_sys::{Function, Promise, Array, Object, Reflect};
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime(u64);
impl SystemTime {
	const UNIX_EPOCH: Self = Self(0);
	pub fn new() -> Self {
		Self(js_sys::Date::now() as u64)
	}
}
impl std::ops::Add<Duration> for SystemTime {
	fn add(self, rhs: Duration) -> Self::Output {
		SystemTime(self.0 + rhs.as_millis())
	}
}
impl std::ops::AddAssign<Duration> for SystemTime {
	fn add_assign(&mut self, rhs: Duration) {
		self.0 += rhs.as_millis();
	}
}

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(catch, js_namespace = Deno)]
	async fn resolveDns(query: &str, record_type: &str) -> Result<JsValue, JsValue>;

	pub type Listener;
	#[wasm_bindgen(method)]
	pub async fn accept(this: &Listener) -> JsValue; // Returns a Conn

	pub type DatagramConn;
	#[wasm_bindgen(method)]
	pub fn addr(this: &DatagramConn) -> Addr;
	#[wasm_bindgen(method, catch)]
	pub async fn receive(this: &DatagramConn, p: Option<&mut [u8]>) -> Result<JsValue, JsValue>; // Promise<[Uint8Array, Addr]>
	#[wasm_bindgen(method, catch)]
	pub async fn send(this: &DatagramConn, p: &[u8], addr: Addr) -> Result<JsValue, JsValue>; // Promise<number>
	#[wasm_bindgen(method)]
	pub fn close(this: &DatagramConn);

	pub type Conn;
	#[wasm_bindgen(method, getter)]
	pub fn localAddr(this: &Conn) -> Addr;
	#[wasm_bindgen(method, getter)]
	pub fn remoteAddr(this: &Conn) -> Addr;
	#[wasm_bindgen(method, catch)]
	pub async fn read(this: &Conn, p: &mut [u8]) -> Result<JsValue, JsValue>; // Promise<number | null>
	#[wasm_bindgen(method, catch)]
	pub async fn write(this: &Conn, p: &[u8]) -> Result<JsValue, JsValue>; // Promise<number>
	#[wasm_bindgen(method)]
	pub fn close(this: &Conn);

	pub type Addr;
	#[wasm_bindgen(method, getter)]
	pub fn transport(this: &Addr) -> String;
	#[wasm_bindgen(method, getter)]
	pub fn hostname(this: &Addr) -> String;
	#[wasm_bindgen(method, getter)]
	pub fn port(this: &Addr) -> u16;

	#[wasm_bindgen(js_namespace = Deno)]
	pub fn listen(options: JsValue) -> Listener;

	#[wasm_bindgen(js_namespace = Deno)]
	pub fn listenDatagram(options: JsValue) -> DatagramConn;
	
	fn setTimeout(cb: Function, millis: u32);

	// #[wasm_bindgen(catch, js_namespace = Deno)]
	// async fn resolveDns(query: &str, record_type: &str) -> Result<JsValue, JsValue>;
}
pub async fn sleep(dur: Duration) {
	let _ = wasm_bindgen_futures::JsFuture::from(Promise::new(&mut move |res, _rej| {
		setTimeout(res, dur.as_millis() as u32);
	})).await;
}
pub struct Elapsed;
pub async fn timeout<F: Future>(d: Duration, f: F) -> Result<F::Output, Elapsed> {
	tokio::select! {
		_ = sleep(d) => Err(Elapsed),
		v = f => Ok(v)
	}
}

impl DatagramConn {
	pub async fn bind<A: ToSocketAddrs>(addr: A) -> std::io::Result<Self> {
		let sa = lookup_host(addr).await?.next().ok_or(std::io::Error::new(std::io::ErrorKind::Other, "No address found during bind"))?;
		let options = Object::new();
		let _ = Reflect::set(&options, &JsValue::from_str("hostname"), &JsValue::from_str(&sa.ip().to_string()));
		let _ = Reflect::set(&options, &JsValue::from_str("port"), &JsValue::from(sa.port()));
		Ok(listenDatagram(JsValue::from(options)))
	}
}

pub trait ToSocketAddrs {
	fn to_addrs(&self) -> Result<Vec<SocketAddr>, (String, u16)>;
}
impl<T: ?Sized + ToSocketAddrs> ToSocketAddrs for &T {
	fn to_addrs(&self) -> Result<Vec<SocketAddr>, (String, u16)> {
		T::to_addrs(self)
	}
}
impl ToSocketAddrs for SocketAddr {
	fn to_addrs(&self) -> Result<Vec<SocketAddr>, (String, u16)> {
		Ok(vec![*self])
	}
}
impl ToSocketAddrs for SocketAddrV4 {
	fn to_addrs(&self) -> Result<Vec<SocketAddr>, (String, u16)> {
		Ok(vec![SocketAddr::V4(*self)])
	}
}
impl ToSocketAddrs for SocketAddrV6 {
	fn to_addrs(&self) -> Result<Vec<SocketAddr>, (String, u16)> {
		Ok(vec![SocketAddr::V6(*self)])
	}
}
impl ToSocketAddrs for (IpAddr, u16) {
	fn to_addrs(&self) -> Result<Vec<SocketAddr>, (String, u16)> {
		Ok(vec![SocketAddr::new(self.0, self.1)])
	}
}
impl ToSocketAddrs for (Ipv4Addr, u16) {
	fn to_addrs(&self) -> Result<Vec<SocketAddr>, (String, u16)> {
		Ok(vec![SocketAddr::new(IpAddr::V4(self.0), self.1)])
	}
}
impl ToSocketAddrs for (Ipv6Addr, u16) {
	fn to_addrs(&self) -> Result<Vec<SocketAddr>, (String, u16)> {
		Ok(vec![SocketAddr::new(IpAddr::V6(self.0), self.1)])
	}
}
impl ToSocketAddrs for &[SocketAddr] {
	fn to_addrs(&self) -> Result<Vec<SocketAddr>, (String, u16)> {
		Ok(self.into_iter().cloned().collect())
	}
}
impl ToSocketAddrs for str {
	fn to_addrs(&self) -> Result<Vec<SocketAddr>, (String, u16)> {
		let (ip, port) = self.rsplit_once(':').unwrap();
		let ip = ip.to_owned();
		let port = port.parse().unwrap();
		Err((ip, port))
	}
}
impl ToSocketAddrs for (&str, u16) {
	fn to_addrs(&self) -> Result<Vec<SocketAddr>, (String, u16)> {
		Err((self.0.to_owned(), self.1))
	}
}
impl ToSocketAddrs for (String, u16) {
	fn to_addrs(&self) -> Result<Vec<SocketAddr>, (String, u16)> {
		Err(self.clone())
	}
}
impl ToSocketAddrs for String {
	fn to_addrs(&self) -> Result<Vec<SocketAddr>, (String, u16)> {
		self.as_str().to_addrs()
	}
}

pub async fn lookup_host<T>(host: T) -> std::io::Result<impl Iterator<Item = SocketAddr>>
where
	T: ToSocketAddrs,
{
	match host.to_addrs() {
		Ok(v) => Ok(v.into_iter()),
		Err((query, port)) => {
			match tokio::try_join!(
				resolveDns(&query, "A"),
				resolveDns(&query, "AAAA")
			) {
				Err(_e) => {
					Err(std::io::Error::new(std::io::ErrorKind::Other, "Deno's resolveDns failed."))
				},
				Ok((v4, v6)) => {
					let v4 = v4.unchecked_into::<Array>();
					let v6 = v6.unchecked_into::<Array>();
					Ok(v4.iter().chain(v6.iter()).flat_map(|v| -> Option<IpAddr> { v.as_string().unwrap().parse().ok() }).map(|ip| SocketAddr::new(ip, port)).collect::<Vec<_>>().into_iter())
				}
			}
		}
	}
}
