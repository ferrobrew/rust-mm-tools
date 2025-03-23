use std::collections::{HashMap, HashSet};

use super::{AdfFile, AdfMemberValue, AdfPrimitive, AdfScalarType, AdfType};
use serde::{Deserialize, Serialize};

use super::reflection::{
    AdfReflectedPrimitive, AdfReflectedScalar, AdfReflectedValue, AdfReflectionContext,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct AdfXml {
    #[serde(rename = "@extension")]
    pub extension: String,
    #[serde(
        rename = "@embedded-types",
        skip_serializing_if = "<&bool as std::ops::Not>::not",
        default
    )]
    pub embedded_types: bool,
    #[serde(rename = "type", default)]
    pub types: Vec<AdfXmlType>,
    #[serde(rename = "instance", default)]
    pub instances: Vec<AdfXmlValue>,
}

impl AdfXml {
    pub fn new(adf: &AdfFile, context: &AdfReflectionContext, extension: &str) -> Self {
        // Reflect instances
        let instances: Vec<(&str, AdfReflectedValue)> = adf
            .instances
            .iter()
            .filter_map(|instance| {
                // TODO: Throw an error when `reflect_instance` uses results
                context
                    .read_instance(&instance)
                    .ok()
                    .map(|x| (instance.name.as_ref(), x))
            })
            .collect();

        // Collect used types and build a list of unique names
        let types = collect_types(instances.iter().map(|instance| &instance.1));
        let names: HashMap<u32, String> = types
            .iter()
            .filter_map(|&type_hash| type_name(type_hash, &context).map(|name| (type_hash, name)))
            .collect();

        Self {
            extension: extension.to_string(),
            embedded_types: !adf.types.is_empty(),
            types: {
                let mut types = names
                    .iter()
                    .map(|(&type_hash, type_name)| AdfXmlType {
                        type_name: type_name.to_string(),
                        type_hash,
                    })
                    .collect::<Vec<AdfXmlType>>();
                types.sort_by(|a, b| a.type_name.cmp(&b.type_name));
                types
            },
            instances: {
                instances
                    .iter()
                    .map(|instance| {
                        AdfXmlValue::from_value_named(
                            &instance.1,
                            instance.0.to_string(),
                            context,
                            &names,
                        )
                    })
                    .collect::<Vec<AdfXmlValue>>()
            },
        }
    }

    pub fn convert(&self, context: &AdfReflectionContext) -> AdfFile {
        let mut result = AdfFile::default();

        // Build type look up
        let types: HashMap<&str, u32> = self
            .types
            .iter()
            .map(|type_info| (type_info.type_name.as_ref(), type_info.type_hash))
            .collect();

        // Insert embedded types
        if self.embedded_types {
            result.types = self
                .types
                .iter()
                .filter_map(|type_info| {
                    let Some(type_info) = context.get_type(type_info.type_hash) else {
                        todo!("failed to find type: {}", type_info.type_name);
                    };

                    (!matches!(
                        type_info.primitive,
                        // We can skip types that only exist in builtin_types.adf
                        AdfPrimitive::Scalar | AdfPrimitive::String | AdfPrimitive::Deferred
                    ))
                    .then(|| {
                        let mut type_info = type_info.clone();
                        // We don't need default values when writing
                        for member in type_info.members.iter_mut() {
                            member.value = AdfMemberValue::UninitializedValue(());
                        }
                        type_info
                    })
                })
                .collect();
        }

        // Reconstruct reflected instances
        let instances: Vec<(&str, AdfReflectedValue)> = self
            .instances
            .iter()
            .map(|instance| {
                (
                    instance
                        .name
                        .as_ref()
                        .expect("instance must have name")
                        .as_ref(),
                    instance.to_value(&types, context),
                )
            })
            .collect();

        // Create final instance buffers
        for instance in instances {
            context.write_instance(&instance.0, &instance.1, &mut result);
        }

        result
    }
}

fn collect_types<'a>(values: impl Iterator<Item = &'a AdfReflectedValue>) -> HashSet<u32> {
    let mut default = HashSet::<u32>::default();
    fold_values(&mut default, values);
    default
}

fn fold_values<'a>(
    types: &mut HashSet<u32>,
    values: impl Iterator<Item = &'a AdfReflectedValue>,
) -> &mut HashSet<u32> {
    values.fold(types, |types, value| insert_value(types, value))
}

fn insert_value<'a>(
    types: &'a mut HashSet<u32>,
    value: &AdfReflectedValue,
) -> &'a mut HashSet<u32> {
    let types = match &value.1 {
        AdfReflectedPrimitive::Structure(values) => fold_values(types, values.iter()),
        AdfReflectedPrimitive::Pointer(value) => insert_value(types, &value),
        AdfReflectedPrimitive::Array(values) => fold_values(types, values.iter()),
        AdfReflectedPrimitive::InlineArray(values) => fold_values(types, values.iter()),
        AdfReflectedPrimitive::Deferred(value) => insert_value(types, &value),
        _ => types,
    };
    insert(types, value.0)
}

fn insert(types: &mut HashSet<u32>, type_hash: u32) -> &mut HashSet<u32> {
    types.insert(type_hash);
    types
}

fn type_name(type_hash: u32, context: &AdfReflectionContext) -> Option<String> {
    context
        .get_type(type_hash)
        .and_then(|type_info| match type_info.primitive {
            AdfPrimitive::Scalar => scalar_name(type_info).map(str::to_string),
            AdfPrimitive::Structure => Some(type_info.name.to_string()),
            AdfPrimitive::Pointer => type_name(type_info.element_type_hash, context)
                .map(|name| format!("Pointer[{}]", name).into()),
            AdfPrimitive::Array => type_name(type_info.element_type_hash, context)
                .map(|name| format!("[{}]", name).into()),
            AdfPrimitive::InlineArray => type_name(type_info.element_type_hash, context)
                .map(|name| format!("[{}; {}]", name, type_info.element_length).clone()),
            AdfPrimitive::String => Some("String".into()),
            AdfPrimitive::Recursive => type_name(type_info.element_type_hash, context)
                .map(|name| format!("Recursive[{}]", name).clone()),
            AdfPrimitive::Bitfield => scalar_name(type_info)
                .map(|name| format!("{}: {}", name, type_info.element_length).into()),
            AdfPrimitive::Enumeration => scalar_name(type_info)
                .map(|name| format!("{}: {}", type_info.name.as_ref(), name).into()),
            AdfPrimitive::StringHash => {
                scalar_name(type_info).map(|name| format!("Hash[{}]", name).into())
            }
            AdfPrimitive::Deferred => Some("Any".into()),
        })
}

fn scalar_name(type_info: &AdfType) -> Option<&str> {
    match type_info.scalar_type {
        AdfScalarType::Signed => match type_info.size {
            1 => Some("i8".into()),
            2 => Some("i16".into()),
            4 => Some("i32".into()),
            8 => Some("i64".into()),
            _ => None,
        },
        AdfScalarType::Unsigned => match type_info.size {
            1 => Some("u8".into()),
            2 => Some("u16".into()),
            4 => Some("u32".into()),
            8 => Some("u64".into()),
            _ => None,
        },
        AdfScalarType::Float => match type_info.size {
            4 => Some("f32".into()),
            8 => Some("f64".into()),
            _ => None,
        },
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AdfXmlType {
    #[serde(rename = "@name")]
    pub type_name: String,
    #[serde(rename = "$text")]
    pub type_hash: u32,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct AdfXmlValue {
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    #[serde(rename = "@type")]
    pub type_name: String,
    #[serde(rename = "member", default)]
    pub members: Vec<AdfXmlValue>,
    #[serde(rename = "value", default)]
    pub values: Vec<AdfXmlValue>,
    #[serde(rename = "$text", skip_serializing_if = "String::is_empty", default)]
    pub value: String,
}

impl AdfXmlValue {
    pub fn from_value(
        value: &AdfReflectedValue,
        context: &AdfReflectionContext,
        names: &HashMap<u32, String>,
    ) -> Self {
        // TODO: Throw an error instead of unwrapping
        let type_info = context.get_type(value.0).unwrap();
        let type_name = names.get(&value.0).unwrap().clone();

        let mut result = Self {
            type_name,
            ..Default::default()
        };

        match &value.1 {
            AdfReflectedPrimitive::Scalar(scalar) => {
                result.value = scalar_string(scalar);
            }
            AdfReflectedPrimitive::Structure(values) => {
                result.members.reserve(values.len());
                for (name, value) in type_info
                    .members
                    .iter()
                    .map(|x| x.name.as_ref())
                    .zip(values.iter())
                {
                    result.members.push(Self::from_value_named(
                        value,
                        name.to_string(),
                        context,
                        names,
                    ));
                }
            }
            AdfReflectedPrimitive::Pointer(value) => {
                result.values.push(Self::from_value(value, context, names));
            }
            AdfReflectedPrimitive::Array(values) => {
                result.values.reserve(values.len());
                for value in values.iter() {
                    result.values.push(Self::from_value(value, context, names));
                }
            }
            AdfReflectedPrimitive::InlineArray(values) => {
                result.values.reserve(values.len());
                for value in values.iter() {
                    result.values.push(Self::from_value(value, context, names));
                }
            }
            AdfReflectedPrimitive::String(string) => {
                result.value = string.to_string();
            }
            AdfReflectedPrimitive::Bitfield(scalar) => {
                result.value = scalar_string(scalar);
            }
            AdfReflectedPrimitive::Enumeration(scalar) => {
                // TODO: Write enum value name / bitmask
                result.value = scalar_string(scalar);
            }
            AdfReflectedPrimitive::StringHash(scalar) => {
                // TODO: Write string instead of scalar if we can
                result.value = scalar_string(scalar);
            }
            AdfReflectedPrimitive::Deferred(value) => {
                result.values.push(Self::from_value(value, context, names));
            }
        };

        result
    }

    pub fn from_value_named(
        value: &AdfReflectedValue,
        name: String,
        context: &AdfReflectionContext,
        names: &HashMap<u32, String>,
    ) -> Self {
        let mut result = Self::from_value(value, context, names);
        result.name = Some(name);
        result
    }

    pub fn to_value(
        &self,
        types: &HashMap<&str, u32>,
        context: &AdfReflectionContext,
    ) -> AdfReflectedValue {
        let Some(type_info) = types
            .get(self.type_name.as_str())
            .and_then(|&type_hash| context.get_type(type_hash))
        else {
            todo!("failed to find type: {}", self.type_name);
        };

        let primitive = match type_info.primitive {
            AdfPrimitive::Scalar => {
                AdfReflectedPrimitive::Scalar(scalar_value(&self.value, type_info))
            }
            AdfPrimitive::Structure => AdfReflectedPrimitive::Structure(
                self.members
                    .iter()
                    .map(|member| member.to_value(types, context))
                    .collect(),
            ),
            AdfPrimitive::Pointer => AdfReflectedPrimitive::Pointer(
                self.values
                    .get(0)
                    .expect("pointer must have only one value")
                    .to_value(types, context)
                    .into(),
            ),
            AdfPrimitive::Array => AdfReflectedPrimitive::Array(
                self.values
                    .iter()
                    .map(|value| value.to_value(types, context))
                    .collect::<Vec<_>>()
                    .into(),
            ),
            AdfPrimitive::InlineArray => AdfReflectedPrimitive::InlineArray(
                self.values
                    .iter()
                    .map(|value| value.to_value(types, context))
                    .collect(),
            ),
            AdfPrimitive::String => AdfReflectedPrimitive::String(self.value.clone().into()),
            AdfPrimitive::Recursive => todo!("recursive is not yet supported!"),
            AdfPrimitive::Bitfield => {
                AdfReflectedPrimitive::Bitfield(scalar_value(&self.value, type_info))
            }
            AdfPrimitive::Enumeration => {
                AdfReflectedPrimitive::Enumeration(scalar_value(&self.value, type_info))
            }
            AdfPrimitive::StringHash => {
                AdfReflectedPrimitive::StringHash(scalar_value(&self.value, type_info))
            }
            AdfPrimitive::Deferred => AdfReflectedPrimitive::Deferred(
                self.values
                    .get(0)
                    .expect("deferred must have only one value")
                    .to_value(types, context)
                    .into(),
            ),
        };
        AdfReflectedValue(type_info.type_hash, primitive)
    }
}

fn scalar_string(scalar: &AdfReflectedScalar) -> String {
    match scalar {
        AdfReflectedScalar::U8(value) => format!("{value}"),
        AdfReflectedScalar::I8(value) => format!("{value}"),
        AdfReflectedScalar::U16(value) => format!("{value}"),
        AdfReflectedScalar::I16(value) => format!("{value}"),
        AdfReflectedScalar::U32(value) => format!("{value}"),
        AdfReflectedScalar::I32(value) => format!("{value}"),
        AdfReflectedScalar::F32(value) => format!("{value}"),
        AdfReflectedScalar::U64(value) => format!("{value}"),
        AdfReflectedScalar::I64(value) => format!("{value}"),
        AdfReflectedScalar::F64(value) => format!("{value}"),
    }
}

fn scalar_value(scalar: &str, type_info: &AdfType) -> AdfReflectedScalar {
    macro_rules! parse {
        ($t:tt) => {
            match scalar.parse::<$t>() {
                Ok(value) => value,
                _ => todo!("failed to parse {}: {}", stringify!($t), scalar),
            }
        };
    }
    match type_info.scalar_type {
        AdfScalarType::Signed => match type_info.size {
            1 => AdfReflectedScalar::I8(parse!(i8)),
            2 => AdfReflectedScalar::I16(parse!(i16)),
            4 => AdfReflectedScalar::I32(parse!(i32)),
            8 => AdfReflectedScalar::I64(parse!(i64)),
            size => todo!("unexpected scalar size: {}", size),
        },
        AdfScalarType::Unsigned => match type_info.size {
            1 => AdfReflectedScalar::U8(parse!(u8)),
            2 => AdfReflectedScalar::U16(parse!(u16)),
            4 => AdfReflectedScalar::U32(parse!(u32)),
            8 => AdfReflectedScalar::U64(parse!(u64)),
            size => todo!("unexpected scalar size: {}", size),
        },
        AdfScalarType::Float => match type_info.size {
            4 => AdfReflectedScalar::F32(parse!(f32)),
            8 => AdfReflectedScalar::F64(parse!(f64)),
            size => todo!("unexpected scalar size: {}", size),
        },
    }
}
