use binrw::BinWrite;

pub mod length;
pub use length::*;

pub mod null_string;
pub use null_string::*;

pub trait WriterExt {
    fn pad(&mut self, padding: u64) -> binrw::BinResult<u64>;
    fn align(&mut self, alignment: u64) -> binrw::BinResult<u64>;
}

impl<T: std::io::Write + std::io::Seek> WriterExt for T {
    #[inline]
    fn pad(&mut self, padding: u64) -> binrw::BinResult<u64> {
        for _ in 0..padding {
            0u8.write_le(self)?;
        }
        Ok(self.stream_position()?)
    }

    #[inline]
    fn align(&mut self, alignment: u64) -> binrw::BinResult<u64> {
        #[inline(always)]
        const fn align(value: u64, alignment: u64) -> u64 {
            let align = alignment - 1;
            (value + align) & !align
        }
        let position = self.stream_position()?;
        let aligned = align(position, alignment);
        Ok(self.pad(aligned - position)?)
    }
}
