use super::*;
use crate::codecs::*;
use crate::error::Result;

use chrono::prelude::*;
use std::time::{Duration, UNIX_EPOCH};

#[tokio::test]
async fn test_packetizer() -> Result<()> {
    let multiple_payload = Bytes::from_static(&[0; 128]);
    let g722 = Box::new(g7xx::G722Payloader {});
    let seq = Box::new(new_random_sequencer());

    //use the G722 payloader here, because it's very simple and all 0s is valid G722 data.
    let mut packetizer = new_packetizer(100, 98, 0x1234ABCD, g722, seq, 90000);
    let packets = packetizer.packetize(&multiple_payload, 2000).await?;

    if packets.len() != 2 {
        let mut packet_lengths = String::new();
        for i in 0..packets.len() {
            packet_lengths +=
                format!("Packet {} length {}\n", i, packets[i].payload.len()).as_str();
        }
        assert!(
            false,
            "Generated {} packets instead of 2\n{}",
            packets.len(),
            packet_lengths,
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_packetizer_abs_send_time() -> Result<()> {
    let g722 = Box::new(g7xx::G722Payloader {});
    let sequencer = Box::new(new_fixed_sequencer(1234));

    let time_gen: Option<FnTimeGen> = Some(Arc::new(
        || -> Pin<Box<dyn Future<Output = SystemTime> + 'static>> {
            Box::pin(async move {
                let loc = FixedOffset::west(5 * 60 * 60); // UTC-5
                let t = loc.ymd(1985, 6, 23).and_hms_nano(4, 0, 0, 0);
                UNIX_EPOCH
                    .checked_add(Duration::from_nanos(t.timestamp_nanos() as u64))
                    .unwrap_or(UNIX_EPOCH)
            })
        },
    ));

    //use the G722 payloader here, because it's very simple and all 0s is valid G722 data.
    let mut pktizer = PacketizerImpl {
        mtu: 100,
        payload_type: 98,
        ssrc: 0x1234ABCD,
        payloader: g722,
        sequencer,
        timestamp: 45678,
        clock_rate: 90000,
        abs_send_time: 0,
        time_gen,
    };
    pktizer.enable_abs_send_time(1);

    let payload = Bytes::from_static(&[0x11, 0x12, 0x13, 0x14]);
    let packets = pktizer.packetize(&payload, 2000).await?;

    let expected = Packet {
        header: Header {
            version: 2,
            padding: false,
            extension: true,
            marker: true,
            payload_type: 98,
            sequence_number: 1234,
            timestamp: 45678,
            ssrc: 0x1234ABCD,
            csrc: vec![],
            extension_profile: 0xBEDE,
            extensions: vec![Extension {
                id: 1,
                payload: Bytes::from_static(&[0x40, 0, 0]),
            }],
        },
        payload: Bytes::from_static(&[0x11, 0x12, 0x13, 0x14]),
    };

    if packets.len() != 1 {
        assert!(false, "Generated {} packets instead of 1", packets.len())
    }

    assert_eq!(expected, packets[0]);

    Ok(())
}

#[tokio::test]
async fn test_packetizer_timestamp_rollover_does_not_panic() -> Result<()> {
    let g722 = Box::new(g7xx::G722Payloader {});
    let seq = Box::new(new_random_sequencer());

    let payload = Bytes::from_static(&[0; 128]);
    let mut packetizer = new_packetizer(100, 98, 0x1234ABCD, g722, seq, 90000);

    packetizer.packetize(&payload, 10).await?;

    packetizer.packetize(&payload, u32::MAX).await?;

    packetizer.skip_samples(u32::MAX);

    Ok(())
}
