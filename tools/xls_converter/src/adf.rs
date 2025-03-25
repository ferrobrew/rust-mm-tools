use std::sync::Arc;

use mm_file_formats::adf::{
    AdfRead, AdfReaderReferences, AdfTypeInfo, AdfWrite, AdfWriterReferences,
};

#[derive(Default, Debug)]
pub struct XlsBook {
    pub sheet: Arc<Vec<XlsSheet>>,
    pub cell: Arc<Vec<XlsCell>>,
    pub string_data: Arc<Vec<Arc<String>>>,
    pub value_data: Arc<Vec<f32>>,
    pub bool_data: Arc<Vec<u8>>,
    pub date_data: Arc<Vec<Arc<String>>>,
    pub color_data: Arc<Vec<u32>>,
    pub attribute: Arc<Vec<XlsAttribute>>,
}

impl AdfTypeInfo for XlsBook {
    const NAME: &str = "XLSBook";
    const HASH: u32 = 192098653;
    const SIZE: u64 = 128;
    const ALIGN: u64 = 8;
}

impl AdfRead for XlsBook {
    fn read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, mm_file_formats::adf::AdfReadWriteError> {
        Ok(Self {
            sheet: AdfRead::read(reader, references)?,
            cell: AdfRead::read(reader, references)?,
            string_data: AdfRead::read(reader, references)?,
            value_data: AdfRead::read(reader, references)?,
            bool_data: AdfRead::read(reader, references)?,
            date_data: AdfRead::read(reader, references)?,
            color_data: AdfRead::read(reader, references)?,
            attribute: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for XlsBook {
    fn write<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), mm_file_formats::adf::AdfReadWriteError> {
        self.sheet.write(writer, references)?;
        self.cell.write(writer, references)?;
        self.string_data.write(writer, references)?;
        self.value_data.write(writer, references)?;
        self.bool_data.write(writer, references)?;
        self.date_data.write(writer, references)?;
        self.color_data.write(writer, references)?;
        self.attribute.write(writer, references)?;
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct XlsSheet {
    pub cols: u32,
    pub rows: u32,
    pub cell_index: Arc<Vec<u32>>,
    pub name: Arc<String>,
}

impl AdfTypeInfo for XlsSheet {
    const NAME: &str = "XLSSheet";
    const HASH: u32 = 3649567627;
    const SIZE: u64 = 32;
    const ALIGN: u64 = 8;
}

impl AdfRead for XlsSheet {
    fn read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, mm_file_formats::adf::AdfReadWriteError> {
        Ok(Self {
            cols: AdfRead::read(reader, references)?,
            rows: AdfRead::read(reader, references)?,
            cell_index: AdfRead::read(reader, references)?,
            name: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for XlsSheet {
    fn write<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), mm_file_formats::adf::AdfReadWriteError> {
        self.cols.write(writer, references)?;
        self.rows.write(writer, references)?;
        self.cell_index.write(writer, references)?;
        self.name.write(writer, references)?;
        Ok(())
    }
}

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq)]
pub struct XlsCell {
    pub kind: u16,
    pub data_index: u32,
    pub attribute_index: u32,
}

impl AdfTypeInfo for XlsCell {
    const NAME: &str = "XLSCell";
    const HASH: u32 = 2598569309;
    const SIZE: u64 = 12;
    const ALIGN: u64 = 4;
}

impl AdfRead for XlsCell {
    fn read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, mm_file_formats::adf::AdfReadWriteError> {
        Ok(Self {
            kind: AdfRead::read(reader, references)?,
            data_index: AdfRead::read(reader, references)?,
            attribute_index: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for XlsCell {
    fn write<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), mm_file_formats::adf::AdfReadWriteError> {
        self.kind.write(writer, references)?;
        self.data_index.write(writer, references)?;
        self.attribute_index.write(writer, references)?;
        Ok(())
    }
}

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq)]
pub struct XlsAttribute {
    pub fg_color_index: u8,
    pub bg_color_index: u8,
}

impl AdfTypeInfo for XlsAttribute {
    const NAME: &str = "XLSAttribute";
    const HASH: u32 = 2397202994;
    const SIZE: u64 = 2;
    const ALIGN: u64 = 1;
}

impl AdfRead for XlsAttribute {
    fn read<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, mm_file_formats::adf::AdfReadWriteError> {
        Ok(Self {
            fg_color_index: AdfRead::read(reader, references)?,
            bg_color_index: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for XlsAttribute {
    fn write<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), mm_file_formats::adf::AdfReadWriteError> {
        self.fg_color_index.write(writer, references)?;
        self.bg_color_index.write(writer, references)?;
        Ok(())
    }
}
