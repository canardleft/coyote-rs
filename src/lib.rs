#![doc = include_str!("../README.md")]

use btleplug::api::{Characteristic, Peripheral, PeripheralProperties};
use tracing::{Span, debug, instrument, warn};
use uuid::Uuid;

mod messages;
pub mod parameters;

/// Error thrown by operations against a [`PulseHost3`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Bluetooth error while communicating with a device.
    #[error("bluetooth error: {0}")]
    BluetoothError(#[from] btleplug::Error),
    /// Error decoding a value.
    #[error("failed to decode value {value:?} given by device: {msg}")]
    DecodeError {
        /// Bytes returned by the device.
        value: Box<[u8]>,
        /// Error message.
        msg: String,
    },
    /// The device returned no properties.
    #[error("the device returned no properties")]
    NoProperties,
    /// The device didn't report an expected characteristic.
    #[error(
        "a characteristic with UUID {uuid} under service {uuid} was expected but not reported by the device"
    )]
    CharacteristicNotFound {
        /// UUID of the expected characteristic.
        uuid: Uuid,
        /// UUID of the expected characteristic's service.
        service_uuid: Uuid,
    },
}

/// "Pulse host"/central box, version 3.
pub struct PulseHost3<P> {
    peripheral: P,
    // TODO: can this change?
    perip_properties: PeripheralProperties,
    cmd_characteristic: Characteristic,
    resp_characteristic: Characteristic,
    // TODO: what does this do?
    #[allow(dead_code)]
    batt_characteristic: Characteristic,
}

impl<P> PulseHost3<P> {
    /// Check if a peripheral's properties match what we expect for this device.
    ///
    /// If this returns `false`, this probably means you are trying to connect to the the wrong device.
    pub fn peripheral_matches(properties: &PeripheralProperties) -> bool {
        if let Some(ref name) = properties.local_name {
            name.contains("47L121000")
        } else {
            false
        }
    }
}

impl<P: Peripheral> PulseHost3<P> {
    const CMD_RESP_SERVICE_UUID: Uuid = Uuid::from_u128(0x0000180c_0000_1000_8000_00805f9b34fb);
    const CMD_CHAR_UUID: Uuid = Uuid::from_u128(0x0000150a_0000_1000_8000_00805f9b34fb);
    const RESP_CHAR_UUID: Uuid = Uuid::from_u128(0x0000150b_0000_1000_8000_00805f9b34fb);

    const BATT_SERVICE_UUID: Uuid = Uuid::from_u128(0x00001500_0000_1000_8000_00805f9b34fb);
    const BATT_CHAR_UUID: Uuid = Uuid::from_u128(0x0000180a_0000_1000_8000_00805f9b34fb);

    fn extract_characteristic<'a>(
        chars: impl IntoIterator<Item = &'a Characteristic>,
        uuid: Uuid,
        service_uuid: Uuid,
    ) -> Result<Characteristic, Error> {
        chars
            .into_iter()
            .find(|ch| ch.uuid == uuid && ch.service_uuid == service_uuid)
            .cloned()
            .ok_or(Error::CharacteristicNotFound { uuid, service_uuid })
    }

    /// Create a new controller wrapping a bluetooth peripheral.
    ///
    /// - `peripheral` should be a **connected**
    /// Also sets limits on initialisation.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] when `peripheral` didn't provide properties or when there is a
    /// communication error over bluetooth.
    #[instrument(skip(peripheral), fields(addr))]
    pub async fn new(peripheral: P, limits: parameters::ChannelLimits) -> Result<Self, Error> {
        let Some(perip_properties) = peripheral.properties().await? else {
            return Err(Error::NoProperties);
        };

        // check it
        Span::current().record("addr", perip_properties.address.to_string_no_delim());
        if !Self::peripheral_matches(&perip_properties) {
            warn!("peripheral properties didn't match");
        }

        // extract characteristics
        peripheral.discover_services().await?;
        let chars = peripheral.characteristics();
        let cmd_characteristic =
            Self::extract_characteristic(&chars, Self::CMD_CHAR_UUID, Self::CMD_RESP_SERVICE_UUID)?;
        let resp_characteristic = Self::extract_characteristic(
            &chars,
            Self::RESP_CHAR_UUID,
            Self::CMD_RESP_SERVICE_UUID,
        )?;
        let batt_characteristic =
            Self::extract_characteristic(&chars, Self::BATT_CHAR_UUID, Self::BATT_SERVICE_UUID)?;

        let to_ret = Self {
            peripheral,
            perip_properties,
            cmd_characteristic,
            resp_characteristic,
            batt_characteristic,
        };

        // set limits - imperative that we do this to avoid nasty surprises
        // TODO: test for me
        to_ret.set_limits(limits).await?;

        Ok(to_ret)
    }

    /// Write channel values to the device.
    #[instrument]
    pub async fn write_values(
        &self,
        values: parameters::ChannelValues,
    ) -> Result<messages::PulseStrengthResponse, Error> {
        // send the command
        let encoded_command = messages::SetValuesCommand(values).encode();
        debug!(?encoded_command, "writing B0 command");
        self.peripheral
            .write(
                &self.cmd_characteristic,
                &encoded_command,
                btleplug::api::WriteType::WithResponse,
            )
            .await?;

        // read the response
        let raw_resp = self.peripheral.read(&self.resp_characteristic).await?;
        debug!(response = ?raw_resp.as_slice(), "read B1 response");
        let response =
            messages::PulseStrengthResponse::try_decode(raw_resp.as_slice()).map_err(|msg| {
                Error::DecodeError {
                    msg,
                    value: raw_resp.into_boxed_slice(),
                }
            })?;

        Ok(response)
    }

    /// Set the channel limits on this device.
    #[instrument]
    pub async fn set_limits(&self, limits: parameters::ChannelLimits) -> Result<(), Error> {
        let encoded_command = messages::SetLimitsCommand(limits.clone()).encode();
        debug!(?encoded_command, "setting limits");
        self.peripheral
            .write(
                &self.cmd_characteristic,
                &encoded_command,
                btleplug::api::WriteType::WithoutResponse,
            )
            .await?;
        Ok(())
    }
}

impl<P> core::fmt::Debug for PulseHost3<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Coyote 3.0 Pulse Host {{ addr: {}, name: {:?}{} }}",
            self.perip_properties.address,
            self.perip_properties.local_name,
            if !Self::peripheral_matches(&self.perip_properties) {
                " (mismatch!)"
            } else {
                ""
            }
        )?;
        Ok(())
    }
}
