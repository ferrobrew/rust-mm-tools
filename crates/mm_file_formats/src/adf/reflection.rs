use std::{collections::HashMap, sync::Arc};

use aligned_vec::{AVec, RuntimeAlign};

use super::{
    AdfFile, AdfInstance, AdfPrimitive, AdfScalarType, AdfType, AdfTypeLib, BUILT_IN_TYPE_LIBRARY,
    TYPE_LIBRARIES,
};

#[derive(Clone, Debug, Default)]
pub struct AdfReflectionContext {
    types: HashMap<u32, AdfType>,
}

// TODO: we need to handle endian swapping + possible stack overflow; it's OK for now
impl AdfReflectionContext {
    pub fn from_extension(extension: impl AsRef<str>) -> binrw::BinResult<AdfReflectionContext> {
        let mut result = Self::default();
        result.load_types_from_library(BUILT_IN_TYPE_LIBRARY)?;
        let extension = extension.as_ref();
        for library in TYPE_LIBRARIES {
            if library.extension == extension {
                result.load_types_from_library(library)?;
            }
        }
        Ok(result)
    }

    pub fn load_types_from_library(&mut self, library: &AdfTypeLib) -> binrw::BinResult<()> {
        self.load_types_from_file(&library.load()?);
        Ok(())
    }

    pub fn load_types_from_file(&mut self, file: &AdfFile) {
        self.types
            .extend(file.types.iter().map(|x| (x.type_hash, x.clone())));
    }

    pub fn get_type(&self, type_hash: u32) -> Option<&AdfType> {
        self.types.get(&type_hash)
    }

    pub fn read_instance(&self, instance: &AdfInstance) -> Result<AdfReflectedValue, ()> {
        let Some(buffer) = instance.buffer.try_lock().ok() else {
            todo!("failed to lock buffer");
        };

        self.read_value_by_hash(instance.type_hash, buffer.as_slice(), 0, 0)
    }

    pub fn write_instance(
        &self,
        name: &impl AsRef<str>,
        value: &AdfReflectedValue,
        adf: &mut AdfFile,
    ) {
        let Some(type_info) = self.get_type(value.0) else {
            todo!("failed to get type info: {}", value.0);
        };

        let Some(instance) = adf.new_instance_from_type(name, type_info) else {
            todo!("failed to create instance: {}", name.as_ref());
        };

        let Ok(mut buffer) = instance.buffer.try_lock() else {
            todo!("failed to lock buffer");
        };

        self.write_value_by_hash(&value.1, value.0, &mut buffer, 0, 0)
            .expect("failed to write value");
    }

    fn read_value_by_hash(
        &self,
        type_hash: u32,
        buffer: &[u8],
        offset: usize,
        shift: usize,
    ) -> Result<AdfReflectedValue, ()> {
        let Some(type_info) = self.get_type(type_hash) else {
            todo!("failed to get type: {}", type_hash);
        };

        self.read_value_by_info(type_info, buffer, offset, shift)
    }

    fn write_value_by_hash(
        &self,
        primitive: &AdfReflectedPrimitive,
        type_hash: u32,
        buffer: &mut AVec<u8, RuntimeAlign>,
        offset: usize,
        shift: usize,
    ) -> Result<(), ()> {
        let Some(type_info) = self.get_type(type_hash) else {
            return Err(());
        };

        self.write_value_by_info(primitive, type_info, buffer, offset, shift)
    }

    fn read_value_by_info(
        &self,
        type_info: &AdfType,
        buffer: &[u8],
        offset: usize,
        shift: usize,
    ) -> Result<AdfReflectedValue, ()> {
        let type_size = type_info.size as usize;
        let alignment = type_info.alignment as usize;
        let type_hash = type_info.type_hash;
        // Validate the buffer contains the requested slice
        if buffer.len() < type_size || buffer.len() - type_size < offset {
            todo!("slice outside of buffer");
        };

        let slice = &buffer[offset..offset + type_size];
        let pointer = slice.as_ptr() as usize;
        // Validate the slice is correctly aligned
        if (pointer % alignment) != 0 {
            todo!("alignment incorrect");
        };

        Ok(match type_info.primitive {
            AdfPrimitive::Scalar => AdfReflectedValue(
                type_hash,
                AdfReflectedPrimitive::Scalar(Self::read_scalar(type_info, slice)?),
            ),
            AdfPrimitive::Structure => {
                let mut members = Vec::with_capacity(type_info.members.len());
                for member in type_info.members.iter() {
                    let member_offset = member.offsets.byte() as usize;
                    let member_bit_offset = member.offsets.bit() as usize;
                    members.push(self.read_value_by_hash(
                        member.type_hash,
                        buffer,
                        offset + member_offset,
                        member_bit_offset,
                    )?);
                }
                AdfReflectedValue(type_hash, AdfReflectedPrimitive::Structure(members))
            }
            AdfPrimitive::Pointer => {
                let Some(type_info) = self.get_type(type_info.element_type_hash) else {
                    todo!("failed to get type info: {}", type_info.element_type_hash);
                };
                // TODO: map of pointers, so we have one Arc<AdfReflectedValue> per read
                let offset = *bytemuck::from_bytes::<u64>(slice) as usize;
                AdfReflectedValue(
                    type_hash,
                    AdfReflectedPrimitive::Pointer(
                        self.read_value_by_info(type_info, buffer, offset, 0)?
                            .into(),
                    ),
                )
            }
            AdfPrimitive::Array => {
                let Some(type_info) = self.get_type(type_info.element_type_hash) else {
                    todo!("failed to get type info: {}", type_info.element_type_hash);
                };
                // TODO: map of pointers, so we have one Arc<Vec<AdfReflectedValue>> per read
                let offset = *bytemuck::from_bytes::<u64>(&slice[0..8]) as usize;
                let count = *bytemuck::from_bytes::<u64>(&slice[8..16]) as usize;
                AdfReflectedValue(
                    type_hash,
                    AdfReflectedPrimitive::Array(
                        self.read_array(type_info, buffer, offset, count)?.into(),
                    ),
                )
            }
            AdfPrimitive::InlineArray => {
                let count = type_info.element_length as usize;
                let Some(type_info) = self.get_type(type_info.element_type_hash) else {
                    todo!("failed to get type info: {}", type_info.element_type_hash);
                };
                AdfReflectedValue(
                    type_hash,
                    AdfReflectedPrimitive::InlineArray(
                        self.read_array(type_info, buffer, offset, count)?.into(),
                    ),
                )
            }
            AdfPrimitive::String => {
                // TODO: map of pointers, so we have one Arc<String> per read
                let start = *bytemuck::from_bytes::<u64>(slice) as usize;
                let mut end = start;
                while end < buffer.len() && buffer[end] != 0 {
                    end += 1;
                }
                let slice = &buffer[start..end];
                AdfReflectedValue(
                    type_hash,
                    AdfReflectedPrimitive::String(
                        String::from_utf8_lossy(slice).to_string().into(),
                    ),
                )
            }
            AdfPrimitive::Recursive => {
                todo!("Recursive is not implemented (no idea how this type works, sorry)")
            }
            AdfPrimitive::Bitfield => AdfReflectedValue(
                type_hash,
                AdfReflectedPrimitive::Bitfield(Self::read_bitfield(type_info, slice, shift)?),
            ),
            AdfPrimitive::Enumeration => AdfReflectedValue(
                type_hash,
                AdfReflectedPrimitive::Enumeration(Self::read_scalar(type_info, slice)?),
            ),
            AdfPrimitive::StringHash => {
                // TODO: why is element_type_hash = 0x48c5294d? Doesn't matter for our use case, but still.
                AdfReflectedValue(
                    type_hash,
                    AdfReflectedPrimitive::StringHash(Self::read_scalar(type_info, slice)?),
                )
            }
            AdfPrimitive::Deferred => {
                todo!("Deferred is not implemented (type_hash + offset)")
            }
        })
    }

    fn write_value_by_info(
        &self,
        value: &AdfReflectedPrimitive,
        type_info: &AdfType,
        buffer: &mut AVec<u8, RuntimeAlign>,
        offset: usize,
        shift: usize,
    ) -> Result<(), ()> {
        let type_size = type_info.size as usize;
        let alignment = type_info.alignment as usize;
        // Validate the buffer contains the requested slice
        if buffer.len() < type_size || buffer.len() - type_size < offset {
            todo!("slice outside of buffer");
        };

        let buffer_size = buffer.len();
        let slice = &mut buffer[offset..offset + type_size];
        let pointer = slice.as_ptr() as usize;
        // Validate the slice is correctly aligned
        if (pointer % alignment) != 0 {
            todo!("incorrect alignment");
        };

        // Validate primitive type
        macro_rules! validate_primitive {
            ($p:expr) => {{
                if type_info.primitive != $p {
                    todo!("invalid primitive: {:?}", $p);
                }
            }};
        }

        #[inline(always)]
        const fn align(value: usize, alignment: usize) -> usize {
            let align = alignment - 1;
            (value + align) & !align
        }

        match value {
            AdfReflectedPrimitive::Scalar(scalar) => {
                validate_primitive!(AdfPrimitive::Scalar);
                Self::write_scalar(slice, scalar, type_info)?;
            }
            AdfReflectedPrimitive::Structure(members) => {
                validate_primitive!(AdfPrimitive::Structure);
                if members.len() != type_info.members.len() {
                    todo!("unexpected member count: {}", members.len());
                };
                for (value, member) in members.iter().zip(type_info.members.iter()) {
                    if value.0 != member.type_hash {
                        todo!("unexpected member type: {}", value.0);
                    }
                    let member_offset = member.offsets.byte() as usize;
                    let member_bit_offset = member.offsets.bit() as usize;
                    self.write_value_by_hash(
                        &value.1,
                        member.type_hash,
                        buffer,
                        offset + member_offset,
                        member_bit_offset,
                    )?;
                }
            }
            AdfReflectedPrimitive::Pointer(value) => {
                validate_primitive!(AdfPrimitive::Pointer);
                if type_info.element_type_hash != value.0 {
                    todo!("unexpected pointer type: {}", value.0);
                };
                let Some(type_info) = self.get_type(type_info.element_type_hash) else {
                    todo!("failed to get type info: {}", type_info.element_type_hash);
                };

                let offset = align(buffer_size, type_info.alignment as usize);
                *bytemuck::from_bytes_mut::<u64>(&mut slice[0..8]) = offset as u64;

                let slack = offset - buffer_size;
                let size = slack + (type_info.size as usize);
                buffer.resize(buffer_size + size, 0u8);

                self.write_value_by_info(&value.1, type_info, buffer, offset, 0)?;
            }
            AdfReflectedPrimitive::Array(values) => {
                validate_primitive!(AdfPrimitive::Array);
                let Some(type_info) = self.get_type(type_info.element_type_hash) else {
                    todo!("failed to get type info: {}", type_info.element_type_hash);
                };

                let offset = align(buffer_size, type_info.alignment as usize);
                *bytemuck::from_bytes_mut::<u64>(&mut slice[0..8]) = offset as u64;

                let count = values.len();
                *bytemuck::from_bytes_mut::<u64>(&mut slice[8..16]) = count as u64;

                let slack = offset - buffer_size;
                let size = slack + (type_info.size as usize) * count;
                buffer.resize(buffer_size + size, 0u8);

                self.write_array(values, type_info, buffer, offset, count)?;
            }
            AdfReflectedPrimitive::InlineArray(values) => {
                validate_primitive!(AdfPrimitive::InlineArray);
                let count = type_info.element_length as usize;
                if values.len() != count {
                    todo!("unexpected array size: {}", values.len());
                };
                let Some(type_info) = self.get_type(type_info.element_type_hash) else {
                    todo!("failed to get type info: {}", type_info.element_type_hash);
                };
                self.write_array(values, type_info, buffer, offset, count)?;
            }
            AdfReflectedPrimitive::String(string) => {
                validate_primitive!(AdfPrimitive::String);

                let offset = buffer_size as u64;
                *bytemuck::from_bytes_mut::<u64>(slice) = offset;

                let size = string.as_bytes().len() + 1;
                buffer.resize(buffer_size + size, 0u8);
                for (i, &char) in string.as_bytes().iter().enumerate() {
                    buffer[buffer_size + i] = char;
                }
            }
            AdfReflectedPrimitive::Bitfield(scalar) => {
                validate_primitive!(AdfPrimitive::Bitfield);
                Self::write_bitfield(slice, scalar, type_info, shift)?;
            }
            AdfReflectedPrimitive::Enumeration(scalar) => {
                validate_primitive!(AdfPrimitive::Enumeration);
                Self::write_scalar(slice, scalar, type_info)?;
            }
            AdfReflectedPrimitive::StringHash(scalar) => {
                validate_primitive!(AdfPrimitive::StringHash);
                Self::write_scalar(slice, scalar, type_info)?;
            }
            AdfReflectedPrimitive::Deferred(_) => {
                validate_primitive!(AdfPrimitive::Deferred);
            }
        };

        Ok(())
    }

    fn read_array(
        &self,
        type_info: &AdfType,
        buffer: &[u8],
        offset: usize,
        count: usize,
    ) -> Result<Vec<AdfReflectedValue>, ()> {
        let element_size = type_info.size as usize;
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            values.push(self.read_value_by_info(
                type_info,
                buffer,
                offset + (i * element_size),
                0,
            )?);
        }
        Ok(values)
    }

    fn write_array(
        &self,
        values: &[AdfReflectedValue],
        type_info: &AdfType,
        buffer: &mut AVec<u8, RuntimeAlign>,
        offset: usize,
        count: usize,
    ) -> Result<(), ()> {
        if values.len() != count {
            todo!("unexpected array length: {}", values.len());
        }

        let element_size = type_info.size as usize;
        for (i, value) in values.iter().enumerate() {
            if value.0 != type_info.type_hash {
                todo!("unexpected type: {}", value.0);
            }

            self.write_value_by_info(&value.1, type_info, buffer, offset + (i * element_size), 0)?;
        }
        Ok(())
    }

    fn read_scalar(type_info: &AdfType, buffer: &[u8]) -> Result<AdfReflectedScalar, ()> {
        use bytemuck::from_bytes as read;
        match type_info.scalar_type {
            AdfScalarType::Signed => match type_info.size {
                1 => Ok(AdfReflectedScalar::I8(*read(buffer))),
                2 => Ok(AdfReflectedScalar::I16(*read(buffer))),
                4 => Ok(AdfReflectedScalar::I32(*read(buffer))),
                8 => Ok(AdfReflectedScalar::I64(*read(buffer))),
                size => todo!("invalid scalar size: {}", size),
            },
            AdfScalarType::Unsigned => match type_info.size {
                1 => Ok(AdfReflectedScalar::U8(*read(buffer))),
                2 => Ok(AdfReflectedScalar::U16(*read(buffer))),
                4 => Ok(AdfReflectedScalar::U32(*read(buffer))),
                8 => Ok(AdfReflectedScalar::U64(*read(buffer))),
                size => todo!("invalid scalar size: {}", size),
            },
            AdfScalarType::Float => match type_info.size {
                4 => Ok(AdfReflectedScalar::F32(*read(buffer))),
                8 => Ok(AdfReflectedScalar::F64(*read(buffer))),
                size => todo!("invalid scalar size: {}", size),
            },
        }
    }

    fn write_scalar(
        buffer: &mut [u8],
        scalar: &AdfReflectedScalar,
        type_info: &AdfType,
    ) -> Result<(), ()> {
        macro_rules! write {
            ($t:tt, $st:expr, $v:expr) => {{
                let size = std::mem::size_of::<$t>() as u32;
                if type_info.size != size {
                    todo!("invalid scalar size: {}", size);
                }
                if type_info.alignment != size {
                    todo!("invalid scalar alignment: {}", size);
                }
                if type_info.scalar_type != $st {
                    todo!("invalid scalar type: {:?}", $st);
                }
                *bytemuck::from_bytes_mut::<$t>(buffer) = *$v;
                Ok(())
            }};
        }

        match scalar {
            AdfReflectedScalar::U8(value) => write!(u8, AdfScalarType::Unsigned, value),
            AdfReflectedScalar::I8(value) => write!(i8, AdfScalarType::Signed, value),
            AdfReflectedScalar::U16(value) => write!(u16, AdfScalarType::Unsigned, value),
            AdfReflectedScalar::I16(value) => write!(i16, AdfScalarType::Signed, value),
            AdfReflectedScalar::U32(value) => write!(u32, AdfScalarType::Unsigned, value),
            AdfReflectedScalar::I32(value) => write!(i32, AdfScalarType::Signed, value),
            AdfReflectedScalar::F32(value) => write!(f32, AdfScalarType::Float, value),
            AdfReflectedScalar::U64(value) => write!(u64, AdfScalarType::Unsigned, value),
            AdfReflectedScalar::I64(value) => write!(i64, AdfScalarType::Signed, value),
            AdfReflectedScalar::F64(value) => write!(f64, AdfScalarType::Float, value),
        }
    }

    fn read_bitfield(
        type_info: &AdfType,
        buffer: &[u8],
        shift: usize,
    ) -> Result<AdfReflectedScalar, ()> {
        let mask = ((1usize << type_info.element_length as usize) - 1usize) << shift;
        macro_rules! read {
            ($t:tt) => {
                (*bytemuck::from_bytes::<$t>(buffer) & mask as $t) >> shift
            };
        }
        match type_info.scalar_type {
            AdfScalarType::Signed => match type_info.size {
                1 => Ok(AdfReflectedScalar::I8(read!(i8))),
                2 => Ok(AdfReflectedScalar::I16(read!(i16))),
                4 => Ok(AdfReflectedScalar::I32(read!(i32))),
                8 => Ok(AdfReflectedScalar::I64(read!(i64))),
                size => todo!("invalid scalar size: {}", size),
            },
            AdfScalarType::Unsigned => match type_info.size {
                1 => Ok(AdfReflectedScalar::U8(read!(u8))),
                2 => Ok(AdfReflectedScalar::U16(read!(u16))),
                4 => Ok(AdfReflectedScalar::U32(read!(u32))),
                8 => Ok(AdfReflectedScalar::U64(read!(u64))),
                size => todo!("invalid scalar size: {}", size),
            },
            AdfScalarType::Float => todo!("invalid scalar type: {:?}", type_info.scalar_type),
        }
    }

    fn write_bitfield(
        buffer: &mut [u8],
        scalar: &AdfReflectedScalar,
        type_info: &AdfType,
        shift: usize,
    ) -> Result<(), ()> {
        macro_rules! write {
            ($t:tt, $st:expr, $v:expr) => {{
                let mask = (1 << type_info.element_length) - 1;
                let size = std::mem::size_of::<$t>() as u32;
                if type_info.size != size {
                    todo!("invalid scalar size: {}", size);
                }
                if type_info.alignment != size {
                    todo!("invalid scalar alignment: {}", size);
                }
                if type_info.scalar_type != $st {
                    todo!("invalid scalar type: {:?}", $st);
                }
                *bytemuck::from_bytes_mut::<$t>(buffer) |= ($v & mask) << shift;
                Ok(())
            }};
        }

        match scalar {
            AdfReflectedScalar::U8(value) => write!(u8, AdfScalarType::Unsigned, value),
            AdfReflectedScalar::I8(value) => write!(i8, AdfScalarType::Signed, value),
            AdfReflectedScalar::U16(value) => write!(u16, AdfScalarType::Unsigned, value),
            AdfReflectedScalar::I16(value) => write!(i16, AdfScalarType::Signed, value),
            AdfReflectedScalar::U32(value) => write!(u32, AdfScalarType::Unsigned, value),
            AdfReflectedScalar::I32(value) => write!(i32, AdfScalarType::Signed, value),
            AdfReflectedScalar::U64(value) => write!(u64, AdfScalarType::Unsigned, value),
            AdfReflectedScalar::I64(value) => write!(i64, AdfScalarType::Signed, value),
            _ => todo!("invalid scalar value: {:?}", scalar),
        }
    }
}

#[derive(Clone, Debug)]
pub enum AdfReflectedPrimitive {
    // Represents a numeric value.
    Scalar(AdfReflectedScalar),
    // Represents a structure comprised of multiple reflected values.
    Structure(Vec<AdfReflectedValue>),
    // Represents an indirect reflected value of the specified type.
    Pointer(Arc<AdfReflectedValue>),
    // Represents an indirect array of reflected values.
    Array(Arc<Vec<AdfReflectedValue>>),
    // Represents an array of reflected values.
    InlineArray(Vec<AdfReflectedValue>),
    // Represents an indirect string.
    String(Arc<String>),
    // Represents a bitfield derived from a numeric value.
    Bitfield(AdfReflectedScalar),
    // Represents an enumeration derived from a numeric value.
    Enumeration(AdfReflectedScalar),
    // Represents an numeric value derived from a string hash.
    StringHash(AdfReflectedScalar),
    // Represents an indirect reflected value of any type.
    Deferred(Arc<AdfReflectedValue>),
}

#[derive(Clone, Debug)]
pub enum AdfReflectedScalar {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
    U64(u64),
    I64(i64),
    F64(f64),
}

#[derive(Clone, Debug)]
pub struct AdfReflectedValue(pub u32, pub AdfReflectedPrimitive);
