//! `RawHidChannel` implementation over `async-hid`.
//!
//! The published `hidpp 0.2` derives short/long-report support by reading the
//! HID report descriptor, but `async-hid 0.4` only exposes descriptors on
//! Linux. We avoid the path entirely by pre-filtering to the Logitech HID++
//! long-report usage page at enumeration time, then returning a hardcoded
//! `Some((true, true))` from `supports_short_long_hidpp`.

use std::error::Error;

use async_hid::{AsyncHidRead, AsyncHidWrite, DeviceInfo, DeviceReader, DeviceWriter};
use hidpp::{async_trait, channel::RawHidChannel};
use tokio::sync::Mutex;

pub(crate) struct AsyncHidChannel {
    reader: Mutex<DeviceReader>,
    writer: Mutex<DeviceWriter>,
    info: DeviceInfo,
}

impl AsyncHidChannel {
    pub(crate) fn new(reader: DeviceReader, writer: DeviceWriter, info: DeviceInfo) -> Self {
        Self {
            reader: Mutex::new(reader),
            writer: Mutex::new(writer),
            info,
        }
    }
}

#[async_trait]
impl RawHidChannel for AsyncHidChannel {
    fn vendor_id(&self) -> u16 {
        self.info.vendor_id
    }

    fn product_id(&self) -> u16 {
        self.info.product_id
    }

    async fn write_report(&self, src: &[u8]) -> Result<usize, Box<dyn Error>> {
        let mut w = self.writer.lock().await;
        w.write_output_report(src).await?;
        Ok(src.len())
    }

    async fn read_report(&self, buf: &mut [u8]) -> Result<usize, Box<dyn Error>> {
        let mut r = self.reader.lock().await;
        Ok(r.read_input_report(buf).await?)
    }

    fn supports_short_long_hidpp(&self) -> Option<(bool, bool)> {
        Some((true, true))
    }

    async fn get_report_descriptor(&self, _buf: &mut [u8]) -> Result<usize, Box<dyn Error>> {
        Err("get_report_descriptor is not implemented; pre-filter to HID++ usage pages".into())
    }
}
