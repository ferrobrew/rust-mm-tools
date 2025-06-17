use std::io::Write;

use anyhow::{bail, Context};
use binrw::{BinRead, BinWrite};
use clap::Parser;
use serde::{Deserialize, Serialize};

use mm_file_formats::adf::AdfFile;

mod adf;
use adf::EffectRTSystem;

mod emitter;
use emitter::{
    BoxEmitter, CylinderEmitter, FlareEmitter, SpecialEffectEmitter, SphericalEmitter,
    SplineEmitter,
};

mod instantiator;
use instantiator::{
    CameraFacingBillboard, CameraFacingBillboardBlend, DecalEffect, EmbeddedEffect,
    FullScreenEffect, LightEffect, ModelSpawn, ModelSpawnOnBirth, NormalMappedBillboard,
    SimpleTrailInstantiator, SoundEffect, SoundFocusEffect, SoundMixerEffect, TrailDespawn,
    TrailEffect, VibrationEffect, WorldFacingBillboard, WorldFacingBillboardBlend,
};

mod modifier;
use modifier::{
    AdjustByPositionModifier, CameraVelocityEmitterModifier, ColorModulateModifier,
    ColorOpacityModifier, ContinuesProjectToTerrainModifier, DampingAngularVelocityModifier,
    DampingModifier, EmitterFeedbackModifier, FlareIrisModifier, GravitationModifier,
    GravityPointModifier, HueSatLumModulateModifier, InheritVelocityEmitterModifier,
    LocalWindModifier, MaterialEmitterModifier, NoiseModifier, OffsetEmitterModifier,
    OnBirthProjectToTerrainModifier, ParticleFadeBoxModifier, ParticleFeedbackModifier,
    PlaneCollisionModifier, RotationModifier, SizeModifier, SphereCollisionModifier,
    SplinePositionModifier, VariableDisableEmitterModifier, VortexModifier, WindModifier,
};

mod xml;
use xml::{
    XmlEffectRTEmitterTemplate, XmlEffectRTEmitterTemplateParam, XmlEffectRTInstantiator,
    XmlEffectRTInstantiatorParam, XmlEffectRTModifier, XmlEffectRTModifierParam, XmlEffectRTSystem,
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

    match extension {
        "effc" => {
            // Parse the ADF
            let adf = AdfFile::read_le(&mut reader).context("Failed to parse ADF")?;

            // Find associated ADF instance
            let instance = adf
                .get_instance_by_info::<EffectRTSystem>("RuntimeEffect")
                .context("failed to find matching instance")?;

            // Read associated instance
            let effect: EffectRTSystem = instance.read()?;

            fn parameter<T: Into<usize>>(
                effect: &EffectRTSystem,
                indices: &[u16],
                param: T,
            ) -> Option<f32> {
                indices
                    .get(param.into())
                    .and_then(|x| effect.params.get(*x as usize))
                    .cloned()
            }

            macro_rules! emitter_templates {
                ($type:ident, $template:ident) => {
                    XmlEffectRTEmitterTemplate {
                        type_name: $type::NAME.into(),
                        lifetime: effect.params[($template.emitter_lifetime_index & 0xFFFF) as usize],
                        lifetime_spread: effect.params[($template.emitter_lifetime_index >> 16) as usize],
                        start_time: effect.params[($template.start_time & 0xFFFF) as usize],
                        start_time_spread: effect.params[($template.start_time >> 16) as usize],
                        max_particles: $template.max_particles,
                        emitter_params: $type::PARAMETERS
                            .iter()
                            .map_while(|param| {
                                parameter(&effect, &$template.emitter_params, *param).map(|value| {
                                    XmlEffectRTEmitterTemplateParam {
                                        name: format!("{:?}", param),
                                        value: value,
                                    }
                                })
                            })
                            .collect::<Vec<_>>()
                            .into()
                    }
                };
                ($template:ident, [$($type:ident,)+]) => {
                    match $template.type_hash {
                        $($type::HASH => Some(emitter_templates!($type, $template)),)*
                        _ => None
                    }
                };
            }

            macro_rules! modifiers {
                ($type:ident, $modifier:ident) => {
                    XmlEffectRTModifier {
                        type_name: $type::NAME.into(),
                        float_params: $type::PARAMETERS
                            .iter()
                            .map_while(|param| {
                                parameter(&effect, &$modifier.parameters.float_param_indices, *param).map(|value| {
                                    XmlEffectRTModifierParam {
                                        name: format!("{:?}", param),
                                        value: value,
                                    }
                                })
                            })
                            .collect::<Vec<_>>()
                            .into()
                    }
                };
                ($modifier:ident, [$($type:ident,)+]) => {
                    match $modifier.type_hash {
                        $($type::HASH => Some(modifiers!($type, $modifier)),)*
                        _ => None
                    }
                };
            }

            macro_rules! instantiators {
                ($type:ident, $instantiator:ident) => {
                    XmlEffectRTInstantiator {
                        type_name: $type::NAME.into(),
                        float_params: $type::PARAMETERS
                            .iter()
                            .map_while(|param| {
                                parameter(&effect, &$instantiator.parameters.float_param_indices, *param).map(|value| {
                                    XmlEffectRTInstantiatorParam {
                                        name: format!("{:?}", param),
                                        value: value,
                                    }
                                })
                            })
                            .collect::<Vec<_>>()
                            .into()
                    }
                };
                ($instantiator:ident, [$($type:ident,)+]) => {
                    match $instantiator.type_hash {
                        $($type::HASH => Some(instantiators!($type, $instantiator)),)*
                        _ => None
                    }
                };
            }

            let xml_effect = XmlEffectRTSystem {
                num_params: effect.params.len(),
                emitter_templates: effect
                    .emitter_templates
                    .iter()
                    .map_while(|template| {
                        emitter_templates!(
                            template,
                            [
                                BoxEmitter,
                                CylinderEmitter,
                                FlareEmitter,
                                SpecialEffectEmitter,
                                SphericalEmitter,
                                SplineEmitter,
                            ]
                        )
                    })
                    .collect::<Vec<_>>()
                    .into(),
                modifiers: effect
                    .modifiers
                    .iter()
                    .map_while(|modifier| {
                        modifiers!(
                            modifier,
                            [
                                AdjustByPositionModifier,
                                CameraVelocityEmitterModifier,
                                ColorModulateModifier,
                                ColorOpacityModifier,
                                ContinuesProjectToTerrainModifier,
                                DampingAngularVelocityModifier,
                                DampingModifier,
                                EmitterFeedbackModifier,
                                FlareIrisModifier,
                                GravitationModifier,
                                GravityPointModifier,
                                HueSatLumModulateModifier,
                                InheritVelocityEmitterModifier,
                                LocalWindModifier,
                                MaterialEmitterModifier,
                                NoiseModifier,
                                OffsetEmitterModifier,
                                OnBirthProjectToTerrainModifier,
                                ParticleFadeBoxModifier,
                                ParticleFeedbackModifier,
                                PlaneCollisionModifier,
                                RotationModifier,
                                SizeModifier,
                                SphereCollisionModifier,
                                SplinePositionModifier,
                                VariableDisableEmitterModifier,
                                VortexModifier,
                                WindModifier,
                            ]
                        )
                    })
                    .collect::<Vec<_>>()
                    .into(),
                instantiators: effect
                    .instantiators
                    .iter()
                    .map_while(|instantiator| {
                        instantiators!(
                            instantiator,
                            [
                                CameraFacingBillboard,
                                CameraFacingBillboardBlend,
                                DecalEffect,
                                EmbeddedEffect,
                                FullScreenEffect,
                                LightEffect,
                                ModelSpawn,
                                ModelSpawnOnBirth,
                                NormalMappedBillboard,
                                SimpleTrailInstantiator,
                                SoundEffect,
                                SoundFocusEffect,
                                SoundMixerEffect,
                                TrailDespawn,
                                TrailEffect,
                                VibrationEffect,
                                WorldFacingBillboard,
                                WorldFacingBillboardBlend,
                            ]
                        )
                    })
                    .collect::<Vec<_>>()
                    .into(),
            };

            // Configure XML serializer
            let mut buffer = String::new();
            let mut serializer = quick_xml::se::Serializer::with_root(&mut buffer, Some("effect"))?;
            serializer.indent('\t', 1);
            serializer.expand_empty_elements(true);

            // Write XML
            xml_effect.serialize(serializer)?;
            let mut file = std::fs::File::create(args.file.with_extension("xml"))?;
            file.write_all(buffer.as_bytes())?;
        }
        "xml" => {
            // Parse the XML
            let mut deserializer = quick_xml::de::Deserializer::from_reader(reader);
            let effect = XmlEffectRTSystem::deserialize(&mut deserializer)?;

            // Parse the ADF
            let file = std::fs::File::open(args.file.with_extension("effc"))
                .context("Failed to open effc")?;
            let mut reader = std::io::BufReader::new(file);
            let adf = AdfFile::read_le(&mut reader).context("Failed to parse ADF")?;

            // Find associated ADF instance
            let instance = adf
                .get_instance_by_info::<EffectRTSystem>("RuntimeEffect")
                .context("failed to find matching instance")?;

            // Read associated instance
            let mut adf_effect: EffectRTSystem = instance.read()?;

            // Clone params
            let mut params = adf_effect
                .params
                .iter()
                .cloned()
                .take(effect.num_params)
                .collect::<Vec<_>>();
            let mut create_param = |value: &f32| {
                params.iter().position(|x| x == value).unwrap_or_else(|| {
                    params.push(*value);
                    params.len()
                })
            };

            // Update templates
            adf_effect.emitter_templates = effect
                .emitter_templates
                .iter()
                .enumerate()
                .map(|(i, template)| {
                    let mut adf_template = adf_effect.emitter_templates[i].clone();

                    let a = create_param(&template.lifetime);
                    let b = create_param(&template.lifetime_spread);
                    adf_template.emitter_lifetime_index = (a | b << 16) as u32;

                    let a = create_param(&template.start_time);
                    let b = create_param(&template.start_time_spread);
                    adf_template.start_time = (a | b << 16) as u32;

                    adf_template.max_particles = template.max_particles;

                    adf_template.emitter_params = template
                        .emitter_params
                        .iter()
                        .map(|param| create_param(&param.value) as u16)
                        .collect::<Vec<_>>()
                        .into();
                    adf_template
                })
                .collect::<Vec<_>>()
                .into();

            // Update modifiers
            adf_effect.modifiers = effect
                .modifiers
                .iter()
                .enumerate()
                .map(|(i, modifier)| {
                    let mut adf_modifier = adf_effect.modifiers[i].clone();
                    adf_modifier.parameters.float_param_indices = modifier
                        .float_params
                        .iter()
                        .map(|param| create_param(&param.value) as u16)
                        .collect::<Vec<_>>()
                        .into();
                    adf_modifier
                })
                .collect::<Vec<_>>()
                .into();

            // Update templates
            adf_effect.instantiators = effect
                .instantiators
                .iter()
                .enumerate()
                .map(|(i, instantiator)| {
                    let mut adf_instantiator = adf_effect.instantiators[i].clone();
                    adf_instantiator.parameters.float_param_indices = instantiator
                        .float_params
                        .iter()
                        .map(|param| create_param(&param.value) as u16)
                        .collect::<Vec<_>>()
                        .into();
                    adf_instantiator
                })
                .collect::<Vec<_>>()
                .into();

            // Update params
            adf_effect.params = params.into();

            // Write effect to existing instance
            instance.write(&adf_effect)?;

            // Finally write it to disk
            let mut file = std::fs::File::create(args.file.with_extension("effc"))?;
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
