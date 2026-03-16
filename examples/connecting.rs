//! Connection and usage example.
//!
//! Connects and configures the device, then loops a simple waveform.
//!
//! This example is intended to illustrate how to connect this library to `btleplug`.
//! For more information, you should see that library's documentation.

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Manager, Peripheral};
use coyote::PulseHost3;
use coyote::parameters::{ChannelLimit, ChannelLimits, ChannelValues, StrengthChange};
use std::error::Error;
use std::time::Duration;
use tokio::time;
use tracing::info;

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

    let manager = Manager::new().await.unwrap();

    // get the first bluetooth adapter
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().nth(0).unwrap();

    // start scanning for devices
    info!("searching for coyote 3 devices");
    central.start_scan(ScanFilter::default()).await?;
    time::sleep(Duration::from_secs(2)).await;

    // figure out which connected peripheral is our coyote 3
    let mut peripheral = None;
    for p in central
        .peripherals()
        .await
        .expect("couldn't get peripherals")
    {
        if PulseHost3::<Peripheral>::peripheral_matches(
            &p.properties()
                .await
                .expect("couldn't get properties")
                .expect("peripheral gave no properties"),
        ) {
            peripheral = Some(p);
            break;
        }
    }
    let peripheral = peripheral.expect("couldn't find connected coyote 3 device");
    peripheral
        .connect()
        .await
        .expect("failed to connect to peripheral");

    // connect and configure the device
    info!("connecting to peripheral");
    let pulse_host = PulseHost3::new(
        peripheral,
        ChannelLimits {
            channel_a: ChannelLimit {
                upper_limit: 20.try_into().unwrap(),
                frequency_balance: 100,
                strength_balance: 100,
            },
            channel_b: ChannelLimit {
                upper_limit: 20.try_into().unwrap(),
                frequency_balance: 100,
                strength_balance: 100,
            },
        },
    )
    .await
    .expect("error initialising device");

    loop {
        pulse_host
            .write_values(ChannelValues {
                sequence_number: 0,
                strength_change: StrengthChange::Set {
                    channel_a: 50.try_into().unwrap(),
                    channel_b: 50.try_into().unwrap(),
                },
                channel_a_waveform: [
                    (100, 50).try_into().unwrap(),
                    (120, 50).try_into().unwrap(),
                    (100, 50).try_into().unwrap(),
                    (120, 50).try_into().unwrap(),
                ],
                channel_b_waveform: [
                    (100, 50).try_into().unwrap(),
                    (120, 50).try_into().unwrap(),
                    (100, 50).try_into().unwrap(),
                    (120, 50).try_into().unwrap(),
                ],
            })
            .await
            .expect("failed to set channel values");

        time::sleep(Duration::from_millis(100)).await;
    }
}
