use std::io::Write;

use anyhow::{anyhow, bail, Context};
use binrw::{BinRead, BinWrite};
use clap::Parser;
use serde::{Deserialize, Serialize};

use mm_file_formats::adf::{
    AdfFile, AdfReflectionContext, AdfXml, BUILT_IN_TYPE_LIBRARY, TYPE_LIBRARIES,
};

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

    if extension == "xml" {
        // Parse the XML
        let mut deserializer = quick_xml::de::Deserializer::from_reader(reader);
        let adf = AdfXml::deserialize(&mut deserializer)?;

        // Load types based on extension
        load_types(&mut context, &adf.extension)?;

        // Write ADF
        let output = adf.convert(&context);
        let file = std::fs::File::create(args.file.with_extension(""))?;
        let mut writer = std::io::BufWriter::new(file);
        output.write_le(&mut writer)?;
    } else {
        // Load types based on extension
        load_types(&mut context, extension)?;

        // Parse the ADF, intentionally not loading additional types
        let adf = AdfFile::read_le(&mut reader).context("Failed to parse ADF")?;

        // Configure XML serializer
        let mut buffer = String::new();
        let mut serializer = quick_xml::se::Serializer::with_root(&mut buffer, Some("adf"))?;
        serializer.indent('\t', 1);
        serializer.expand_empty_elements(true);

        // Write XML
        AdfXml::new(&adf, &context, extension).serialize(serializer)?;
        let mut file = std::fs::File::create(args.file.with_extension(format!("{extension}.xml")))?;
        file.write_all(buffer.as_bytes())?;
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
