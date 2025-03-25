use std::{collections::HashMap, hash::Hash, io::Write, sync::Arc};

use aligned_vec::AVec;
use anyhow::{anyhow, bail, Context};
use binrw::{BinRead, BinWrite};
use clap::Parser;
use serde::{Deserialize, Serialize};

use mm_file_formats::{
    adf::{
        AdfFile, AdfInstance, AdfRead, AdfReaderReferences, AdfReflectionContext, AdfTypeInfo,
        AdfWrite, AdfWriterReferences, BUILT_IN_TYPE_LIBRARY, TYPE_LIBRARIES,
    },
    common::NullString,
};

mod adf;
use adf::{XlsAttribute, XlsBook, XlsCell, XlsSheet};

mod xml;
use xml::{XmlBook, XmlCell, XmlCellKind, XmlRow, XmlSheet};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if !args.file.is_file() {
        bail!("{:?} is not a file", args.file);
    }

    let extension = args
        .file
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .context("Failed to determine file extension")?;

    // Open the file
    let file = std::fs::File::open(args.file.clone()).context("Failed to open file")?;
    let mut reader = std::io::BufReader::new(file);

    // Load built-in types
    let mut context = AdfReflectionContext::default();
    context.load_types(&BUILT_IN_TYPE_LIBRARY.load()?);

    match extension {
        "xlsc" => {
            // Load types based on extension
            load_types(&mut context, extension)?;

            // Parse the ADF, intentionally not loading additional types
            let adf = AdfFile::read_le(&mut reader).context("Failed to parse ADF")?;

            // Find associated ADF instance
            let instance = adf
                .instances
                .iter()
                .find(|instance| {
                    instance.name.as_ref() == "XLSBook" && instance.type_hash == XlsBook::HASH
                })
                .context("failed to find matching instance")?;

            // Parse associated instance
            if let Ok(buffer) = instance.buffer.lock() {
                let mut reader = std::io::BufReader::new(std::io::Cursor::new(buffer.as_ref()));
                let mut references = AdfReaderReferences::default();
                let book = XlsBook::read(&mut reader, &mut references)?;

                let mut xml_book = XmlBook::default();

                xml_book.sheets.reserve(book.sheet.len());
                for sheet in book.sheet.iter() {
                    let mut xml_sheet = XmlSheet::default();
                    xml_sheet.name = sheet.name.to_string();

                    xml_sheet.rows.reserve(sheet.rows as usize);
                    for y in 0..sheet.rows {
                        let mut xml_row = XmlRow::default();

                        xml_row.cells.reserve(sheet.cols as usize);
                        for x in 0..sheet.cols {
                            let cell_index = sheet
                                .cell_index
                                .get((x + y * sheet.cols) as usize)
                                .context("Failed to get cell index")?;

                            let cell = book
                                .cell
                                .get(*cell_index as usize)
                                .context("Failed to get cell data")?;

                            let (kind, value) = match cell.kind {
                                0 => book
                                    .bool_data
                                    .get(cell.data_index as usize)
                                    .map(|data| (XmlCellKind::Bool, data.to_string())),
                                1 => book
                                    .string_data
                                    .get(cell.data_index as usize)
                                    .map(|data| (XmlCellKind::String, data.to_string())),
                                2 => book
                                    .value_data
                                    .get(cell.data_index as usize)
                                    .map(|data| (XmlCellKind::Value, data.to_string())),
                                3 => book
                                    .date_data
                                    .get(cell.data_index as usize)
                                    .map(|data| (XmlCellKind::Date, data.to_string())),
                                4 => book
                                    .color_data
                                    .get(cell.data_index as usize)
                                    .map(|data| (XmlCellKind::Color, data.to_string())),
                                kind => {
                                    bail!("Unknown cell data type: {kind}");
                                }
                            }
                            .context("Failed to get cell data")?;

                            let attributes = book
                                .attribute
                                .get(cell.attribute_index as usize)
                                .context("Failed to get cell style")?;
                            let foreground_color = book
                                .color_data
                                .get(
                                    (attributes.fg_color_index % book.color_data.len() as u8)
                                        as usize,
                                )
                                .cloned()
                                .context("Failed to get foreground color")?;
                            let background_color = book
                                .color_data
                                .get(
                                    (attributes.bg_color_index % book.color_data.len() as u8)
                                        as usize,
                                )
                                .cloned()
                                .context("Failed to get background color")?;

                            xml_row.cells.push(XmlCell {
                                kind,
                                foreground_color,
                                background_color,
                                value,
                            });
                        }

                        xml_sheet.rows.push(xml_row);
                    }

                    xml_book.sheets.push(xml_sheet);
                }

                // Configure XML serializer
                let mut buffer = String::new();
                let mut serializer =
                    quick_xml::se::Serializer::with_root(&mut buffer, Some("book"))?;
                serializer.indent('\t', 1);
                serializer.expand_empty_elements(true);

                // Write XML
                xml_book.serialize(serializer)?;
                let mut file = std::fs::File::create(args.file.with_extension("xml"))?;
                file.write_all(buffer.as_bytes())?;
            };
        }
        "xml" => {
            // Parse the XML
            let mut deserializer = quick_xml::de::Deserializer::from_reader(reader);
            let xml_book = XmlBook::deserialize(&mut deserializer)?;

            let mut cells = Collection::<XlsCell>::default();
            let mut attributes = Collection::<XlsAttribute>::default();
            let mut strings = Collection::<String, Arc<String>>::default();
            let mut values = Collection::<u32, f32>::default();
            let mut bools = Collection::<u8>::default();
            let mut dates = Collection::<String, Arc<String>>::default();
            let mut colors = Collection::<u32>::default();

            let mut sheets = Vec::with_capacity(xml_book.sheets.len());

            for xml_sheet in &xml_book.sheets {
                let name = strings.value(&xml_sheet.name);
                let cols = xml_sheet
                    .rows
                    .get(0)
                    .context("Failed to determine column count")?
                    .cells
                    .len() as u32;
                let rows = xml_sheet.rows.len() as u32;

                let mut indices = Vec::with_capacity((cols * rows) as usize);
                for row in &xml_sheet.rows {
                    if row.cells.len() as u32 != cols {
                        bail!("Row column length mismatch!");
                    }

                    for cell in &row.cells {
                        let attribute_index = attributes.index(&XlsAttribute {
                            fg_color_index: colors.index(&cell.foreground_color) as u8,
                            bg_color_index: colors.index(&cell.background_color) as u8,
                        }) as u32;

                        let (kind, data_index) = match cell.kind {
                            XmlCellKind::Bool => (
                                0,
                                bools.index(&cell.value.parse().context("Failed to parse Bool")?)
                                    as u32,
                            ),
                            XmlCellKind::String => (1, strings.index(&cell.value) as u32),
                            XmlCellKind::Value => (
                                2,
                                values.index(&cell.value.parse().context("Failed to parse Value")?)
                                    as u32,
                            ),
                            XmlCellKind::Date => (3, dates.index(&cell.value) as u32),
                            XmlCellKind::Color => (
                                4,
                                colors.index(&cell.value.parse().context("Failed to parse Color")?)
                                    as u32,
                            ),
                        };

                        indices.push(cells.index(&XlsCell {
                            kind,
                            data_index,
                            attribute_index,
                        }) as u32);
                    }
                }

                sheets.push(XlsSheet {
                    cols,
                    rows,
                    cell_index: indices.into(),
                    name,
                });
            }

            // Write book to buffer
            let mut buffer = vec![];
            {
                let mut writer = std::io::BufWriter::new(std::io::Cursor::new(&mut buffer));
                let mut references = AdfWriterReferences::default();
                references.0 = XlsBook::SIZE;
                XlsBook {
                    sheet: sheets.into(),
                    cell: cells.values.into(),
                    string_data: strings.values.into(),
                    value_data: values.values.into(),
                    bool_data: bools.values.into(),
                    date_data: dates.values.into(),
                    color_data: colors.values.into(),
                    attribute: attributes.values.into(),
                }
                .write(&mut writer, &mut references)?;
            }

            // Load XLSC type library
            let mut adf = TYPE_LIBRARIES
                .iter()
                .find(|lib| lib.extension == "xlsc")
                .context("Failed to find type library")?
                .load()
                .context("Failed to load type library")?;

            // Overwrite it's instances / description
            adf.description = NullString::from("");
            adf.instances = vec![AdfInstance {
                name: NullString::from("XLSBook").into(),
                type_hash: XlsBook::HASH,
                buffer: AVec::from_iter(XlsBook::ALIGN as usize, buffer.into_iter()).into(),
            }
            .into()];

            // Finally write it to disk
            let mut file = std::fs::File::create(args.file.with_extension("xlsc"))?;
            let mut writer = std::io::BufWriter::new(&mut file);
            adf.write_le(&mut writer)?;
        }
        extension => {
            bail!("This tool does not support the '{extension}' extension");
        }
    }

    Ok(())
}

#[derive(Parser)]
struct Args {
    #[arg()]
    file: std::path::PathBuf,
}

fn load_types(context: &mut AdfReflectionContext, extension: &str) -> anyhow::Result<()> {
    let mut found_types = false;
    for type_library in TYPE_LIBRARIES {
        if type_library.extension == extension {
            found_types = true;
            context.load_types(&type_library.load()?);
        }
    }
    found_types
        .then_some(())
        .ok_or(anyhow!("Failed to find type libraries for {}", extension))
}

#[derive(Default)]
struct Collection<T: Eq + Hash + Clone, V = T> {
    indices: HashMap<T, usize>,
    values: Vec<V>,
}

impl<T: Eq + Hash + Clone> Collection<T> {
    pub fn index(&mut self, value: &T) -> usize {
        self.indices.get(value).cloned().unwrap_or_else(|| {
            let index = self.values.len();
            self.indices.insert(value.clone(), index);
            self.values.insert(index, value.clone());
            index
        })
    }
}

impl Collection<u32, f32> {
    pub fn index(&mut self, value: &f32) -> usize {
        let key = value.to_bits();
        self.indices.get(&key).cloned().unwrap_or_else(|| {
            let index = self.values.len();
            self.indices.insert(key, index);
            self.values.insert(index, value.clone().into());
            index
        })
    }
}

impl Collection<String, Arc<String>> {
    pub fn index(&mut self, value: &str) -> usize {
        self.indices.get(value).cloned().unwrap_or_else(|| {
            let index = self.values.len();
            let value = value.to_string();
            self.indices.insert(value.clone(), index);
            self.values.insert(index, value.to_string().into());
            index
        })
    }

    pub fn value(&mut self, value: &str) -> Arc<String> {
        let index = self.index(value);
        self.values.get(index).cloned().unwrap()
    }
}
