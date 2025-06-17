use serde::{Deserialize, Serialize};

macro_rules! xml_wrapper {
    ($name:ident, $item:ident, $rename:literal) => {
        #[derive(Debug, Default, Deserialize, Serialize)]
        pub struct $name {
            #[serde(rename = $rename, default)]
            value: Vec<$item>,
        }

        impl std::ops::Deref for $name {
            type Target = Vec<$item>;
            fn deref(&self) -> &Self::Target {
                &self.value
            }
        }

        impl From<Vec<$item>> for $name {
            fn from(value: Vec<$item>) -> Self {
                Self { value }
            }
        }

        impl From<$name> for Vec<$item> {
            fn from(wrapper: $name) -> Vec<$item> {
                wrapper.value
            }
        }
    };
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmlEffectRTSystem {
    #[serde(rename = "@num_params", default)]
    pub num_params: usize,
    #[serde(rename = "templates", default)]
    pub emitter_templates: XmlEffectRTEmitterTemplates,
    pub modifiers: XmlEffectRTModifiers,
    pub instantiators: XmlEffectRTInstantiators,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmlEffectRTEmitterTemplate {
    #[serde(rename = "@type", default)]
    pub type_name: String,
    pub lifetime: f32,
    pub lifetime_spread: f32,
    pub start_time: f32,
    pub start_time_spread: f32,
    pub max_particles: u16,
    #[serde(rename = "parameters", default)]
    pub emitter_params: XmlEffectRTEmitterTemplateParams,
}
xml_wrapper!(
    XmlEffectRTEmitterTemplates,
    XmlEffectRTEmitterTemplate,
    "template"
);

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmlEffectRTModifier {
    #[serde(rename = "@type", default)]
    pub type_name: String,
    #[serde(rename = "parameters", default)]
    pub float_params: XmlEffectRTModifierParams,
}
xml_wrapper!(XmlEffectRTModifiers, XmlEffectRTModifier, "modifier");

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct XmlEffectRTInstantiator {
    #[serde(rename = "@type", default)]
    pub type_name: String,
    #[serde(rename = "parameters", default)]
    pub float_params: XmlEffectRTInstantiatorParams,
}
xml_wrapper!(
    XmlEffectRTInstantiators,
    XmlEffectRTInstantiator,
    "instantiator"
);

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct XmlEffectRTFloatParam {
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "$text", default)]
    pub value: f32,
}

pub type XmlEffectRTEmitterTemplateParam = XmlEffectRTFloatParam;
xml_wrapper!(
    XmlEffectRTEmitterTemplateParams,
    XmlEffectRTEmitterTemplateParam,
    "parameter"
);

pub type XmlEffectRTModifierParam = XmlEffectRTFloatParam;
xml_wrapper!(
    XmlEffectRTModifierParams,
    XmlEffectRTModifierParam,
    "parameter"
);

pub type XmlEffectRTInstantiatorParam = XmlEffectRTFloatParam;
xml_wrapper!(
    XmlEffectRTInstantiatorParams,
    XmlEffectRTInstantiatorParam,
    "parameter"
);
