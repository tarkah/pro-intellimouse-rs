use byteorder::{BigEndian, ByteOrder, LittleEndian};
use hidapi::HidDevice;
use log::info;

const VID: u16 = 0x045E;
const PID: u16 = 0x082A;
const WRITE_REPORT_ID: u8 = 0x24;
const WRITE_REPORT_LEN: usize = 73;
const READ_REPORT_ID: u8 = 0x27;
const READ_REPORT_LEN: usize = 41;

pub struct IntelliMouse {
    device: HidDevice,
}

impl IntelliMouse {
    pub fn connect() -> Result<Self> {
        let api = hidapi::HidApi::new().map_err(Error::ApiContext)?;

        let device = api.open(VID, PID).map_err(Error::DeviceOpen)?;

        info!("Successfully connected to Pro IntelliMouse");

        Ok(IntelliMouse { device })
    }

    pub fn read_property(&self, property: Property) -> Result<PropertyValue> {
        let mut report = [0; WRITE_REPORT_LEN];
        report[0] = WRITE_REPORT_ID;
        report[1] = property.as_read_byte();

        info!("send feature report...");

        self.device
            .send_feature_report(&report)
            .map_err(Error::HidError)?;

        let mut result = [0; READ_REPORT_LEN];
        result[0] = READ_REPORT_ID;

        info!("get feature report...");

        let result_len = self
            .device
            .get_feature_report(&mut result)
            .map_err(Error::HidError)?;

        let data = &result[1..1 + result_len as usize];

        Ok(match property {
            Property::Color => PropertyValue::Color(BigEndian::read_u32(&data)),
            Property::Distance => PropertyValue::Distance(Distance::from(data[0])),
            Property::Dpi => PropertyValue::Dpi(LittleEndian::read_u32(&data)),
            Property::PollingRate => PropertyValue::PollingRate(PollingRate::from(data[0])),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Property {
    Color,
    Distance,
    Dpi,
    PollingRate,
}

impl Property {
    fn as_write_byte(self) -> u8 {
        use Property::*;

        match self {
            Color => 0xB2,
            Dpi => 0x96,
            PollingRate => 0x83,
            Distance => 0xB8,
        }
    }

    fn as_read_byte(self) -> u8 {
        if self == Property::Distance {
            self.as_write_byte() - 2
        } else {
            self.as_write_byte() + 1
        }
    }
}

#[derive(Debug)]
pub enum PropertyValue {
    Color(u32),
    Distance(Distance),
    Dpi(u32),
    PollingRate(PollingRate),
}

#[derive(Debug)]
pub enum PollingRate {
    Rate1000,
    Rate500,
    Rate125,
}

impl From<u8> for PollingRate {
    fn from(byte: u8) -> Self {
        use PollingRate::*;

        match byte {
            0 => Rate1000,
            1 => Rate500,
            2 => Rate125,
            _ => panic!("Unknown polling rate byte: {}", byte),
        }
    }
}

#[derive(Debug)]
pub enum Distance {
    Two,
    Three,
}

impl From<u8> for Distance {
    fn from(byte: u8) -> Self {
        use Distance::*;

        match byte {
            0 => Two,
            1 => Three,
            _ => panic!("Unknown distance byte: {}", byte),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error connecting to hid context: {0}")]
    ApiContext(hidapi::HidError),
    #[error("error connecting to device: {0}")]
    DeviceOpen(hidapi::HidError),
    #[error("{0}")]
    HidError(hidapi::HidError),
}

pub type Result<T> = std::result::Result<T, Error>;
