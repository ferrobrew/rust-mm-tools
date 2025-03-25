use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmlBook {
    #[serde(rename = "sheet", default)]
    pub sheets: Vec<XmlSheet>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmlSheet {
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "row", default)]
    pub rows: Vec<XmlRow>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmlRow {
    #[serde(rename = "cell", default)]
    pub cells: Vec<XmlCell>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmlCell {
    #[serde(rename = "@type", default)]
    pub kind: XmlCellKind,
    #[serde(rename = "@fg-color", default)]
    pub foreground_color: u32,
    #[serde(rename = "@bg-color", default)]
    pub background_color: u32,
    #[serde(rename = "$text", skip_serializing_if = "String::is_empty", default)]
    pub value: String,
}

#[repr(u16)]
#[derive(Debug, Default, Deserialize, Serialize)]
pub enum XmlCellKind {
    #[default]
    Bool,
    String,
    Value,
    Date,
    Color,
}
