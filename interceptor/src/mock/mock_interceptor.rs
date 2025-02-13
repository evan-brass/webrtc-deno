use crate::*;

use std::future::Future;
use std::pin::Pin;

pub type BindRtcpReaderFn = Box<
    dyn (Fn(
            Arc<dyn RTCPReader>,
        )
            -> Pin<Box<dyn Future<Output = Arc<dyn RTCPReader>>>>)
       
       ,
>;

pub type BindRtcpWriterFn = Box<
    dyn (Fn(
            Arc<dyn RTCPWriter>,
        )
            -> Pin<Box<dyn Future<Output = Arc<dyn RTCPWriter>>>>)
       
       ,
>;
pub type BindLocalStreamFn = Box<
    dyn (Fn(
            &StreamInfo,
            Arc<dyn RTPWriter>,
        ) -> Pin<Box<dyn Future<Output = Arc<dyn RTPWriter>>>>)
       
       ,
>;
pub type UnbindLocalStreamFn =
    Box<dyn (Fn(&StreamInfo) -> Pin<Box<dyn Future<Output = ()>>>)>;
pub type BindRemoteStreamFn = Box<
    dyn (Fn(
            &StreamInfo,
            Arc<dyn RTPReader>,
        ) -> Pin<Box<dyn Future<Output = Arc<dyn RTPReader>>>>)
       
       ,
>;
pub type UnbindRemoteStreamFn =
    Box<dyn (Fn(&StreamInfo) -> Pin<Box<dyn Future<Output = ()>>>)>;
pub type CloseFn =
    Box<dyn (Fn() -> Pin<Box<dyn Future<Output = Result<()>>>>)>;

/// MockInterceptor is an mock Interceptor fot testing.
#[derive(Default)]
pub struct MockInterceptor {
    pub bind_rtcp_reader_fn: Option<BindRtcpReaderFn>,
    pub bind_rtcp_writer_fn: Option<BindRtcpWriterFn>,
    pub bind_local_stream_fn: Option<BindLocalStreamFn>,
    pub unbind_local_stream_fn: Option<UnbindLocalStreamFn>,
    pub bind_remote_stream_fn: Option<BindRemoteStreamFn>,
    pub unbind_remote_stream_fn: Option<UnbindRemoteStreamFn>,
    pub close_fn: Option<CloseFn>,
}

#[async_trait(?Send)]
impl Interceptor for MockInterceptor {
    /// bind_rtcp_reader lets you modify any incoming RTCP packets. It is called once per sender/receiver, however this might
    /// change in the future. The returned method will be called once per packet batch.
    async fn bind_rtcp_reader(
        &self,
        reader: Arc<dyn RTCPReader>,
    ) -> Arc<dyn RTCPReader> {
        if let Some(f) = &self.bind_rtcp_reader_fn {
            f(reader).await
        } else {
            reader
        }
    }

    /// bind_rtcp_writer lets you modify any outgoing RTCP packets. It is called once per PeerConnection. The returned method
    /// will be called once per packet batch.
    async fn bind_rtcp_writer(
        &self,
        writer: Arc<dyn RTCPWriter>,
    ) -> Arc<dyn RTCPWriter> {
        if let Some(f) = &self.bind_rtcp_writer_fn {
            f(writer).await
        } else {
            writer
        }
    }

    /// bind_local_stream lets you modify any outgoing RTP packets. It is called once for per LocalStream. The returned method
    /// will be called once per rtp packet.
    async fn bind_local_stream(
        &self,
        info: &StreamInfo,
        writer: Arc<dyn RTPWriter>,
    ) -> Arc<dyn RTPWriter> {
        if let Some(f) = &self.bind_local_stream_fn {
            f(info, writer).await
        } else {
            writer
        }
    }

    /// unbind_local_stream is called when the Stream is removed. It can be used to clean up any data related to that track.
    async fn unbind_local_stream(&self, info: &StreamInfo) {
        if let Some(f) = &self.unbind_local_stream_fn {
            f(info).await
        }
    }

    /// bind_remote_stream lets you modify any incoming RTP packets. It is called once for per RemoteStream. The returned method
    /// will be called once per rtp packet.
    async fn bind_remote_stream(
        &self,
        info: &StreamInfo,
        reader: Arc<dyn RTPReader>,
    ) -> Arc<dyn RTPReader> {
        if let Some(f) = &self.bind_remote_stream_fn {
            f(info, reader).await
        } else {
            reader
        }
    }

    /// unbind_remote_stream is called when the Stream is removed. It can be used to clean up any data related to that track.
    async fn unbind_remote_stream(&self, info: &StreamInfo) {
        if let Some(f) = &self.unbind_remote_stream_fn {
            f(info).await
        }
    }

    /// close closes the Interceptor, cleaning up any data if necessary.
    async fn close(&self) -> Result<()> {
        if let Some(f) = &self.close_fn {
            f().await
        } else {
            Ok(())
        }
    }
}
