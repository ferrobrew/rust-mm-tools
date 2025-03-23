pub mod length;
pub use length::*;

pub mod null_string;
pub use null_string::*;

#[inline(always)]
const fn align(value: u64, alignment: u64) -> u64 {
    let align = alignment - 1;
    (value + align) & !align
}

pub trait ReaderExt {
    fn skip(&mut self, padding: u64) -> Result<u64, std::io::Error>;
    fn align(&mut self, alignment: u64) -> Result<u64, std::io::Error>;
    fn seek_absolute(&mut self, position: u64) -> Result<u64, std::io::Error>;
}

impl<T: std::io::Read + std::io::Seek> ReaderExt for T {
    #[inline]
    fn skip(&mut self, padding: u64) -> Result<u64, std::io::Error> {
        self.seek_relative(padding as i64)?;
        Ok(self.stream_position()?)
    }

    #[inline]
    fn align(&mut self, alignment: u64) -> Result<u64, std::io::Error> {
        let position = self.stream_position()?;
        let aligned = align(position, alignment);
        Ok(self.skip(aligned - position)?)
    }

    fn seek_absolute(&mut self, position: u64) -> Result<u64, std::io::Error> {
        Ok(self.seek(std::io::SeekFrom::Start(position))?)
    }
}

pub trait WriterExt {
    fn pad(&mut self, padding: u64) -> Result<u64, std::io::Error>;
    fn align(&mut self, alignment: u64) -> Result<u64, std::io::Error>;
    fn seek_absolute(&mut self, position: u64) -> Result<u64, std::io::Error>;
}

impl<T: std::io::Write + std::io::Seek> WriterExt for T {
    #[inline]
    fn pad(&mut self, padding: u64) -> Result<u64, std::io::Error> {
        for _ in 0..padding {
            self.write(&[0])?;
        }
        Ok(self.stream_position()?)
    }

    #[inline]
    fn align(&mut self, alignment: u64) -> Result<u64, std::io::Error> {
        let position = self.stream_position()?;
        let aligned = align(position, alignment);
        Ok(self.pad(aligned - position)?)
    }

    fn seek_absolute(&mut self, position: u64) -> Result<u64, std::io::Error> {
        Ok(self.seek(std::io::SeekFrom::Start(position))?)
    }
}
