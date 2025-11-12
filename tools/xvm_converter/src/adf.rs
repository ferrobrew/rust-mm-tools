use std::sync::Arc;

use mm_file_formats::adf::{
    AdfRead, AdfReaderReferences, AdfTypeInfo, AdfWrite, AdfWriterReferences,
};

#[derive(Default, Debug)]
pub struct XvmFormatModule {
    pub name_hash: u32,
    pub src_crc: u32,
    pub properties: u32,
    pub module_size: u32,
    pub debug_info_array: u64,
    pub this_instance: XvmFormatStructInstance,
    pub functions: Arc<Vec<XvmFormatFunction>>,
    pub import_hashes: Arc<Vec<u32>>,
    pub constants: Arc<Vec<XvmFormatConstant>>,
    pub string_hashes: Arc<Vec<u32>>,
    pub string_buffer: Arc<Vec<u8>>,
    pub debug_string_pointer: u64,
    pub debug_strings: u64,
    pub name: Arc<Vec<u8>>,
}

impl AdfTypeInfo for XvmFormatModule {
    const NAME: &str = "XvmFormatModule";
    const HASH: u32 = 1104159559;
    const SIZE: u64 = 152;
    const ALIGN: u64 = 8;
}

impl AdfRead for XvmFormatModule {
    fn read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, mm_file_formats::adf::AdfReadWriteError> {
        Ok(Self {
            name_hash: AdfRead::read(reader, references)?,
            src_crc: AdfRead::read(reader, references)?,
            properties: AdfRead::read(reader, references)?,
            module_size: AdfRead::read(reader, references)?,
            debug_info_array: AdfRead::read(reader, references)?,
            this_instance: AdfRead::read(reader, references)?,
            functions: AdfRead::read(reader, references)?,
            import_hashes: AdfRead::read(reader, references)?,
            constants: AdfRead::read(reader, references)?,
            string_hashes: AdfRead::read(reader, references)?,
            string_buffer: AdfRead::read(reader, references)?,
            debug_string_pointer: AdfRead::read(reader, references)?,
            debug_strings: AdfRead::read(reader, references)?,
            name: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for XvmFormatModule {
    fn write<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), mm_file_formats::adf::AdfReadWriteError> {
        self.name_hash.write(writer, references)?;
        self.src_crc.write(writer, references)?;
        self.properties.write(writer, references)?;
        self.module_size.write(writer, references)?;
        self.debug_info_array.write(writer, references)?;
        self.this_instance.write(writer, references)?;
        self.functions.write(writer, references)?;
        self.import_hashes.write(writer, references)?;
        self.constants.write(writer, references)?;
        self.string_hashes.write(writer, references)?;
        self.string_buffer.write(writer, references)?;
        self.debug_string_pointer.write(writer, references)?;
        self.debug_strings.write(writer, references)?;
        self.name.write(writer, references)?;
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct XvmFormatStructInstance {
    pub instance: u64,
    pub kind: u64,
}

impl AdfTypeInfo for XvmFormatStructInstance {
    const NAME: &str = "XvmFormatStructInstance";
    const HASH: u32 = 837236792;
    const SIZE: u64 = 16;
    const ALIGN: u64 = 8;
}

impl AdfRead for XvmFormatStructInstance {
    fn read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, mm_file_formats::adf::AdfReadWriteError> {
        Ok(Self {
            instance: AdfRead::read(reader, references)?,
            kind: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for XvmFormatStructInstance {
    fn write<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), mm_file_formats::adf::AdfReadWriteError> {
        self.instance.write(writer, references)?;
        self.kind.write(writer, references)?;
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct XvmFormatFunction {
    pub name_hash: u32,
    pub locals_count: u16,
    pub arg_count: u16,
    pub instructions: Arc<Vec<u16>>,
    pub max_stack_depth: u16,
    pub module: u64,
    pub lineno_ptr: u64,
    pub colno_ptr: u64,
    pub name: Arc<Vec<u8>>,
}

impl AdfTypeInfo for XvmFormatFunction {
    const NAME: &str = "XvmFormatFunction";
    const HASH: u32 = 1607835701;
    const SIZE: u64 = 72;
    const ALIGN: u64 = 8;
}

impl AdfRead for XvmFormatFunction {
    fn read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, mm_file_formats::adf::AdfReadWriteError> {
        Ok(Self {
            name_hash: AdfRead::read(reader, references)?,
            locals_count: AdfRead::read(reader, references)?,
            arg_count: AdfRead::read(reader, references)?,
            instructions: AdfRead::read(reader, references)?,
            max_stack_depth: AdfRead::read(reader, references)?,
            module: AdfRead::read(reader, references)?,
            lineno_ptr: AdfRead::read(reader, references)?,
            colno_ptr: AdfRead::read(reader, references)?,
            name: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for XvmFormatFunction {
    fn write<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), mm_file_formats::adf::AdfReadWriteError> {
        self.name_hash.write(writer, references)?;
        self.locals_count.write(writer, references)?;
        self.arg_count.write(writer, references)?;
        self.instructions.write(writer, references)?;
        self.max_stack_depth.write(writer, references)?;
        self.module.write(writer, references)?;
        self.lineno_ptr.write(writer, references)?;
        self.colno_ptr.write(writer, references)?;
        self.name.write(writer, references)?;
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct XvmFormatConstant {
    pub flags: u64,
    pub value: u64,
}

impl AdfTypeInfo for XvmFormatConstant {
    const NAME: &str = "XvmFormatConstant";
    const HASH: u32 = 1545366412;
    const SIZE: u64 = 16;
    const ALIGN: u64 = 8;
}

impl AdfRead for XvmFormatConstant {
    fn read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, mm_file_formats::adf::AdfReadWriteError> {
        Ok(Self {
            flags: AdfRead::read(reader, references)?,
            value: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for XvmFormatConstant {
    fn write<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), mm_file_formats::adf::AdfReadWriteError> {
        self.flags.write(writer, references)?;
        self.value.write(writer, references)?;
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct XvmFormatStructType {
    pub name_hash: u32,
}

impl AdfTypeInfo for XvmFormatStructType {
    const NAME: &str = "XvmFormatStructType";
    const HASH: u32 = 1179316735;
    const SIZE: u64 = 4;
    const ALIGN: u64 = 4;
}

impl AdfRead for XvmFormatStructType {
    fn read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, mm_file_formats::adf::AdfReadWriteError> {
        Ok(Self {
            name_hash: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for XvmFormatStructType {
    fn write<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), mm_file_formats::adf::AdfReadWriteError> {
        self.name_hash.write(writer, references)?;
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct XvmFunctionDebugArray {
    pub debug_info: Arc<Vec<XvmFormatFunctionDebug>>,
}

impl AdfTypeInfo for XvmFunctionDebugArray {
    const NAME: &str = "XvmFunctionDebugArray";
    const HASH: u32 = 3702547558;
    const SIZE: u64 = 16;
    const ALIGN: u64 = 8;
}

impl AdfRead for XvmFunctionDebugArray {
    fn read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, mm_file_formats::adf::AdfReadWriteError> {
        Ok(Self {
            debug_info: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for XvmFunctionDebugArray {
    fn write<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), mm_file_formats::adf::AdfReadWriteError> {
        self.debug_info.write(writer, references)?;
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct XvmFormatFunctionDebug {
    pub lineno: Arc<Vec<u16>>,
    pub colno: Arc<Vec<u16>>,
    pub name_hash: u32,
}

impl AdfTypeInfo for XvmFormatFunctionDebug {
    const NAME: &str = "XvmFormatFunctionDebug";
    const HASH: u32 = 623902124;
    const SIZE: u64 = 40;
    const ALIGN: u64 = 8;
}

impl AdfRead for XvmFormatFunctionDebug {
    fn read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, mm_file_formats::adf::AdfReadWriteError> {
        Ok(Self {
            lineno: AdfRead::read(reader, references)?,
            colno: AdfRead::read(reader, references)?,
            name_hash: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for XvmFormatFunctionDebug {
    fn write<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), mm_file_formats::adf::AdfReadWriteError> {
        self.lineno.write(writer, references)?;
        self.colno.write(writer, references)?;
        self.name_hash.write(writer, references)?;
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct XvmFormatDebugStrings {
    pub string_buffer_debug: Arc<Vec<u8>>,
}

impl AdfTypeInfo for XvmFormatDebugStrings {
    const NAME: &str = "XvmFormatDebugStrings";
    const HASH: u32 = 4277384585;
    const SIZE: u64 = 16;
    const ALIGN: u64 = 8;
}

impl AdfRead for XvmFormatDebugStrings {
    fn read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, mm_file_formats::adf::AdfReadWriteError> {
        Ok(Self {
            string_buffer_debug: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for XvmFormatDebugStrings {
    fn write<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), mm_file_formats::adf::AdfReadWriteError> {
        self.string_buffer_debug.write(writer, references)?;
        Ok(())
    }
}
