use crate::error::Result;

use super::*;

/// NoOp is an Interceptor that does not modify any packets. It can embedded in other interceptors, so it's
/// possible to implement only a subset of the methods.
pub struct NoOp;

#[async_trait(?Send)]
impl Interceptor for NoOp {
    /// bind_rtcp_reader lets you modify any incoming RTCP packets. It is called once per sender/receiver, however this might
    /// change in the future. The returned method will be called once per packet batch.
    async fn bind_rtcp_reader(
        &self,
        reader: Arc<dyn RTCPReader>,
    ) -> Arc<dyn RTCPReader> {
        reader
    }

    /// bind_rtcp_writer lets you modify any outgoing RTCP packets. It is called once per PeerConnection. The returned method
    /// will be called once per packet batch.
    async fn bind_rtcp_writer(
        &self,
        writer: Arc<dyn RTCPWriter>,
    ) -> Arc<dyn RTCPWriter> {
        writer
    }

    /// bind_local_stream lets you modify any outgoing RTP packets. It is called once for per LocalStream. The returned method
    /// will be called once per rtp packet.
    async fn bind_local_stream(
        &self,
        _info: &StreamInfo,
        writer: Arc<dyn RTPWriter>,
    ) -> Arc<dyn RTPWriter> {
        writer
    }

    /// unbind_local_stream is called when the Stream is removed. It can be used to clean up any data related to that track.
    async fn unbind_local_stream(&self, _info: &StreamInfo) {}

    /// bind_remote_stream lets you modify any incoming RTP packets. It is called once for per RemoteStream. The returned method
    /// will be called once per rtp packet.
    async fn bind_remote_stream(
        &self,
        _info: &StreamInfo,
        reader: Arc<dyn RTPReader>,
    ) -> Arc<dyn RTPReader> {
        reader
    }

    /// unbind_remote_stream is called when the Stream is removed. It can be used to clean up any data related to that track.
    async fn unbind_remote_stream(&self, _info: &StreamInfo) {}

    /// close closes the Interceptor, cleaning up any data if necessary.
    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

#[async_trait(?Send)]
impl RTPReader for NoOp {
    async fn read(&self, _buf: &mut [u8], a: &Attributes) -> Result<(usize, Attributes)> {
        Ok((0, a.clone()))
    }
}

#[async_trait(?Send)]
impl RTCPReader for NoOp {
    async fn read(&self, _buf: &mut [u8], a: &Attributes) -> Result<(usize, Attributes)> {
        Ok((0, a.clone()))
    }
}
