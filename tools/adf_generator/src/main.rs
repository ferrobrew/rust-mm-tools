use std::{collections::HashSet, io::Write, path::PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;
use convert_case::{Case, Casing};

use mm_file_formats::adf::{AdfPrimitive, AdfReflectionContext, AdfScalarType, AdfType};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Create file
    let mut file = std::fs::File::create(args.path).context("Failed to create file")?;
    let mut writer = std::io::BufWriter::new(&mut file);
    macro_rules! out {
        ($($arg:tt)*) => {
            writeln!(writer, $($arg)*)?
        };
    }

    // Load types based on extension
    let context = AdfReflectionContext::from_extension(args.extension)?;

    // Find base type
    let type_info = context
        .get_type_by_name(&args.type_name)
        .context(format!("failed to find type: {}", args.type_name))?;

    // Collect used types
    let types = collect_types(&context, type_info);

    out!("use std::{{");
    out!("    io::{{Read, Seek, Write}},");
    out!("    sync::Arc,");
    out!("}};\n");

    out!("use mm_file_formats::adf::{{");
    out!("    AdfRead, AdfReadWriteError, AdfReaderReferences, AdfTypeInfo, AdfWrite, AdfWriterReferences,");
    out!("}};");
    out!("use mm_hashing::HashString;\n");

    // Write types
    for type_hash in &types {
        let type_info = context
            .get_type_by_hash(*type_hash)
            .context(format!("failed to find type: {type_hash}"))?;

        match type_info.primitive {
            AdfPrimitive::Structure => {
                out!("#[derive(Clone, Default, Debug)]");
                out!("pub struct {} {{", type_info.name.as_str());
                for member in type_info.members.iter() {
                    out!(
                        "    pub {}: {},",
                        member.name.as_str().to_case(Case::Snake),
                        type_name(&context, member.type_hash)?
                    );
                }
                out!("}}\n");

                out!("impl AdfTypeInfo for {} {{", type_info.name.as_str());
                out!("    const NAME: &str = \"{}\";", type_info.name.as_str());
                out!("    const HASH: u32 = {};", type_info.type_hash);
                out!("    const SIZE: u64 = {};", type_info.size);
                out!("    const ALIGN: u64 = {};", type_info.alignment);
                out!("}}\n");

                out!("impl AdfRead for {} {{", type_info.name.as_str());
                out!("    #[inline]");
                out!("    fn read<R: Read + Seek>(");
                out!("        reader: &mut R,");
                out!("        references: &mut AdfReaderReferences,");
                out!("    ) -> Result<Self, AdfReadWriteError> {{");
                out!("        Ok(Self {{");
                for member in type_info.members.iter() {
                    out!(
                        "            {}: AdfRead::read(reader, references)?,",
                        member.name.as_str().to_case(Case::Snake)
                    );
                }
                out!("        }})");
                out!("    }}");
                out!("}}\n");

                out!("impl AdfWrite for {} {{", type_info.name.as_str());
                out!("    #[inline]");
                out!("    fn write<W: Write + Seek>(");
                out!("        &self,");
                out!("        writer: &mut W,");
                out!("        references: &mut AdfWriterReferences,");
                out!("    ) -> Result<(), AdfReadWriteError> {{");
                for member in type_info.members.iter() {
                    out!(
                        "        self.{}.write(writer, references)?;",
                        member.name.as_str().to_case(Case::Snake)
                    );
                }
                out!("        Ok(())");
                out!("    }}");
                out!("}}\n");
            }
            AdfPrimitive::Bitfield | AdfPrimitive::Enumeration => {
                bail!("invalid primitive: {:?}", type_info.primitive)
            }
            _ => {}
        }
    }

    Ok(())
}

#[derive(Parser)]
struct Args {
    #[arg()]
    extension: String,
    #[arg()]
    type_name: String,
    #[arg()]
    path: PathBuf,
}

fn collect_types<'a>(context: &'a AdfReflectionContext, value: &'a AdfType) -> Vec<u32> {
    let mut types = HashSet::<u32>::default();
    let mut post_order = Vec::<u32>::default();
    insert_value_by_hash(context, &mut types, &mut post_order, value.type_hash);
    post_order
}

fn insert_value_by_hash<'a>(
    context: &AdfReflectionContext,
    types: &'a mut HashSet<u32>,
    post_order: &'a mut Vec<u32>,
    type_hash: u32,
) -> (&'a mut HashSet<u32>, &'a mut Vec<u32>) {
    if let Some(type_info) = context.get_type_by_hash(type_hash) {
        insert_value(context, types, post_order, type_info)
    } else {
        (types, post_order)
    }
}

fn insert_value<'a>(
    context: &AdfReflectionContext,
    types: &'a mut HashSet<u32>,
    post_order: &'a mut Vec<u32>,
    value: &AdfType,
) -> (&'a mut HashSet<u32>, &'a mut Vec<u32>) {
    let (types, post_order) = insert(types, post_order, value.type_hash);
    match &value.primitive {
        AdfPrimitive::Structure => {
            for member in value.members.iter() {
                insert_value_by_hash(context, types, post_order, member.type_hash);
            }
            (types, post_order)
        }
        AdfPrimitive::Pointer
        | AdfPrimitive::Array
        | AdfPrimitive::InlineArray
        | AdfPrimitive::Bitfield
        | AdfPrimitive::Enumeration
        | AdfPrimitive::StringHash => {
            insert_value_by_hash(context, types, post_order, value.element_type_hash)
        }
        _ => (types, post_order),
    }
}

fn insert<'a>(
    types: &'a mut HashSet<u32>,
    post_order: &'a mut Vec<u32>,
    type_hash: u32,
) -> (&'a mut HashSet<u32>, &'a mut Vec<u32>) {
    if types.insert(type_hash) {
        post_order.push(type_hash);
    }
    (types, post_order)
}

fn type_name(context: &AdfReflectionContext, type_hash: u32) -> Result<String> {
    let type_info = context
        .get_type_by_hash(type_hash)
        .context("failed to find type: {type_hash}")?;

    let name = match type_info.primitive {
        AdfPrimitive::Scalar => match type_info.size {
            1 => match type_info.scalar_type {
                AdfScalarType::Signed => "i8",
                AdfScalarType::Unsigned => "u8",
                AdfScalarType::Float => bail!(format!(
                    "invalid scalar type ({:?}) for size {}",
                    type_info.scalar_type, type_info.size
                )),
            },
            2 => match type_info.scalar_type {
                AdfScalarType::Signed => "i16",
                AdfScalarType::Unsigned => "u16",
                AdfScalarType::Float => bail!(format!(
                    "invalid scalar type ({:?}) for size {}",
                    type_info.scalar_type, type_info.size
                )),
            },
            4 => match type_info.scalar_type {
                AdfScalarType::Signed => "i32",
                AdfScalarType::Unsigned => "u32",
                AdfScalarType::Float => "f32",
            },
            8 => match type_info.scalar_type {
                AdfScalarType::Signed => "i64",
                AdfScalarType::Unsigned => "u64",
                AdfScalarType::Float => "f64",
            },
            _ => {
                bail!(format!(
                    "invalid scalar type ({:?}) for size {}",
                    type_info.scalar_type, type_info.size
                ))
            }
        },
        AdfPrimitive::Structure | AdfPrimitive::Bitfield | AdfPrimitive::Enumeration => {
            type_info.name.as_str()
        }
        AdfPrimitive::Pointer => &format!(
            "Option<Arc<{}>>",
            type_name(context, type_info.element_type_hash)?
        ),
        AdfPrimitive::Array => &format!(
            "Arc<Vec<{}>>",
            type_name(context, type_info.element_type_hash)?
        ),
        AdfPrimitive::InlineArray => &format!(
            "[{}; {}]",
            type_name(context, type_info.element_type_hash)?,
            type_info.element_length
        ),
        AdfPrimitive::String => "Arc<String>",
        AdfPrimitive::Recursive => todo!(),
        AdfPrimitive::StringHash => "HashString",
        AdfPrimitive::Deferred => "dyn Any",
    };

    Ok(name.into())
}
