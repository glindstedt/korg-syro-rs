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
    VOLCASAMPLE_FUNC_REVERSE,
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

/// Defines a part of a sequence pattern
#[derive(Copy, Clone, Debug)]
pub struct Part {
    data: VolcaSample_Part_Data,
}

macro_rules! impl_param {
    ($i:ident) => {
        paste! {
            pub fn [<$i>](&mut self, [<$i>]: u8) -> Result<&mut Self, SyroError> {
                [<check_ $i>]([<$i>] as u32)?;
                Ok(self)
            }
        }
    };
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
        check_sample_index(sample_num as u32)?;
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
    impl_func_memory_part!(mute, VOLCASAMPLE_FUNC_MUTE);

    impl_param!(level);
    impl_param!(pan);
    impl_param!(amp_eg_attack);
    impl_param!(amp_eg_decay);
    impl_param!(pitch_eg_attack);
    impl_param!(pitch_eg_int);
    impl_param!(pitch_eg_decay);
    impl_param!(starting_point);
    impl_param!(length);
    impl_param!(hi_cut);

    pub fn speed(&mut self, speed: u8) -> Result<&mut Self, SyroError> {
        check_speed_semitone(speed as u32).or(check_speed_continuous(speed as u32))?;
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
        check_part_index(part_index as u32)?;
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
    fn test_part() -> anyhow::Result<()> {
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
            .motion(On)
            .looped(On)
            .reverb(On)
            .reverse(On)
            .mute(Off);
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
