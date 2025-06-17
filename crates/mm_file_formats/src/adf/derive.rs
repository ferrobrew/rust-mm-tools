use std::{
    any::{Any, TypeId},
    collections::HashMap,
    io::{Read, Seek, Write},
    sync::Arc,
};

use const_format::concatcp;
use thiserror::Error;

use mm_hashing::{hash_little32, HashString};

use crate::common::{ReaderExt, WriterExt};

pub trait AdfTypeInfo {
    const NAME: &str;
    const HASH: u32;
    const SIZE: u64;
    const ALIGN: u64;
}

macro_rules! type_name {
    ([$ty:ty; $n:expr]) => {
        concatcp!(
            "IA[",
            <$ty as AdfTypeInfo>::NAME,
            "]4",
            <$ty as AdfTypeInfo>::SIZE * $n,
            <$ty as AdfTypeInfo>::ALIGN
        )
    };
    ([$ty:ty]) => {
        concatcp!("A[", <$ty as AdfTypeInfo>::NAME, "]3168")
    };
    (&$ty:ty) => {
        concatcp!(<$ty as AdfTypeInfo>::NAME, "*288")
    };
}

macro_rules! type_hash {
    // CommonHash ^ ElementHash ^ hash_little32(Length)
    ([$ty:ty; $n:expr]) => {
        hash_little32(
            concatcp!(
                hash_little32(type_name!([$ty; $n]).as_bytes()),
                <$ty as AdfTypeInfo>::HASH,
                concat!($n),
            )
            .as_bytes(),
        )
    };
    // CommonHash ^ ElementHash
    ([$ty:ty]) => {
        hash_little32(
            concatcp!(
                hash_little32(type_name!([$ty]).as_bytes()),
                <$ty as AdfTypeInfo>::HASH,
            )
            .as_bytes(),
        )
    };
    // CommonHash ^ ElementHash
    (&$ty:ty) => {
        hash_little32(
            concatcp!(
                hash_little32(type_name!(&$ty).as_bytes()),
                <$ty as AdfTypeInfo>::HASH,
            )
            .as_bytes(),
        )
    };
    // CommonHash
    ($n:expr, $t:expr, $s:expr, $a:expr) => {
        hash_little32(concat!($n, stringify!($t), stringify!($s), stringify!($a)).as_bytes())
    };
}

macro_rules! type_info {
    ($ty:ty, $($l:expr)+) => {
        type_info!(&$ty);
        type_info!([$ty]);
        $(type_info!([$ty; $l]);)+
    };
    (dyn $ty:tt, $n:expr, $t:expr, $s:expr, $a:expr) => {
        impl AdfTypeInfo for dyn $ty {
            const NAME: &str = $n;
            const HASH: u32 = type_hash!($n, $t, $s, $a);
            const SIZE: u64 = $s;
            const ALIGN: u64 = $s;
        }
    };
    ($ty:ty, $n:expr, $t:expr, $s:expr, $a:expr) => {
        impl AdfTypeInfo for $ty {
            const NAME: &str = $n;
            const HASH: u32 = type_hash!($n, $t, $s, $a);
            const SIZE: u64 = $s;
            const ALIGN: u64 = $a;
        }
        type_info!($ty, 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32);
    };
    (&$ty:ty) => {
        impl AdfTypeInfo for Option<Arc<$ty>> {
            const NAME: &str = type_name!(&$ty);
            const HASH: u32 = type_hash!(&$ty);
            const SIZE: u64 = 8;
            const ALIGN: u64 = 8;
        }
    };
    ([$ty:ty]) => {
        impl AdfTypeInfo for Arc<Vec<$ty>> {
            const NAME: &str = type_name!([$ty]);
            const HASH: u32 = type_hash!([$ty]);
            const SIZE: u64 = 16;
            const ALIGN: u64 = 8;
        }
    };
    ([$ty:ty; $l:expr]) => {
        impl AdfTypeInfo for [$ty; $l] {
            const NAME: &str = type_name!([$ty]);
            const HASH: u32 = type_hash!([$ty; $l]);
            const SIZE: u64 = <$ty as AdfTypeInfo>::SIZE * $l;
            const ALIGN: u64 = <$ty as AdfTypeInfo>::ALIGN;
        }
    };
}

type_info!(u8, "uint8", 0, 1, 1);
type_info!(i8, "int8", 0, 1, 1);
type_info!(u16, "uint16", 0, 2, 2);
type_info!(i16, "int16", 0, 2, 2);
type_info!(u32, "uint32", 0, 4, 4);
type_info!(i32, "int32", 0, 4, 4);
type_info!(f32, "float", 0, 4, 4);
type_info!(u64, "uint64", 0, 8, 8);
type_info!(i64, "int64", 0, 8, 8);
type_info!(f64, "double", 0, 8, 8);
type_info!(Arc<String>, "String", 5, 8, 8);
type_info!(dyn Any, "void", 10, 16, 6);

macro_rules! const_assert {
    ($($tt:tt)*) => {
        const _: () = assert!($($tt)*);
    }
}

const_assert!(<Option<Arc<u32>> as AdfTypeInfo>::HASH == 1283401978);
const_assert!(<f32 as AdfTypeInfo>::HASH == 0x7515A207);
const_assert!(<[f32; 3] as AdfTypeInfo>::HASH == 0xE8541F6E);
const_assert!(<Arc<Vec<f32>> as AdfTypeInfo>::HASH == 0x168B4EB8);

pub type AdfReaderReferences = HashMap<u64, Box<dyn Any>>;
pub type AdfWriterReferences = (u64, HashMap<usize, (u64, TypeId)>);

pub trait AdfRead: Sized {
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError>;
}

pub trait AdfWrite: Sized {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError>;
}

impl<T: AdfRead + AdfTypeInfo + Default + Copy + 'static, const S: usize> AdfRead for [T; S] {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        let mut result = [Default::default(); S];
        reader.align(T::ALIGN)?;
        for i in 0..S {
            result[i] = T::read(reader, references)?;
        }
        Ok(result)
    }
}

impl<T: AdfWrite + AdfTypeInfo + 'static, const S: usize> AdfWrite for [T; S] {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        writer.align(T::ALIGN)?;
        for value in self.iter() {
            value.write(writer, references)?;
        }
        Ok(())
    }
}

impl<T: AdfRead + AdfTypeInfo + 'static> AdfRead for Option<Arc<T>> {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        let offset = u64::read(reader, references)?;
        let position = reader.stream_position()?;
        match offset {
            0 => Ok(None),
            offset => {
                if let Some(reference) = references.get(&offset) {
                    reference
                        .downcast_ref()
                        // If the reference type is correct, clone it
                        .map(|x: &Box<Arc<T>>| Some(x.as_ref().clone()))
                        // Otherwise throw a reference error
                        .ok_or_else(|| AdfReadWriteError::ReferenceError {
                            expected: TypeId::of::<Arc<T>>(),
                            position: position - T::ALIGN,
                        })
                } else {
                    // Seek to offset
                    reader.seek_absolute(offset)?;

                    // Read back one instance of `T`, and store a reference
                    let result = Arc::new(T::read(reader, references)?);
                    references.insert(position, Box::from(result.clone()));

                    // Return to position
                    reader.seek_absolute(position)?;
                    Ok(Some(result))
                }
            }
        }
    }
}

impl<T: AdfWrite + AdfTypeInfo + 'static> AdfWrite for Option<Arc<T>> {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        match self {
            Some(value) => {
                let key = Arc::as_ptr(value) as usize;
                let type_id = TypeId::of::<Arc<T>>();
                if let Some(reference) = references.1.get(&key).cloned() {
                    if reference.1 == type_id {
                        // If the reference type is correct, write it's offset
                        reference.0.write(writer, references)
                    } else {
                        // Otherwise throw a reference error
                        Err(AdfReadWriteError::ReferenceError {
                            expected: type_id,
                            position: writer.stream_position()?,
                        })
                    }
                } else {
                    // Take note of position, seek to tail
                    let position = writer.stream_position()?;
                    writer.seek_absolute(references.0)?;

                    // Align writer, and take note of offset
                    writer.align(T::ALIGN.max(16))?;
                    let offset = writer.stream_position()?;

                    // Write zeroes, and update tail position
                    writer.pad(T::SIZE)?;
                    references.1.insert(key, (offset, type_id));
                    references.0 = writer.stream_position()?;

                    // Restore offset, and write value
                    writer.seek_absolute(offset)?;
                    value.write(writer, references)?;

                    // Restore position, and write offset
                    writer.seek_absolute(position)?;
                    offset.write(writer, references)?;
                    Ok(())
                }
            }
            None => {
                0u64.write(writer, references)?;
                Ok(())
            }
        }
    }
}

impl<T: AdfRead + AdfTypeInfo + 'static> AdfRead for Arc<Vec<T>> {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        let offset = u64::read(reader, references)?;
        let count = u64::read(reader, references)? as usize;
        let position = reader.stream_position()?;
        match offset {
            0 => Ok(Arc::new(Vec::default())),
            offset => {
                if let Some(reference) = references.get(&offset) {
                    reference
                        .downcast_ref()
                        // If the reference type is correct, clone it
                        .map(|x: &Box<Arc<Vec<T>>>| x.as_ref().clone())
                        // Otherwise throw a reference error
                        .ok_or_else(|| AdfReadWriteError::ReferenceError {
                            expected: TypeId::of::<Arc<Vec<T>>>(),
                            position: position - T::ALIGN,
                        })
                } else {
                    // Seek to offset
                    reader.seek_absolute(offset)?;

                    // Read back `count` number of `T`
                    let mut result = Vec::with_capacity(count);
                    for _ in 0..count {
                        result.push(T::read(reader, references)?);
                    }

                    // Store a reference
                    let result = Arc::new(result);
                    references.insert(position, Box::from(result.clone()));

                    // Return to initial writer position
                    reader.seek_absolute(position)?;
                    Ok(result)
                }
            }
        }
    }
}

impl<T: AdfWrite + AdfTypeInfo + 'static> AdfWrite for Arc<Vec<T>> {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        match self.is_empty() {
            false => {
                let key = Arc::as_ptr(self) as usize;
                let type_id = TypeId::of::<Arc<Vec<T>>>();
                if let Some(reference) = references.1.get(&key).cloned() {
                    if reference.1 == type_id {
                        // If the reference type is correct, write it's offset
                        reference.0.write(writer, references)
                    } else {
                        // Otherwise throw a reference error
                        Err(AdfReadWriteError::ReferenceError {
                            expected: type_id,
                            position: writer.stream_position()?,
                        })
                    }
                } else {
                    // Take note of position, seek to tail
                    let position = writer.stream_position()?;
                    writer.seek_absolute(references.0)?;

                    // Align writer, and take note of offset
                    let offset = writer.align(T::ALIGN.max(16))?;

                    // Write zeroes, and update tail position
                    let count = self.len() as u64;
                    writer.pad(T::SIZE * count)?;
                    references.1.insert(key, (offset, type_id));
                    references.0 = writer.stream_position()?;

                    // Restore offset, and write values
                    writer.seek_absolute(offset)?;
                    for value in self.iter() {
                        value.write(writer, references)?;
                    }

                    // Restore position, and write offset + count
                    writer.seek_absolute(position)?;
                    offset.write(writer, references)?;
                    count.write(writer, references)?;
                    Ok(())
                }
            }
            true => {
                0u64.write(writer, references)?;
                0u64.write(writer, references)?;
                Ok(())
            }
        }
    }
}

impl AdfRead for Arc<String> {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        let offset = u64::read(reader, references)?;
        let position = reader.stream_position()?;
        if let Some(reference) = references.get(&offset) {
            reference
                .downcast_ref()
                // If the reference type is correct, clone it
                .map(|x: &Box<Arc<String>>| x.as_ref().clone())
                // Otherwise throw a reference error
                .ok_or_else(|| AdfReadWriteError::ReferenceError {
                    expected: TypeId::of::<Arc<String>>(),
                    position: position - Arc::<String>::ALIGN,
                })
        } else {
            // Seek to offset
            reader.seek_absolute(offset)?;

            // Read back the string
            let mut buffer = Vec::with_capacity(128);
            loop {
                let char = u8::read(reader, references)?;
                if char == 0 {
                    break;
                }
                buffer.push(char);
            }

            // Store a reference
            let result = Arc::new(String::from_utf8_lossy(&buffer).to_string());
            references.insert(position, Box::from(result.clone()));

            // Return to initial writer position
            reader.seek_absolute(position)?;
            Ok(result)
        }
    }
}

impl AdfWrite for Arc<String> {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        let key = Arc::as_ptr(self) as usize;
        let type_id = TypeId::of::<Arc<String>>();
        if let Some(reference) = references.1.get(&key).cloned() {
            if reference.1 == type_id {
                // If the reference type is correct, write it's offset
                reference.0.write(writer, references)
            } else {
                // Otherwise throw a reference error
                Err(AdfReadWriteError::ReferenceError {
                    expected: type_id,
                    position: writer.stream_position()?,
                })
            }
        } else {
            // Write tail position, take note of offset + position
            let offset = references.0;
            offset.write(writer, references)?;
            let position = writer.stream_position()?;

            // Seek to tail position, write data, update tail
            writer.seek_absolute(references.0)?;
            writer.write_all(self.as_bytes())?;
            writer.write_all(&[0u8])?;
            references.0 = writer.stream_position()?;

            // Seek to position
            writer.seek_absolute(position)?;
            Ok(())
        }
    }
}

macro_rules! read_write_scalar {
    ($ty:ty) => {
        impl AdfRead for $ty {
            #[inline]
            fn read<R: Read + Seek>(
                reader: &mut R,
                _references: &mut AdfReaderReferences,
            ) -> Result<Self, AdfReadWriteError> {
                let mut result = Self::default();
                reader.align(Self::ALIGN)?;
                reader.read_exact(bytemuck::bytes_of_mut(&mut result))?;
                Ok(result)
            }
        }

        impl AdfWrite for $ty {
            #[inline]
            fn write<W: Write + Seek>(
                &self,
                writer: &mut W,
                _references: &mut AdfWriterReferences,
            ) -> Result<(), AdfReadWriteError> {
                writer.align(Self::ALIGN)?;
                writer.write(bytemuck::bytes_of(self))?;
                Ok(())
            }
        }
    };
}

read_write_scalar!(u8);
read_write_scalar!(i8);
read_write_scalar!(u16);
read_write_scalar!(i16);
read_write_scalar!(u32);
read_write_scalar!(i32);
read_write_scalar!(f32);
read_write_scalar!(u64);
read_write_scalar!(i64);
read_write_scalar!(f64);

impl AdfTypeInfo for HashString {
    const NAME: &str = "StringHash_48c5294d_4";
    const HASH: u32 = 3225380031;
    const SIZE: u64 = 4;
    const ALIGN: u64 = 4;
}

impl AdfRead for HashString {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        _references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        let mut result = Self::default();
        reader.align(Self::ALIGN)?;
        reader.read_exact(bytemuck::bytes_of_mut(result.hash_mut()))?;
        Ok(result)
    }
}

impl AdfWrite for HashString {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        _references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        writer.align(Self::ALIGN)?;
        writer.write(bytemuck::bytes_of(&self.hash()))?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum AdfReadWriteError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("reference error, expected: {expected:?}, at: {position}")]
    ReferenceError { expected: TypeId, position: u64 },
    #[error("invalid alignment, expected: {expected}, at: {position}")]
    Alignment { expected: u64, position: u64 },
}
