use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use aligned_vec::{avec_rt, AVec, RuntimeAlign};
use binrw::{binrw, BinRead, BinWrite};
use bitflags::bitflags;
use mm_hashing::{hash_little32, HashString};
use modular_bitfield::{
    bitfield,
    prelude::{B24, B8},
};
use thiserror::Error;

use crate::common::{LengthVec, NullString, WriterExt};

#[derive(Clone, Debug, Default)]
pub struct AdfFile {
    pub version: AdfVersion,
    pub instances: Vec<Arc<AdfInstance>>,
    pub types: Vec<AdfType>,
    pub hashes: Vec<HashString>,
    pub description: NullString,
}

impl AdfFile {
    pub fn get_type_by_hash(&self, type_hash: u32) -> Option<&AdfType> {
        self.types
            .iter()
            .find(|type_def| type_def.type_hash == type_hash)
    }

    pub fn get_type_by_name(&self, name: &impl AsRef<str>) -> Option<&AdfType> {
        let name = name.as_ref();
        self.types
            .iter()
            .find(|type_def| type_def.name.as_ref() == name)
    }

    pub fn get_instance(
        &self,
        name: &impl AsRef<str>,
        type_def: &AdfType,
    ) -> Option<Arc<AdfInstance>> {
        let name = name.as_ref();
        let type_hash = type_def.type_hash;
        self.instances
            .iter()
            .find(|inst| inst.type_hash == type_hash && inst.name.as_ref() == name)
            .cloned()
    }

    // TODO: If an instance already exists, we should return an error containing the existing instance
    pub fn new_instance(
        &mut self,
        name: &impl AsRef<str>,
        type_def: &AdfType,
    ) -> Option<Arc<AdfInstance>> {
        if let Some(instance) = self.get_instance(name, type_def) {
            Some(instance)
        } else {
            self.instances.push(Arc::new(AdfInstance {
                name: NullString::from(name.as_ref()).into(),
                type_hash: type_def.type_hash,
                buffer: Mutex::new(
                    avec_rt!([type_def.alignment as usize]| 0u8; type_def.size as usize).into(),
                ),
            }));
            self.instances.last().cloned()
        }
    }

    pub fn remove_instance(&mut self, instance_def: &Arc<AdfInstance>) -> bool {
        let position = self
            .instances
            .iter()
            .position(|i| Arc::ptr_eq(i, instance_def));

        if let Some(index) = position {
            self.instances.swap_remove(index);
            true
        } else {
            false
        }
    }
}

impl BinRead for AdfFile {
    type Args<'a> = ();

    #[inline]
    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        use std::io::SeekFrom::Start;

        let header = AdfHeader::read_options(reader, endian, ())?;

        // Read back array of strings
        let mut strings: Vec<NullString> = Vec::with_capacity(header.string_count as usize);
        if header.string_offset != 0 {
            reader.seek(Start((header.string_offset) as u64))?;

            let mut lengths = vec![0u8; header.string_count as usize];
            reader.read_exact(lengths.as_mut_slice())?;

            for length in lengths {
                let string = NullString::read_options(reader, endian, ())?;
                if string.len() != length as usize {
                    return Err(binrw::Error::AssertFail {
                        pos: reader.stream_position()?,
                        message: format!(
                            "{string:?} was expected to be {length:?} bytes in length"
                        ),
                    });
                }
                strings.push(string);
            }
        }
        let strings = AdfReferenceCollector::<NullString>::new(strings.into());

        // Read back array of instances
        let mut instances: Vec<Arc<AdfInstance>> =
            Vec::with_capacity(header.instance_count as usize);
        if header.instance_offset != 0 {
            reader.seek(Start((header.instance_offset) as u64))?;

            for _ in 0..header.instance_count {
                instances.push(AdfInstance::read_options(reader, endian, (&strings,))?.into());
            }
        }
        let instances = AdfReferenceCollector::<Arc<AdfInstance>>::new(instances.into());

        // Read back array of hashes
        let mut hashes: Vec<HashString> = Vec::with_capacity(header.hash_count as usize);
        if header.hash_offset != 0 {
            reader.seek(Start((header.hash_offset) as u64))?;

            for _ in 0..header.hash_count {
                hashes.push(HashString::read_options(reader, endian, ())?);
            }
        }

        // Read back array of types
        let mut types: Vec<AdfType> = Vec::with_capacity(header.type_count as usize);
        if header.type_offset != 0 {
            reader.seek(Start((header.type_offset) as u64))?;

            for _ in 0..header.type_count {
                types.push(AdfType::read_options(
                    reader,
                    endian,
                    (&strings, &instances),
                )?);
            }
        }

        Ok(AdfFile {
            version: header.version,
            types,
            instances: instances.take(),
            hashes,
            description: header.description,
        })
    }
}

impl BinWrite for AdfFile {
    type Args<'a> = ();

    #[inline]
    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        use std::io::SeekFrom::Start;

        // Write initial header values
        let header_offset = writer.stream_position()?;
        let mut header = AdfHeader::default();
        header.description = self.description.clone();
        header.write_options(writer, endian, ())?;

        let strings = AdfReferenceCollector::<NullString>::default();
        let instances = AdfReferenceCollector::<Arc<AdfInstance>>::from(std::cell::RefCell::new(
            self.instances.clone(),
        ));

        // Write types
        header.type_count = self.types.len() as u32;
        if header.type_count > 0 {
            header.type_offset = writer.align(16)? as u32;
            for adf_type in &self.types {
                adf_type.write_options(writer, endian, (&strings, &instances))?;
            }
        }

        // Write instances
        header.instance_count = instances.borrow().len() as u32;
        if header.instance_count > 0 {
            header.instance_offset = writer.align(16)? as u32;
            for _ in 0..instances.borrow().len() {
                [0u64; 3].write_options(writer, endian, ())?;
            }

            // TODO: instances are 128 byte aligned, needs corrected before writing, requires &impl Iter<Item = &AdfFile> to be passed with *all referenced* types
            let mut buffer_offset = writer.stream_position()?;
            writer.seek(Start(header.instance_offset as u64))?;
            for adf_instance in instances.borrow().iter() {
                adf_instance.write_options(writer, endian, (&strings, &mut buffer_offset))?;
            }
            writer.seek(Start(buffer_offset))?;
        }

        // Write strings
        header.string_count = strings.borrow().len() as u32;
        if header.string_count > 0 {
            header.string_offset = writer.align(16)? as u32;
            for string in strings.borrow().iter() {
                (string.len() as u8).write_options(writer, endian, ())?;
            }
            for string in strings.borrow().iter() {
                string.write_options(writer, endian, ())?;
            }
        }

        // TODO: write hashes?

        // Write final header
        header.file_size = writer.stream_position()? as u32;
        writer.seek(Start(header_offset))?;
        header.write_options(writer, endian, ())?;
        Ok(())
    }
}

#[binrw]
#[brw(repr = u32)]
#[derive(Clone, Copy, Debug, Default)]
pub enum AdfVersion {
    #[default]
    V4 = 4,
}

#[binrw]
#[brw(magic = b" FDA")]
#[derive(Clone, Debug, Default)]
struct AdfHeader {
    pub version: AdfVersion,
    pub instance_count: u32,
    pub instance_offset: u32,
    pub type_count: u32,
    pub type_offset: u32,
    pub hash_count: u32,
    pub hash_offset: u32,
    pub string_count: u32,
    pub string_offset: u32,
    pub file_size: u32,
    #[brw(pad_before = 20)]
    pub description: NullString,
}

#[derive(Debug)]
pub struct AdfInstance {
    pub name: AdfReference<NullString>,
    pub type_hash: u32,
    pub buffer: Mutex<AVec<u8, RuntimeAlign>>,
}

impl Default for AdfInstance {
    fn default() -> Self {
        Self {
            name: Default::default(),
            type_hash: Default::default(),
            buffer: Mutex::from(AVec::new(0)),
        }
    }
}

impl PartialEq for AdfInstance {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.type_hash == other.type_hash
            && (&raw const self.buffer == &raw const other.buffer)
    }
}

impl BinRead for AdfInstance {
    type Args<'a> = (&'a AdfReferenceCollector<NullString>,);

    #[inline]
    fn read_options<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::prelude::BinResult<Self> {
        let name_hash = u32::read_options(reader, endian, ())?;
        let type_hash = u32::read_options(reader, endian, ())?;
        let buffer_offset = u32::read_options(reader, endian, ())? as u64;
        let buffer_size = u32::read_options(reader, endian, ())? as usize;
        let name = AdfReference::<NullString>::read_options(reader, endian, (args.0,))?;

        let position = reader.stream_position()?;
        reader.seek(std::io::SeekFrom::Start(buffer_offset))?;

        let mut buffer = avec_rt!([128]| 0u8; buffer_size);
        for byte in buffer.iter_mut() {
            *byte = u8::read_options(reader, endian, ())?;
        }

        reader.seek(std::io::SeekFrom::Start(position))?;

        if name_hash == hash_little32(name.as_bytes()) {
            Ok(Self {
                name,
                type_hash,
                buffer: buffer.into(),
            })
        } else {
            Err(binrw::Error::Custom {
                pos: reader.stream_position()?,
                err: Box::new(AdfInstanceError::InvalidNameHash(name_hash)),
            })
        }
    }
}

impl BinWrite for AdfInstance {
    type Args<'a> = (&'a AdfReferenceCollector<NullString>, &'a mut u64);

    #[inline]
    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        use std::io::SeekFrom::Start;

        // Remember our instance offset
        let instance_offset = writer.stream_position()?;

        if let Ok(buffer) = self.buffer.lock() {
            // Seek to aligned buffer offset + write buffer
            writer.seek(Start(*args.1))?;
            let buffer_offset = writer.align(buffer.alignment() as u64)? as u32;
            writer.write(&buffer)?;
            *args.1 = writer.stream_position()?;

            // Return to instance offset, and write instance data
            writer.seek(Start(instance_offset))?;
            hash_little32(self.name.as_bytes()).write_options(writer, endian, ())?;
            self.type_hash.write_options(writer, endian, ())?;
            buffer_offset.write_options(writer, endian, ())?;
            (buffer.len() as u32).write_options(writer, endian, ())?;
            self.name.write_options(writer, endian, (args.0,))?;
            Ok(())
        } else {
            Err(binrw::Error::Custom {
                pos: writer.stream_position()?,
                err: Box::new(AdfInstanceError::MutexFailure),
            })
        }
    }
}

#[derive(Error, Debug)]
pub enum AdfInstanceError {
    #[error("invalid hash")]
    InvalidNameHash(u32),
    #[error("failed to get mutex")]
    MutexFailure,
}

#[binrw]
#[brw(import(strings: &AdfReferenceCollector<NullString>, instances: &AdfReferenceCollector<Arc<AdfInstance>>))]
#[derive(Clone, Default, Debug, PartialEq)]
pub struct AdfType {
    pub primitive: AdfPrimitive,
    pub size: u32,
    pub alignment: u32,
    pub type_hash: u32,
    #[brw(args(strings))]
    pub name: AdfReference<NullString>,
    pub flags: AdfTypeFlags,
    pub scalar_type: AdfScalarType,
    pub element_type_hash: u32,
    pub element_length: u32,
    #[brw(if(matches!(primitive, AdfPrimitive::Structure)), args(strings, instances))]
    pub members: LengthVec<AdfMember, u32>,
    #[brw(if(matches!(primitive, AdfPrimitive::Enumeration)), args(strings))]
    pub enumerations: LengthVec<AdfEnum, u32>,
    #[brw(if(!matches!(primitive, AdfPrimitive::Enumeration | AdfPrimitive::Structure)), pad_after = 4)]
    pub padding: (),
}

#[binrw]
#[brw(repr = u32)]
#[derive(Clone, Default, Debug, PartialEq)]
pub enum AdfPrimitive {
    #[default]
    Scalar,
    Structure,
    Pointer,
    Array,
    InlineArray,
    String,
    Recursive,
    Bitfield,
    Enumeration,
    StringHash,
    Deferred,
}

bitflags! {
    #[binrw]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd)]
    pub struct AdfTypeFlags: u16 {
        const NONE = 0;
        const POD_READ = 1 << 0;
        const POD_WRITE = 1 << 1;
        const FINALIZE = 1 << 15;
    }
}

#[binrw]
#[brw(repr = u16)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd)]
pub enum AdfScalarType {
    #[default]
    Signed,
    Unsigned,
    Float,
}

#[binrw]
#[brw(import(strings: &AdfReferenceCollector<NullString>, instances: &AdfReferenceCollector<Arc<AdfInstance>>))]
#[derive(Clone, Default, Debug, PartialEq)]
pub struct AdfMember {
    #[brw(args(strings))]
    pub name: AdfReference<NullString>,
    pub type_hash: u32,
    pub alignment: u32,
    pub offsets: AdfMemberOffsets,
    #[brw(args(instances))]
    pub value: AdfMemberValue,
}

#[bitfield]
#[binrw]
#[br(map = Self::from_bytes)]
#[bw(map = |x: &Self| x.bytes)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct AdfMemberOffsets {
    pub byte: B24,
    pub bit: B8,
}

#[binrw]
#[brw(import(
    instances: &AdfReferenceCollector<Arc<AdfInstance>>
))]
#[derive(Clone, Debug, PartialEq)]
pub enum AdfMemberValue {
    #[brw(magic = 0u32)]
    UninitializedValue(#[brw(magic = 0u64)] ()),
    #[brw(magic = 1u32)]
    InlineValue(u64),
    #[brw(magic = 2u32)]
    InstanceValue(#[brw(args(instances))] AdfReference<Arc<AdfInstance>>),
}

impl Default for AdfMemberValue {
    #[inline]
    fn default() -> Self {
        Self::UninitializedValue(())
    }
}

#[binrw]
#[brw(import(strings: &AdfReferenceCollector<NullString>))]
#[derive(Clone, Default, Debug, PartialEq)]
pub struct AdfEnum {
    #[brw(args(strings))]
    pub name: AdfReference<NullString>,
    pub value: i32,
}

pub type AdfReferenceCollector<T> = std::rc::Rc<std::cell::RefCell<Vec<T>>>;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct AdfReference<T>(T);

impl<T: Clone + AdfReferenceIdentity<T>> BinRead for AdfReference<T>
where
    for<'a> T: 'a,
{
    type Args<'a> = (&'a AdfReferenceCollector<T>,);

    #[inline]
    fn read_options<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::prelude::BinResult<Self> {
        let pool = args.0.borrow();
        let pos = reader.stream_position()?;
        let identity = u64::read_options(reader, endian, ())?;
        <T as AdfReferenceIdentity<T>>::value(&pool, identity)
            .map(Self)
            .ok_or_else(|| binrw::Error::Custom {
                pos,
                err: Box::new(AdfReferenceError::InvalidIdentity(identity)),
            })
    }
}

impl<T: Clone + AdfReferenceEq + AdfReferenceIdentity<T>> BinWrite for AdfReference<T>
where
    for<'a> T: 'a,
{
    type Args<'a> = (&'a AdfReferenceCollector<T>,);

    #[inline]
    fn write_options<W: std::io::prelude::Write + std::io::prelude::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::prelude::BinResult<()> {
        let mut pool = args.0.borrow_mut();
        let pos = writer.stream_position()?;
        self.identity(&pool)
            .or_else(|| {
                pool.push(self.0.clone());
                self.identity(&pool)
            })
            .ok_or_else(|| binrw::Error::Custom {
                pos,
                err: Box::new(AdfReferenceError::CollectionFailure),
            })?
            .write_options(writer, endian, ())?;
        Ok(())
    }
}

impl<T> From<T> for AdfReference<T> {
    fn from(value: T) -> Self {
        AdfReference(value)
    }
}

impl<T> Deref for AdfReference<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for AdfReference<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Error, Debug)]
pub enum AdfReferenceError {
    #[error("invalid identity {0}")]
    InvalidIdentity(u64),
    #[error("failed to collect reference")]
    CollectionFailure,
}

trait AdfReferenceEq {
    fn eq(&self, other: &Self) -> bool;
}

impl AdfReferenceEq for NullString {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl AdfReferenceEq for Arc<AdfInstance> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(self, other)
    }
}

trait AdfReferenceIdentity<T> {
    fn value(
        pool: &[<AdfReferenceCollector<T> as AdfReferenceInner>::Inner],
        identity: u64,
    ) -> Option<T>;

    fn identity(
        &self,
        pool: &[<AdfReferenceCollector<T> as AdfReferenceInner>::Inner],
    ) -> Option<u64>;
}

impl AdfReferenceIdentity<NullString> for NullString {
    #[inline]
    fn value(pool: &[NullString], identity: u64) -> Option<NullString> {
        pool.get(identity as usize).cloned()
    }

    #[inline]
    fn identity(&self, pool: &[NullString]) -> Option<u64> {
        pool.iter().position(|x| x == self).map(|x| x as u64)
    }
}

impl AdfReferenceIdentity<Arc<AdfInstance>> for Arc<AdfInstance> {
    #[inline]
    fn value(pool: &[Arc<AdfInstance>], identity: u64) -> Option<Arc<AdfInstance>> {
        let identity = identity as u32;
        pool.iter()
            .find(|x| hash_little32(x.name.as_bytes()) == identity)
            .cloned()
            .or_else(|| {
                // TODO: Some ADFs were saved without defaults... not sure if we care?
                Some(AdfInstance::default().into())
            })
    }

    #[inline]
    fn identity(&self, pool: &[Arc<AdfInstance>]) -> Option<u64> {
        pool.iter()
            .position(|x| AdfReferenceEq::eq(x, self))
            .map(|_| hash_little32(self.name.as_bytes()) as u64)
    }
}

pub trait AdfReferenceInner {
    type Inner;
}

impl<T> AdfReferenceInner for AdfReferenceCollector<T> {
    type Inner = T;
}
