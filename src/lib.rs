use std::mem::MaybeUninit;
use std::collections::HashMap;

use byteorder::{ByteOrder, LittleEndian};
use thiserror::Error;
use korg_syro_sys as syro;

#[derive(Error, Debug, PartialEq)]
pub enum SyroError {
    #[error("invalid value {val} for '{name}', expected at least {} and at most {}", .lo, .hi)]
    OutOfBounds { val: u32, name: &'static str, lo: usize, hi: usize },

    #[error("unhandled SyroStatus {status:?}")]
    SyroStatus {
        status: syro::SyroStatus
    }
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
        _ => Err(SyroError::SyroStatus {status})
    }
}

// Encapsulates ownership of SyroData
struct SyroDataBundle {
    data: Vec<u8>,
    syro_data: syro::SyroData
}

impl SyroDataBundle {
    fn sample(index: u32, data_type: syro::SyroDataType, mut data: Vec<u8>, sample_rate: u32, bit_depth: u32) -> Self {
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

        Self {
            data,
            syro_data,
        }
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

        Self {
            data,
            syro_data,
        }
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

        Self {
            data,
            syro_data,
        }
    }

    fn data(&self) -> syro::SyroData {
        self.syro_data
    }
}

#[derive(Default)]
pub struct SyroStream {
    bundles: HashMap<u32, SyroDataBundle>
}

const INDEX_ERROR_NAME: &'static str = "index";
fn check_sample_index(index: u32) -> Result<(), SyroError> {
    if index > 99 {
        return Err(SyroError::OutOfBounds { val: index, name: INDEX_ERROR_NAME, lo: 0, hi: 99 })
    }
    Ok(())
}

const BIT_DEPTH_ERROR_NAME: &'static str = "bit_depth";
fn check_bit_depth(bit_depth: u32) -> Result<(), SyroError> {
    if bit_depth < 8 || bit_depth > 16 {
        return Err(SyroError::OutOfBounds { val: bit_depth, name: BIT_DEPTH_ERROR_NAME, lo: 8, hi: 16 })
    }
    Ok(())
}

fn convert_data(data: Vec<i16>) -> Vec<u8> {
    let mut new_data: Vec<u8> = vec![0; data.len() * 2];
    LittleEndian::write_i16_into(data.as_slice(), new_data.as_mut_slice());
    new_data
}

impl SyroStream {

    // FIXME currently super slow, probably due to unnecessary copying
    // Factory-reset from .alldata file
    pub fn reset(data: Vec<u8>, compression: Option<u32>) -> Result<Vec<i16>, SyroError> {
        let mut syro_stream = Self::default();
        match compression {
            Some(bit_depth) => {
                check_bit_depth(bit_depth)?;
                syro_stream.bundles.insert(0, SyroDataBundle::reset_compressed(data, bit_depth));
            }
            None => {
                syro_stream.bundles.insert(0, SyroDataBundle::reset(data));
            }
        }
        syro_stream.generate()
    }

    pub fn add_sample(&mut self, index: u32, data: Vec<i16>, sample_rate: u32, compression: Option<u32>) -> Result<&Self, SyroError> {
        check_sample_index(index)?;
        let data = convert_data(data);
        let bundle = match compression {
            Some(bit_depth) => {
                check_bit_depth(bit_depth)?;
                SyroDataBundle::sample(index, syro::SyroDataType::DataType_Sample_Compress, data, sample_rate, bit_depth)
            },
            None => SyroDataBundle::sample(index, syro::SyroDataType::DataType_Sample_Liner, data, sample_rate, 0),
        };
        self.bundles.insert(index, bundle);
        Ok(self)
    }

    pub fn erase_sample(&mut self, index: u32) -> Result<&Self, SyroError> {
        check_sample_index(index)?;
        self.bundles.insert(index, SyroDataBundle::erase(index));
        Ok(self)
    }

    pub fn generate(self) -> Result<Vec<i16>, SyroError> {
        let data: Vec<syro::SyroData> = self.bundles.iter()
            .map(|(_, v)| v.data())
            .collect();

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
        let mut handle: MaybeUninit::<syro::SyroHandle> = MaybeUninit::uninit();

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
            // TODO investigate why GetSample keeps throwing NoData
            match check_syro_status(status) {
                Err(SyroError::SyroStatus { status }) => {
                    if status == syro::SyroStatus::Status_NoData {
                        // ignore NoData as it seems normal?
                        Ok(())
                    } else {
                        Err(SyroError::SyroStatus { status })
                    }
                }
                result => result
            }?;
        }
        buffer.push(left);
        buffer.push(right);
    }

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use waver;

    // 0.5 second sine wave
    fn sine_wave() -> Vec<i16> {
        let mut wf = waver::Waveform::<i16>::new(44100.0);
        wf.superpose(waver::Wave { frequency: 440.0, ..Default::default() })
            .normalize_amplitudes();
        wf.iter().take(22050).collect()
    }

    #[test]
    fn out_of_bounds() {
        let mut syro_stream = SyroStream::default();
        let result = syro_stream.add_sample(100, vec![], 44100, None);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), SyroError::OutOfBounds { val: 100, name: "index".into(), lo: 0, hi: 99});
    }

    #[test]
    fn basic() -> anyhow::Result<(), anyhow::Error> {
        let input_data: Vec<i16> = sine_wave();

        let mut syro_stream = SyroStream::default();

        syro_stream.add_sample(0, input_data, 44100, None)?;
        syro_stream.erase_sample(1)?;

        let output = syro_stream.generate()?;
        Ok(())
    }
}
