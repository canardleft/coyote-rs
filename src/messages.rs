use crate::parameters::{ChannelValues, StrengthChange};

#[derive(Debug)]
pub struct SetValuesCommand(pub ChannelValues);

impl SetValuesCommand {
    pub fn encode(&self) -> [u8; 20] {
        let mut output = [0; 20];

        // header
        output[0] = 0xB0;

        // serial
        output[1] |= self.0.sequence_number << 4;

        // strength change
        match self.0.strength_change {
            StrengthChange::Unchanged => {
                output[1] |= 0b00;
            }
            StrengthChange::Increase {
                channel_a,
                channel_b,
            } => {
                output[1] |= 0b01;
                output[2] = channel_a.0;
                output[3] = channel_b.0;
            }
            StrengthChange::Decrease {
                channel_a,
                channel_b,
            } => {
                output[1] |= 0b10;
                output[2] = channel_a.0;
                output[3] = channel_b.0;
            }
            StrengthChange::Set {
                channel_a,
                channel_b,
            } => {
                output[1] |= 0b11;
                output[2] = channel_a.0;
                output[3] = channel_b.0;
            }
        };

        // channel a waveform
        for i in 0..4 {
            output[4 + i] = self.0.channel_a_waveform[i].frequency;
        }
        for i in 0..4 {
            output[8 + i] = self.0.channel_a_waveform[i].intensity;
        }

        // channel b waveform
        for i in 0..4 {
            output[12 + i] = self.0.channel_b_waveform[i].frequency;
        }
        for i in 0..4 {
            output[16 + i] = self.0.channel_b_waveform[i].intensity;
        }

        output
    }
}
/// `B1` message returned by the device when pulse strength changes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PulseStrengthResponse {
    /// Number in sequence for this message.
    ///
    /// When non-zero, corresponds to the sequence number in a [`SetValuesCommand`].
    pub sequence_number: u8,
    /// Actual intensity of Channel A.
    pub channel_a_intensity: u8,
    /// Actual intensity of Channel B.
    pub channel_b_intensity: u8,
}

impl PulseStrengthResponse {
    pub fn try_decode(value: &[u8]) -> Result<Self, String> {
        match value.as_array() {
            Some(
                &[
                    0xB1,
                    sequence_number,
                    channel_a_intensity,
                    channel_b_intensity,
                ],
            ) => Ok(Self {
                sequence_number,
                channel_a_intensity,
                channel_b_intensity,
            }),
            Some(_) => Err(format!(
                "header byte 0x{:x} does not match expected value of 0xB1 for B1 message",
                value[0]
            )),
            None => Err(format!(
                "bad length {} for B1 message, should be exactly 4 bytes",
                value.len()
            )),
        }
    }
}

/// `BF` command to set upper limits on a channel.
pub struct SetLimitsCommand(pub crate::parameters::ChannelLimits);

impl SetLimitsCommand {
    pub fn encode(&self) -> [u8; 7] {
        [
            0xbf,
            self.0.channel_a.upper_limit.0,
            self.0.channel_b.upper_limit.0,
            self.0.channel_a.frequency_balance,
            self.0.channel_b.frequency_balance,
            self.0.channel_a.strength_balance,
            self.0.channel_b.strength_balance,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use insta::assert_snapshot;
    use rstest::rstest;

    #[rstest]
    #[case(SetValuesCommand(ChannelValues {
        sequence_number: 0b1110,
        strength_change: StrengthChange::Increase{
            channel_a: 101.try_into().unwrap(),
            channel_b: 105.try_into().unwrap(),
        },
        channel_a_waveform:  [
            (102, 50).try_into().unwrap(),
            (103, 51).try_into().unwrap(),
            (104, 52).try_into().unwrap(),
            (105, 49).try_into().unwrap(),
        ],
        channel_b_waveform: [
            (94, 42).try_into().unwrap(),
            (95, 43).try_into().unwrap(),
            (96, 44).try_into().unwrap(),
            (97, 41).try_into().unwrap(),
        ],
    }))]
    fn test_encoding(#[case] value: SetValuesCommand) {
        let encoded: String = value
            .encode()
            .into_iter()
            .map(|byte| format!("{byte:x}"))
            .collect();
        assert_snapshot!(format!("0x{encoded}"));
    }
}
