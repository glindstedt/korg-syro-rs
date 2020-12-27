//!
//! Bindings for building patterns for the Volca Sample sequencer.
//!
//! # Examples
//!
//! Simple
//!
//! ```rust
//! use korg_syro::SyroStream;
//! use korg_syro::pattern::*;
//!
//! let mut syro_stream = SyroStream::default();
//!
//! let mut pattern = Pattern::default();
//! pattern.with_part(
//!     0u8,
//!     Part::for_sample(0)?.with_steps(
//!         Steps::builder()
//!             .on(Step::One)
//!             .on(Step::Three)
//!             .on(Step::Five)
//!             .on(Step::Seven)
//!             .build(),
//!     ).build(),
//! )?;
//!
//! syro_stream.add_pattern(0, pattern)?;
//!
//! # Ok::<(), korg_syro::SyroError>(())
//! ```
//!
//! Bells & Whistles
//!
//! ```rust
//! use korg_syro::SyroStream;
//! use korg_syro::pattern::*;
//! use korg_syro::pattern::Toggle::*;
//!
//! let mut syro_stream = SyroStream::default();
//!
//! let mut pattern = Pattern::default();
//! pattern.with_part(
//!     0u8,
//!     Part::for_sample(0)?.with_steps(
//!         Steps::builder()
//!             .on(Step::One)
//!             .on(Step::Three)
//!             .on(Step::Five)
//!             .on(Step::Seven)
//!             .build(),
//!     )
//!     .level(42)?
//!     .pan(42)?
//!     .amp_eg_attack(42)?
//!     .amp_eg_decay(42)?
//!     .pitch_eg_attack(42)?
//!     .pitch_eg_int(42)?
//!     .pitch_eg_decay(42)?
//!     .starting_point(42)?
//!     .length(42)?
//!     .hi_cut(42)?
//!     .motion(On)
//!     .looped(On)
//!     .reverb(On)
//!     .reverse(On)
//!     .mute(Off)
//!     .build()
//! )?;
//!
//! syro_stream.add_pattern(0, pattern)?;
//!
//! # Ok::<(), korg_syro::SyroError>(())
//! ```
//!
use korg_syro_sys::{
    VolcaSample_Part_Data, VolcaSample_Pattern_Data, VOLCASAMPLE_FUNC_LOOP,
    VOLCASAMPLE_FUNC_MOTION, VOLCASAMPLE_FUNC_MUTE, VOLCASAMPLE_FUNC_REVERB,
    VOLCASAMPLE_FUNC_REVERSE, VOLCASAMPLE_PARAM_AMPEG_ATTACK, VOLCASAMPLE_PARAM_AMPEG_DECAY,
    VOLCASAMPLE_PARAM_HICUT, VOLCASAMPLE_PARAM_LENGTH, VOLCASAMPLE_PARAM_LEVEL,
    VOLCASAMPLE_PARAM_PAN, VOLCASAMPLE_PARAM_PITCHEG_ATTACK, VOLCASAMPLE_PARAM_PITCHEG_DECAY,
    VOLCASAMPLE_PARAM_PITCHEG_INT, VOLCASAMPLE_PARAM_SPEED, VOLCASAMPLE_PARAM_START_POINT,
};
pub use num_enum;
use num_enum::TryFromPrimitive;

use crate::macros::*;
use crate::{check_sample_index, SyroError};

/// Defines the available steps
#[derive(Copy, Clone, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum Step {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
    Thirteen,
    Fourteen,
    Fifteen,
    Sixteen,
}

impl Step {
    pub fn to_bitmask(self) -> u16 {
        1 << self as u16
    }
}

/// Builder for a step sequence
#[derive(Copy, Clone, Debug)]
pub struct Steps {
    steps: u16,
}

impl Steps {
    pub fn builder() -> Self {
        Self { steps: 0 }
    }

    /// Turns on the step
    pub fn on(&mut self, step: Step) -> &mut Self {
        self.steps |= step.to_bitmask();
        self
    }

    pub fn build(self) -> Self {
        self
    }

    pub fn to_bytes(self) -> u16 {
        self.steps
    }
}

/// Defines a toggle value
#[derive(Copy, Clone, Debug)]
pub enum Toggle {
    On,
    Off,
}

max_check!(pattern_index, 9);
max_check!(part_index, 9);

max_check!(level, 127);
bounds_check!(pan, 1, 127);
bounds_check!(speed_semitone, 40, 88);
bounds_check!(speed_continuous, 129, 255);
max_check!(amp_eg_attack, 127);
max_check!(amp_eg_decay, 127);
bounds_check!(pitch_eg_int, 1, 127);
max_check!(pitch_eg_attack, 127);
max_check!(pitch_eg_decay, 127);
max_check!(starting_point, 127);
max_check!(length, 127);
max_check!(hi_cut, 127);

// there's two valid ranges for speed
fn check_speed(speed: u8) -> Result<(), SyroError> {
    check_speed_semitone(speed).or(check_speed_continuous(speed))
}

/// Defines a part of a sequence pattern
#[derive(Copy, Clone, Debug)]
pub struct Part {
    data: VolcaSample_Part_Data,
}

macro_rules! impl_func_memory_part {
    ($i:ident, $j:ident) => {
        paste! {
            pub fn [<$i>](&mut self, value: Toggle) -> &mut Self {
                self.toggle_func_memory_part($j, value);
                self
            }
        }
    };
}

impl Part {
    pub fn for_sample(sample_num: u16) -> Result<Self, SyroError> {
        check_sample_index(sample_num as u8)?;
        let mut data = VolcaSample_Part_Data::default();
        data.SampleNum = sample_num;

        Ok(Self { data })
    }

    pub fn with_steps(&mut self, steps: Steps) -> &mut Self {
        println!("Steps: {:?}", steps);
        self.data.StepOn = steps.to_bytes();
        self
    }

    fn toggle_func_memory_part(&mut self, func: u32, value: Toggle) {
        match value {
            Toggle::On => {
                self.data.FuncMemoryPart |= func as u8;
            }
            Toggle::Off => {
                self.data.FuncMemoryPart &= !(func as u8);
            }
        }
    }

    impl_func_memory_part!(motion, VOLCASAMPLE_FUNC_MOTION);
    impl_func_memory_part!(looped, VOLCASAMPLE_FUNC_LOOP);
    impl_func_memory_part!(reverb, VOLCASAMPLE_FUNC_REVERB);
    impl_func_memory_part!(reverse, VOLCASAMPLE_FUNC_REVERSE);

    pub fn mute(&mut self, value: Toggle) -> &mut Self {
        // apparently mute toggle is reversed
        match value {
            Toggle::On => {
                self.data.FuncMemoryPart &= VOLCASAMPLE_FUNC_MUTE as u8;
            }
            Toggle::Off => {
                self.data.FuncMemoryPart |= !(VOLCASAMPLE_FUNC_MUTE as u8);
            }
        }
        self
    }

    pub fn level(&mut self, level: u8) -> Result<&mut Self, SyroError> {
        check_level(level)?;
        self.data.Param[VOLCASAMPLE_PARAM_LEVEL as usize] = level;
        Ok(self)
    }

    pub fn pan(&mut self, pan: u8) -> Result<&mut Self, SyroError> {
        check_pan(pan)?;
        self.data.Param[VOLCASAMPLE_PARAM_PAN as usize] = pan;
        Ok(self)
    }

    pub fn speed(&mut self, speed: u8) -> Result<&mut Self, SyroError> {
        check_speed(speed)?;
        self.data.Param[VOLCASAMPLE_PARAM_SPEED as usize] = speed;
        Ok(self)
    }

    pub fn amp_eg_attack(&mut self, amp_eg_attack: u8) -> Result<&mut Self, SyroError> {
        check_amp_eg_attack(amp_eg_attack)?;
        self.data.Param[VOLCASAMPLE_PARAM_AMPEG_ATTACK as usize] = amp_eg_attack;
        Ok(self)
    }

    pub fn amp_eg_decay(&mut self, amp_eg_decay: u8) -> Result<&mut Self, SyroError> {
        check_amp_eg_decay(amp_eg_decay)?;
        self.data.Param[VOLCASAMPLE_PARAM_AMPEG_DECAY as usize] = amp_eg_decay;
        Ok(self)
    }

    pub fn pitch_eg_attack(&mut self, pitch_eg_attack: u8) -> Result<&mut Self, SyroError> {
        check_pitch_eg_attack(pitch_eg_attack)?;
        self.data.Param[VOLCASAMPLE_PARAM_PITCHEG_ATTACK as usize] = pitch_eg_attack;
        Ok(self)
    }

    pub fn pitch_eg_int(&mut self, pitch_eg_int: u8) -> Result<&mut Self, SyroError> {
        check_pitch_eg_int(pitch_eg_int)?;
        self.data.Param[VOLCASAMPLE_PARAM_PITCHEG_INT as usize] = pitch_eg_int;
        Ok(self)
    }

    pub fn pitch_eg_decay(&mut self, pitch_eg_decay: u8) -> Result<&mut Self, SyroError> {
        check_pitch_eg_decay(pitch_eg_decay)?;
        self.data.Param[VOLCASAMPLE_PARAM_PITCHEG_DECAY as usize] = pitch_eg_decay;
        Ok(self)
    }

    pub fn starting_point(&mut self, starting_point: u8) -> Result<&mut Self, SyroError> {
        check_starting_point(starting_point)?;
        self.data.Param[VOLCASAMPLE_PARAM_START_POINT as usize] = starting_point;
        Ok(self)
    }

    pub fn length(&mut self, length: u8) -> Result<&mut Self, SyroError> {
        check_starting_point(length)?;
        self.data.Param[VOLCASAMPLE_PARAM_LENGTH as usize] = length;
        Ok(self)
    }

    pub fn hi_cut(&mut self, hi_cut: u8) -> Result<&mut Self, SyroError> {
        check_hi_cut(hi_cut)?;
        self.data.Param[VOLCASAMPLE_PARAM_HICUT as usize] = hi_cut;
        Ok(self)
    }

    /// Valid values in the sequence are 0-127
    pub fn level_start_motion_seq(&mut self, sequence: [u8; 16]) -> Result<&mut Self, SyroError> {
        sequence
            .iter()
            .map(|&v| check_level(v))
            .collect::<Result<(), SyroError>>()?;
        self.data.Motion[korg_syro_sys::VOLCASAMPLE_MOTION_LEVEL_0 as usize] = sequence;
        Ok(self)
    }

    /// Valid values in the sequence are 0-127
    pub fn level_end_motion_seq(&mut self, sequence: [u8; 16]) -> Result<&mut Self, SyroError> {
        sequence
            .iter()
            .map(|&v| check_level(v))
            .collect::<Result<(), SyroError>>()?;
        self.data.Motion[korg_syro_sys::VOLCASAMPLE_MOTION_LEVEL_1 as usize] = sequence;
        Ok(self)
    }

    /// Valid values in the sequence are 1-127
    pub fn pan_start_motion_seq(&mut self, sequence: [u8; 16]) -> Result<&mut Self, SyroError> {
        sequence
            .iter()
            .map(|&v| check_pan(v))
            .collect::<Result<(), SyroError>>()?;
        self.data.Motion[korg_syro_sys::VOLCASAMPLE_MOTION_PAN_0 as usize] = sequence;
        Ok(self)
    }

    /// Valid values in the sequence are 1-127
    pub fn pan_end_motion_seq(&mut self, sequence: [u8; 16]) -> Result<&mut Self, SyroError> {
        sequence
            .iter()
            .map(|&v| check_pan(v))
            .collect::<Result<(), SyroError>>()?;
        self.data.Motion[korg_syro_sys::VOLCASAMPLE_MOTION_PAN_1 as usize] = sequence;
        Ok(self)
    }

    /// Valid values in the sequence are 40-88 for semitones, and 129-255 for continuous
    pub fn speed_start_motion_seq(&mut self, sequence: [u8; 16]) -> Result<&mut Self, SyroError> {
        sequence
            .iter()
            .map(|&v| check_speed(v))
            .collect::<Result<(), SyroError>>()?;
        self.data.Motion[korg_syro_sys::VOLCASAMPLE_MOTION_SPEED_0 as usize] = sequence;
        Ok(self)
    }

    /// Valid values in the sequence are 40-88 for semitones, and 129-255 for continuous
    pub fn speed_end_motion_seq(&mut self, sequence: [u8; 16]) -> Result<&mut Self, SyroError> {
        sequence
            .iter()
            .map(|&v| check_speed(v))
            .collect::<Result<(), SyroError>>()?;
        self.data.Motion[korg_syro_sys::VOLCASAMPLE_MOTION_SPEED_1 as usize] = sequence;
        Ok(self)
    }

    /// Valid values in the sequence are 0-127
    pub fn amp_eg_attack_motion_seq(&mut self, sequence: [u8; 16]) -> Result<&mut Self, SyroError> {
        sequence
            .iter()
            .map(|&v| check_amp_eg_attack(v))
            .collect::<Result<(), SyroError>>()?;
        self.data.Motion[korg_syro_sys::VOLCASAMPLE_MOTION_AMPEG_ATTACK as usize] = sequence;
        Ok(self)
    }

    /// Valid values in the sequence are 0-127
    pub fn amp_eg_decay_motion_seq(&mut self, sequence: [u8; 16]) -> Result<&mut Self, SyroError> {
        sequence
            .iter()
            .map(|&v| check_amp_eg_decay(v))
            .collect::<Result<(), SyroError>>()?;
        self.data.Motion[korg_syro_sys::VOLCASAMPLE_MOTION_AMPEG_DECAY as usize] = sequence;
        Ok(self)
    }

    /// Valid values in the sequence are 1-127
    pub fn pitch_eg_int_motion_seq(&mut self, sequence: [u8; 16]) -> Result<&mut Self, SyroError> {
        sequence
            .iter()
            .map(|&v| check_pitch_eg_int(v))
            .collect::<Result<(), SyroError>>()?;
        self.data.Motion[korg_syro_sys::VOLCASAMPLE_MOTION_PITCHEG_INT as usize] = sequence;
        Ok(self)
    }

    /// Valid values in the sequence are 0-127
    pub fn pitch_eg_attack_motion_seq(
        &mut self,
        sequence: [u8; 16],
    ) -> Result<&mut Self, SyroError> {
        sequence
            .iter()
            .map(|&v| check_pitch_eg_attack(v))
            .collect::<Result<(), SyroError>>()?;
        self.data.Motion[korg_syro_sys::VOLCASAMPLE_MOTION_PITCHEG_ATTACK as usize] = sequence;
        Ok(self)
    }

    /// Valid values in the sequence are 0-127
    pub fn pitch_eg_decay_motion_seq(
        &mut self,
        sequence: [u8; 16],
    ) -> Result<&mut Self, SyroError> {
        sequence
            .iter()
            .map(|&v| check_pitch_eg_decay(v))
            .collect::<Result<(), SyroError>>()?;
        self.data.Motion[korg_syro_sys::VOLCASAMPLE_MOTION_PITCHEG_DECAY as usize] = sequence;
        Ok(self)
    }

    /// Valid values in the sequence are 0-127
    pub fn start_point_motion_seq(&mut self, sequence: [u8; 16]) -> Result<&mut Self, SyroError> {
        sequence
            .iter()
            .map(|&v| check_starting_point(v))
            .collect::<Result<(), SyroError>>()?;
        self.data.Motion[korg_syro_sys::VOLCASAMPLE_MOTION_START_POINT as usize] = sequence;
        Ok(self)
    }

    /// Valid values in the sequence are 0-127
    pub fn length_motion_seq(&mut self, sequence: [u8; 16]) -> Result<&mut Self, SyroError> {
        sequence
            .iter()
            .map(|&v| check_length(v))
            .collect::<Result<(), SyroError>>()?;
        self.data.Motion[korg_syro_sys::VOLCASAMPLE_MOTION_LENGTH as usize] = sequence;
        Ok(self)
    }

    /// Valid values in the sequence are 0-127
    pub fn hi_cut_motion_seq(&mut self, sequence: [u8; 16]) -> Result<&mut Self, SyroError> {
        sequence
            .iter()
            .map(|&v| check_hi_cut(v))
            .collect::<Result<(), SyroError>>()?;
        self.data.Motion[korg_syro_sys::VOLCASAMPLE_MOTION_HICUT as usize] = sequence;
        Ok(self)
    }

    pub fn build(self) -> Self {
        self
    }
}

/// Defines a pattern for the sequencer
#[derive(Clone, Debug, Default)]
pub struct Pattern {
    data: VolcaSample_Pattern_Data,
}

impl Pattern {
    pub fn with_part(&mut self, part_index: u8, part: Part) -> Result<&Self, SyroError> {
        check_part_index(part_index)?;
        self.data.Part[part_index as usize] = part.data;
        Ok(self)
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.data.Header.to_le_bytes());
        bytes.extend_from_slice(&self.data.DevCode.to_le_bytes());
        bytes.extend_from_slice(&self.data.Reserved);
        bytes.extend_from_slice(&self.data.ActiveStep.to_le_bytes());
        bytes.extend_from_slice(&self.data.Padding1);
        for part in self.data.Part.iter() {
            let mut part_bytes = vec![];
            part_bytes.extend_from_slice(&part.SampleNum.to_le_bytes());
            part_bytes.extend_from_slice(&part.StepOn.to_le_bytes());
            part_bytes.extend_from_slice(&part.Accent.to_le_bytes());
            part_bytes.extend_from_slice(&part.Reserved.to_le_bytes());
            part_bytes.extend_from_slice(&part.Level.to_le_bytes());
            part_bytes.extend_from_slice(&part.Param);
            part_bytes.extend_from_slice(&part.FuncMemoryPart.to_le_bytes());
            part_bytes.extend_from_slice(&part.Padding1);
            for motion in part.Motion.iter() {
                part_bytes.extend_from_slice(motion);
            }
            bytes.extend_from_slice(part_bytes.as_slice());
        }
        bytes.extend_from_slice(&self.data.Padding2);
        bytes.extend_from_slice(&self.data.Footer.to_le_bytes());
        bytes
    }
}

#[cfg(test)]
mod test {
    use super::Toggle::*;
    use super::*;
    use anyhow;
    use korg_syro_sys::VolcaSample_Pattern_Init;

    #[test]
    fn test_step() {
        let steps = Steps::builder()
            .on(Step::Three)
            .on(Step::Seven)
            .on(Step::Twelve)
            .build()
            .to_bytes();

        println!("{:#018b}", steps);

        assert_eq!(steps, 0b000100001000100);
    }

    #[test]
    fn test_part_builder() -> anyhow::Result<()> {
        let motion_seq: [u8; 16] = [
            1, 8, 16, 24, 32, 40, 48, 56, 64, 72, 80, 88, 96, 104, 112, 120,
        ];
        let continuous_speed_motion_seq: [u8; 16] = [
            129, 137, 145, 153, 161, 169, 177, 185, 193, 201, 209, 217, 225, 233, 241, 249,
        ];
        let semitone_speed_motion_seq: [u8; 16] = [
            40, 43, 46, 49, 52, 55, 58, 61, 64, 67, 70, 73, 76, 79, 82, 85,
        ];
        let _part = Part::for_sample(0)?
            .with_steps(
                Steps::builder()
                    .on(Step::Three)
                    .on(Step::Seven)
                    .on(Step::Thirteen)
                    .build(),
            )
            .level(42)?
            .pan(42)?
            .amp_eg_attack(42)?
            .amp_eg_decay(42)?
            .pitch_eg_attack(42)?
            .pitch_eg_int(42)?
            .pitch_eg_decay(42)?
            .starting_point(42)?
            .length(42)?
            .hi_cut(42)?
            .level_start_motion_seq(motion_seq.clone())?
            .level_end_motion_seq(motion_seq.clone())?
            .pan_start_motion_seq(motion_seq.clone())?
            .pan_end_motion_seq(motion_seq.clone())?
            .speed_start_motion_seq(continuous_speed_motion_seq.clone())?
            .speed_end_motion_seq(semitone_speed_motion_seq.clone())?
            .amp_eg_attack_motion_seq(motion_seq.clone())?
            .amp_eg_decay_motion_seq(motion_seq.clone())?
            .pitch_eg_int_motion_seq(motion_seq.clone())?
            .pitch_eg_attack_motion_seq(motion_seq.clone())?
            .pitch_eg_decay_motion_seq(motion_seq.clone())?
            .start_point_motion_seq(motion_seq.clone())?
            .length_motion_seq(motion_seq.clone())?
            .hi_cut_motion_seq(motion_seq.clone())?
            .motion(On)
            .looped(On)
            .reverb(On)
            .reverse(On)
            .mute(Off)
            .build();

        Ok(())
    }

    #[test]
    fn test_pattern_default() -> anyhow::Result<()> {
        let mut raw_bytes: Vec<u8> = vec![0; std::mem::size_of::<VolcaSample_Pattern_Data>()];
        unsafe {
            VolcaSample_Pattern_Init(raw_bytes.as_mut_ptr() as *mut VolcaSample_Pattern_Data);
        }

        let default_bytes = Pattern::default().to_bytes();

        assert_eq!(raw_bytes, default_bytes);
        Ok(())
    }

    #[test]
    fn test_pattern() -> anyhow::Result<()> {
        let mut pattern = Pattern::default();
        pattern.with_part(
            0u8,
            Part::for_sample(0)?
                .with_steps(
                    Steps::builder()
                        .on(Step::One)
                        .on(Step::Three)
                        .on(Step::Five)
                        .on(Step::Seven)
                        .build(),
                )
                .mute(Off)
                .build(),
        )?;

        let _data = pattern.to_bytes();
        Ok(())
    }
}
