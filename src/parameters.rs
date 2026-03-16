//! Parameters and values which may be passed to the device.

use std::ops::RangeInclusive;

/// Error which may be thrown when trying to convert a value.
///
/// `T` is the backing type.
#[derive(Debug, thiserror::Error)]
#[error(
    "bad value for {type_name}, got {actual} which does not fit into the acceptable range {acceptable:?}"
)]
pub struct ValueError<T> {
    /// Acceptable range of values for this type.
    pub acceptable: RangeInclusive<T>,
    /// Actual value received.
    pub actual: T,
    /// Name of the type we were trying to construct.
    pub type_name: &'static str,
}

/// Changes to channel strength and channel waveform data for both channels.
///
/// Should be written every 100ms.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ChannelValues {
    /// Sequence number of this command.
    ///
    /// Top 4 bits are ignored.
    pub sequence_number: u8,
    /// Change in strength for both channels.
    pub strength_change: StrengthChange,
    /// Waveform for 100ms on channel A.
    pub channel_a_waveform: ChannelWaveform,
    /// Waveform for 100ms on channel B.
    pub channel_b_waveform: ChannelWaveform,
}

/// Strength value change direction for a single channel.
///
/// Effectively equivalent to a number sign.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum StrengthChange {
    /// Strength value has not changed.
    Unchanged,
    /// Strength value must increase by the given amount.
    Increase {
        /// Channel A strength value.
        channel_a: StrengthValue,
        /// Channel B strength value.
        channel_b: StrengthValue,
    },
    /// Strength value must decrease by the given amount.
    Decrease {
        /// Channel A strength value.
        channel_a: StrengthValue,
        /// Channel B strength value.
        channel_b: StrengthValue,
    },
    /// Strength value must be set to the given absolute value.
    Set {
        /// Channel A strength value.
        channel_a: StrengthValue,
        /// Channel B strength value.
        channel_b: StrengthValue,
    },
}

/// Absolute strength value for a channel.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct StrengthValue(pub(crate) u8);

impl StrengthValue {
    const ACCEPTABLE_VALUES: RangeInclusive<u8> = 0..=200;

    /// Create a new value.
    ///
    /// # Panics
    ///
    /// Panics when `value` is more than `200`.
    pub fn new(value: u8) -> StrengthValue {
        value.try_into().expect("bad value passed")
    }
}

impl TryFrom<u8> for StrengthValue {
    type Error = ValueError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if !Self::ACCEPTABLE_VALUES.contains(&value) {
            return Err(ValueError {
                acceptable: Self::ACCEPTABLE_VALUES,
                actual: value,
                type_name: stringify!(StrengthValue),
            });
        }

        Ok(Self(value))
    }
}

/// Full channel waveform.
///
/// Represents 100ms of waveform output.
pub type ChannelWaveform = [ChannelWaveformSegment; 4];

/// Single channel waveform segment.
///
/// Represents 25ms of waveform output.
/// Has a `TryFrom` impl against byte pairs:
///
/// ```
/// # use coyote::parameters::ChannelWaveformSegment;
/// let segment: ChannelWaveformSegment = (10, 100).try_into().unwrap();
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct ChannelWaveformSegment {
    pub(crate) frequency: u8,
    pub(crate) intensity: u8,
}

impl ChannelWaveformSegment {
    const FREQUENCY_ACCEPTABLE: RangeInclusive<u8> = 10..=240;
    const INTENSITY_ACCEPTABLE: RangeInclusive<u8> = 0..=100;

    /// Create a new waveform segment with the given values.
    ///
    /// # Panics
    ///
    /// Panics when `frequency` is not in `10..=240` or `intensity` is not in `0..=100`.
    pub fn new(frequency: u8, intensity: u8) -> Self {
        (frequency, intensity).try_into().expect("bad values given")
    }
}

impl TryFrom<(u8, u8)> for ChannelWaveformSegment {
    type Error = ValueError<u8>;

    fn try_from((frequency, intensity): (u8, u8)) -> Result<Self, Self::Error> {
        if !Self::FREQUENCY_ACCEPTABLE.contains(&frequency) {
            return Err(ValueError {
                acceptable: Self::FREQUENCY_ACCEPTABLE,
                actual: frequency,
                type_name: concat!(stringify!(ChannelWaveformSegment), ".frequency"),
            });
        }

        if !Self::INTENSITY_ACCEPTABLE.contains(&intensity) {
            return Err(ValueError {
                acceptable: Self::INTENSITY_ACCEPTABLE,
                actual: intensity,
                type_name: concat!(stringify!(ChannelWaveformSegment), ".intensity"),
            });
        };

        Ok(Self {
            frequency,
            intensity,
        })
    }
}

/// Limiting values for both channels on a [`crate::PulseHost3`].
///
/// # Example
///
/// ```
/// use coyote::parameters::{ChannelLimits, ChannelLimit};
///
/// let limits = coyote::parameters::ChannelLimits {
///     channel_a: ChannelLimit {
///         upper_limit: 100.try_into().unwrap(),
///         frequency_balance: 100,
///         strength_balance: 100,
///     },
///     channel_b: ChannelLimit {
///         upper_limit: 101.try_into().unwrap(),
///         frequency_balance: 50,
///         strength_balance: 50,
///     },
/// };
/// ```
// TODO: Default
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelLimits {
    /// Channel A characteristics.
    pub channel_a: ChannelLimit,
    /// Channel B characteristics.
    pub channel_b: ChannelLimit,
}

/// Limiting values for a channel on a [`crate::PulseHost3`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelLimit {
    /// Maximum channel strength.
    pub upper_limit: StrengthValue,
    /// Parameter to adjust perceived high and low frequencies.
    pub frequency_balance: u8,
    /// Parameter to adjust waveform pulse width.
    pub strength_balance: u8,
}
