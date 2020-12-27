//!
//! Rust bindings for the [KORG SYRO](https://github.com/korginc/volcasample) library for the Volca Sample.
//!
//!
//! Files for use with the [reset](SyroStream::reset) method
//! can be found here:
//!
//! [https://github.com/korginc/volcasample/tree/master/alldata](https://github.com/korginc/volcasample/tree/master/alldata)
//!
//! # Examples
//!
//! Add/erase samples
//!
//! ```no_run
//! use std::fs::File;
//! use std::io::BufWriter;
//! use korg_syro::SyroStream;
//! use wav;
//!
//! let mut syro_stream = SyroStream::default();
//!
//! syro_stream
//!     .add_sample(0, vec![], 44100, None)?
//!     .erase_sample(1)?;
//! let data = syro_stream.generate()?;
//!
//! // PCM data, 2 channels, 44.1kHz sample rate, 16 bit per sample
//! let header = wav::Header::new(1, 2, 44100, 16);
//!
//! let output = File::create("output.wav").unwrap();
//! wav::write(header, &wav::BitDepth::Sixteen(data), &mut BufWriter::new(output));
//! # Ok::<(), korg_syro::SyroError>(())
//! ```
//!
//! Reset from .alldata file
//!
//! ```no_run
//! use std::fs::File;
//! use std::io::BufWriter;
//! use korg_syro::SyroStream;
//! use wav;
//!
//! let input_data = std::fs::read("all_sample_preset.alldata").unwrap();
//! let data = SyroStream::reset(input_data, Some(16))?;
//!
//! // PCM data, 2 channels, 44.1kHz sample rate, 16 bit per sample
//! let header = wav::Header::new(1, 2, 44100, 16);
//!
//! let output = File::create("output.wav").unwrap();
//! wav::write(header, &wav::BitDepth::Sixteen(data), &mut BufWriter::new(output));
//! # Ok::<(), korg_syro::SyroError>(())
//! ```
use std::mem::MaybeUninit;

use array_init;
use byteorder::{ByteOrder, LittleEndian};
use korg_syro_sys as syro;
use thiserror::Error;

#[macro_use]
mod macros;
use macros::*;

pub mod pattern;

#[derive(Error, Debug, PartialEq)]
pub enum SyroError {
    #[error("invalid value {val} for '{name}', expected at least {} and at most {}", .lo, .hi)]
    OutOfBounds {
        val: u32,
        name: &'static str,
        lo: usize,
        hi: usize,
    },

    #[error("empty stream, provide at least one sample or pattern")]
    EmptyStream,

    #[error("unhandled SyroStatus {status:?}")]
    SyroStatus { status: syro::SyroStatus },
}

fn check_syro_status(status: syro::SyroStatus) -> Result<(), SyroError> {
    match status {
        syro::SyroStatus::Status_Success => Ok(()),
        // TODO probably implement individual errors for these
        // SyroStatus::Status_IllegalDataType
        // SyroStatus::Status_IllegalData
        // SyroStatus::Status_IllegalParameter
        // SyroStatus::Status_OutOfRange_Number
        // SyroStatus::Status_OutOfRange_Quality
        // SyroStatus::Status_NotEnoughMemory
        // SyroStatus::Status_InvalidHandle
        // SyroStatus::Status_NoData
        _ => Err(SyroError::SyroStatus { status }),
    }
}

max_check!(sample_index, 99);
bounds_check!(bit_depth, 8, 16);

// Encapsulates ownership of SyroData
struct SyroDataBundle {
    #[allow(dead_code)]
    data: Vec<u8>,
    syro_data: syro::SyroData,
}

impl SyroDataBundle {
    fn sample(
        index: u32,
        data_type: syro::SyroDataType,
        mut data: Vec<u8>,
        sample_rate: u32,
        bit_depth: u32,
    ) -> Self {
        let syro_data = syro::SyroData {
            DataType: data_type,
            pData: data.as_mut_ptr(),
            // the sample (0-99) or sequence pattern (0-9) number
            Number: index,
            // size of data to be converted (in bytes)
            Size: data.len() as u32,
            // The conversion bit depth. It can be set to 8-16. Seems unused when DataType = Sample_liner
            Quality: bit_depth,
            Fs: sample_rate,
            SampleEndian: korg_syro_sys::Endian::LittleEndian,
        };

        Self { data, syro_data }
    }

    fn erase(index: u32) -> Self {
        let syro_data = syro::SyroData {
            DataType: syro::SyroDataType::DataType_Sample_Erase,
            pData: 0 as *mut u8,
            Number: index,
            Size: 0,
            Quality: 0,
            Fs: 0,
            SampleEndian: korg_syro_sys::Endian::LittleEndian,
        };

        Self {
            data: vec![],
            syro_data,
        }
    }

    fn reset(mut data: Vec<u8>) -> Self {
        let syro_data = syro::SyroData {
            DataType: syro::SyroDataType::DataType_Sample_All,
            pData: data.as_mut_ptr(),
            Size: data.len() as u32,
            Number: 0,
            Quality: 0,
            Fs: 44100,
            SampleEndian: korg_syro_sys::Endian::LittleEndian,
        };

        Self { data, syro_data }
    }

    fn reset_compressed(mut data: Vec<u8>, bit_depth: u32) -> Self {
        let syro_data = syro::SyroData {
            DataType: syro::SyroDataType::DataType_Sample_AllCompress,
            pData: data.as_mut_ptr(),
            Size: data.len() as u32,
            Number: 0,
            Quality: bit_depth,
            Fs: 44100,
            SampleEndian: korg_syro_sys::Endian::LittleEndian,
        };

        Self { data, syro_data }
    }

    fn pattern(index: u32, mut data: Vec<u8>) -> Self {
        let syro_data = syro::SyroData {
            DataType: syro::SyroDataType::DataType_Pattern,
            pData: data.as_mut_ptr(),
            Number: index,
            Size: data.len() as u32,
            Quality: 0,
            Fs: 0,
            SampleEndian: korg_syro_sys::Endian::LittleEndian,
        };

        Self { data, syro_data }
    }

    fn data(&self) -> syro::SyroData {
        self.syro_data
    }
}

/// Builder struct for syrostream data.
///
/// Output from the [generate](SyroStream::generate) or
/// [reset](SyroStream::reset) methods is uncompressed PCM
/// data that can be used to write a .wav file.
pub struct SyroStream {
    samples: [Option<SyroDataBundle>; 100],
    patterns: [Option<SyroDataBundle>; 10],
}

impl Default for SyroStream {
    fn default() -> Self {
        Self {
            samples: array_init::array_init(|_| None),
            patterns: array_init::array_init(|_| None),
        }
    }
}

fn convert_data(data: Vec<i16>) -> Vec<u8> {
    let mut new_data: Vec<u8> = vec![0; data.len() * 2];
    LittleEndian::write_i16_into(data.as_slice(), new_data.as_mut_slice());
    new_data
}

impl SyroStream {
    /// Generate stream from a .alldata file
    pub fn reset(data: Vec<u8>, compression: Option<u32>) -> Result<Vec<i16>, SyroError> {
        let mut syro_stream = Self::default();
        let syro_data_bundle = match compression {
            Some(bit_depth) => {
                check_bit_depth(bit_depth as u8)?;
                SyroDataBundle::reset_compressed(data, bit_depth)
            }
            None => SyroDataBundle::reset(data),
        };
        match syro_stream.samples.get_mut(0) {
            Some(elem) => {
                *elem = Some(syro_data_bundle);
            }
            None => unreachable!(),
        }
        syro_stream.generate()
    }

    /// Add a sample at the given index
    ///
    /// The index must be in the range 0-99. If compression is desired it has to
    /// be in the range of 8-16 bits.
    ///
    ///_**Note**: there are currently no guards against using samples that are too large._
    pub fn add_sample(
        &mut self,
        index: u32,
        data: Vec<i16>,
        sample_rate: u32,
        compression: Option<u32>,
    ) -> Result<&mut Self, SyroError> {
        check_sample_index(index as u8)?;
        let data = convert_data(data);
        let bundle = match compression {
            Some(bit_depth) => {
                check_bit_depth(bit_depth as u8)?;
                SyroDataBundle::sample(
                    index,
                    syro::SyroDataType::DataType_Sample_Compress,
                    data,
                    sample_rate,
                    bit_depth,
                )
            }
            None => SyroDataBundle::sample(
                index,
                syro::SyroDataType::DataType_Sample_Liner,
                data,
                sample_rate,
                0,
            ),
        };
        match self.samples.get_mut(index as usize) {
            Some(elem) => *elem = Some(bundle),
            None => panic!("Index out of bounds, checking must have failed"),
        }
        Ok(self)
    }

    /// Erase the sample at the given index
    ///
    /// The index must be in the range 0-99
    pub fn erase_sample(&mut self, index: u32) -> Result<&mut Self, SyroError> {
        check_sample_index(index as u8)?;
        // TODO maybe refactor to remove the check function and just throw on None
        match self.samples.get_mut(index as usize) {
            Some(elem) => *elem = Some(SyroDataBundle::erase(index)),
            None => panic!("Index out of bounds, checking must have failed"),
        }
        Ok(self)
    }

    /// Add a Pattern at the given index
    ///
    /// The index must be in the range 0-9
    pub fn add_pattern(
        &mut self,
        index: usize,
        pattern: pattern::Pattern,
    ) -> Result<&mut Self, SyroError> {
        pattern::check_pattern_index(index as u8)?;
        let data = SyroDataBundle::pattern(index as u32, pattern.to_bytes());
        if let Some(elem) = self.patterns.get_mut(index) {
            *elem = Some(data);
        }
        Ok(self)
    }

    /// Generates the syro stream
    ///
    /// Ouptut is uncompressed PCM data
    pub fn generate(self) -> Result<Vec<i16>, SyroError> {
        let mut data: Vec<syro::SyroData> = Vec::with_capacity(110);

        for sample in self.samples.iter() {
            if let Some(bundle) = sample {
                data.push(bundle.data());
            }
        }

        for pattern in self.patterns.iter() {
            if let Some(bundle) = pattern {
                data.push(bundle.data());
            }
        }

        if data.len() == 0 {
            return Err(SyroError::EmptyStream);
        }

        // unsafe territory
        let syro_stream = {
            let (handle, num_frames) = init_syro_handle(data)?;
            let result = generate_syro_stream(handle, num_frames);
            free_syro_handle(handle)?;
            result
        }?;
        Ok(syro_stream)
    }
}

fn init_syro_handle(mut data: Vec<syro::SyroData>) -> Result<(syro::SyroHandle, u32), SyroError> {
    let mut num_frames = 0;

    let handle: syro::SyroHandle = unsafe {
        let mut handle: MaybeUninit<syro::SyroHandle> = MaybeUninit::uninit();

        let status = syro::SyroVolcaSample_Start(
            handle.as_mut_ptr(),
            data.as_mut_ptr(),
            data.len() as i32,
            0,
            &mut num_frames,
        );
        check_syro_status(status)?;

        handle.assume_init()
    };

    Ok((handle, num_frames))
}

fn free_syro_handle(handle: syro::SyroHandle) -> Result<(), SyroError> {
    unsafe {
        let status = korg_syro_sys::SyroVolcaSample_End(handle);
        check_syro_status(status)
    }
}

fn generate_syro_stream(handle: syro::SyroHandle, num_frames: u32) -> Result<Vec<i16>, SyroError> {
    let mut left: i16 = 0;
    let mut right: i16 = 0;
    let mut buffer = Vec::with_capacity(num_frames as usize * 2);
    for _ in 0..num_frames {
        unsafe {
            let status = syro::SyroVolcaSample_GetSample(handle, &mut left, &mut right);
            if status == syro::SyroStatus::Status_NoData {
                // TODO investigate why GetSample keeps returning NoData and if it's ok
            } else {
                check_syro_status(status)?;
            }
        }
        buffer.push(left);
        buffer.push(right);
    }

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pattern::*;
    use waver;

    // 0.5 second sine wave
    fn sine_wave() -> Vec<i16> {
        let mut wf = waver::Waveform::<i16>::new(44100.0);
        wf.superpose(waver::Wave {
            frequency: 440.0,
            ..Default::default()
        })
        .normalize_amplitudes();
        wf.iter().take(22050).collect()
    }

    #[test]
    fn out_of_bounds() {
        let mut syro_stream = SyroStream::default();
        let result = syro_stream.add_sample(100, vec![], 44100, None);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            SyroError::OutOfBounds {
                val: 100,
                name: "sample_index".into(),
                lo: 0,
                hi: 99
            }
        );
    }

    #[test]
    fn empty_syrostream() {
        let result = SyroStream::default().generate();
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), SyroError::EmptyStream);
    }

    #[test]
    fn basic() -> anyhow::Result<()> {
        let input_data: Vec<i16> = sine_wave();

        let mut syro_stream = SyroStream::default();

        syro_stream.add_sample(0, input_data, 44100, None)?;
        syro_stream.erase_sample(1)?;
        syro_stream.add_pattern(0, Pattern::default())?;

        let _output = syro_stream.generate()?;
        Ok(())
    }
}
