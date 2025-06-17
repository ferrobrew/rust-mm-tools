use std::{
    io::{Read, Seek, Write},
    sync::Arc,
};

use mm_file_formats::adf::{
    AdfRead, AdfReadWriteError, AdfReaderReferences, AdfTypeInfo, AdfWrite, AdfWriterReferences,
};
use mm_hashing::HashString;

#[derive(Clone, Default, Debug)]
pub struct EffectRTSystem {
    pub emitter_templates: Arc<Vec<EffectRTEmitterTemplate>>,
    pub emitters: Arc<Vec<EffectRTEmitter>>,
    pub modifiers: Arc<Vec<EffectRTModifier>>,
    pub timelines: Arc<Vec<EffectRTTimeline>>,
    pub instantiators: Arc<Vec<EffectRTInstantiator>>,
    pub special_effects: Arc<Vec<EffectRTSpecialEffect>>,
    pub param_handler: Arc<Vec<EffectRTParamHandler>>,
    pub params: Arc<Vec<f32>>,
    pub output_buffer_descriptor: EffectRTOutputBufferDescriptor,
    pub named_parameters: Arc<Vec<EffectRTLocalParam>>,
    pub named_parameters_size: u32,
    pub dependencies: Arc<Vec<HashString>>,
    pub properties: Arc<Vec<f32>>,
}

impl AdfTypeInfo for EffectRTSystem {
    const NAME: &str = "EffectRTSystem";
    const HASH: u32 = 2046539742;
    const SIZE: u64 = 224;
    const ALIGN: u64 = 8;
}

impl AdfRead for EffectRTSystem {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        Ok(Self {
            emitter_templates: AdfRead::read(reader, references)?,
            emitters: AdfRead::read(reader, references)?,
            modifiers: AdfRead::read(reader, references)?,
            timelines: AdfRead::read(reader, references)?,
            instantiators: AdfRead::read(reader, references)?,
            special_effects: AdfRead::read(reader, references)?,
            param_handler: AdfRead::read(reader, references)?,
            params: AdfRead::read(reader, references)?,
            output_buffer_descriptor: AdfRead::read(reader, references)?,
            named_parameters: AdfRead::read(reader, references)?,
            named_parameters_size: AdfRead::read(reader, references)?,
            dependencies: AdfRead::read(reader, references)?,
            properties: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for EffectRTSystem {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        self.emitter_templates.write(writer, references)?;
        self.emitters.write(writer, references)?;
        self.modifiers.write(writer, references)?;
        self.timelines.write(writer, references)?;
        self.instantiators.write(writer, references)?;
        self.special_effects.write(writer, references)?;
        self.param_handler.write(writer, references)?;
        self.params.write(writer, references)?;
        self.output_buffer_descriptor.write(writer, references)?;
        self.named_parameters.write(writer, references)?;
        self.named_parameters_size.write(writer, references)?;
        self.dependencies.write(writer, references)?;
        self.properties.write(writer, references)?;
        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
pub struct EffectRTEmitterTemplate {
    pub emitter_params: Arc<Vec<u16>>,
    pub emitter_timeline_connections: Arc<Vec<u32>>,
    pub particle_modifiers: Arc<Vec<u8>>,
    pub emitter_lifetime_index: u32,
    pub start_time: u32,
    pub alive_instantiators: Arc<Vec<u8>>,
    pub on_death_instantiators: Arc<Vec<u8>>,
    pub max_particles: u16,
    pub flags: u8,
    pub type_hash: u32,
    pub local_params_size: u32,
    pub local_params: Arc<Vec<EffectRTLocalParam>>,
}

impl AdfTypeInfo for EffectRTEmitterTemplate {
    const NAME: &str = "EffectRTEmitterTemplate";
    const HASH: u32 = 4095853401;
    const SIZE: u64 = 120;
    const ALIGN: u64 = 8;
}

impl AdfRead for EffectRTEmitterTemplate {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        Ok(Self {
            emitter_params: AdfRead::read(reader, references)?,
            emitter_timeline_connections: AdfRead::read(reader, references)?,
            particle_modifiers: AdfRead::read(reader, references)?,
            emitter_lifetime_index: AdfRead::read(reader, references)?,
            start_time: AdfRead::read(reader, references)?,
            alive_instantiators: AdfRead::read(reader, references)?,
            on_death_instantiators: AdfRead::read(reader, references)?,
            max_particles: AdfRead::read(reader, references)?,
            flags: AdfRead::read(reader, references)?,
            type_hash: AdfRead::read(reader, references)?,
            local_params_size: AdfRead::read(reader, references)?,
            local_params: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for EffectRTEmitterTemplate {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        self.emitter_params.write(writer, references)?;
        self.emitter_timeline_connections
            .write(writer, references)?;
        self.particle_modifiers.write(writer, references)?;
        self.emitter_lifetime_index.write(writer, references)?;
        self.start_time.write(writer, references)?;
        self.alive_instantiators.write(writer, references)?;
        self.on_death_instantiators.write(writer, references)?;
        self.max_particles.write(writer, references)?;
        self.flags.write(writer, references)?;
        self.type_hash.write(writer, references)?;
        self.local_params_size.write(writer, references)?;
        self.local_params.write(writer, references)?;
        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
pub struct EffectRTLocalParam {
    pub hash: u32,
    pub index: u16,
    pub num_params: u16,
}

impl AdfTypeInfo for EffectRTLocalParam {
    const NAME: &str = "EffectRTLocalParam";
    const HASH: u32 = 2386475346;
    const SIZE: u64 = 8;
    const ALIGN: u64 = 4;
}

impl AdfRead for EffectRTLocalParam {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        Ok(Self {
            hash: AdfRead::read(reader, references)?,
            index: AdfRead::read(reader, references)?,
            num_params: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for EffectRTLocalParam {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        self.hash.write(writer, references)?;
        self.index.write(writer, references)?;
        self.num_params.write(writer, references)?;
        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
pub struct EffectRTEmitter {
    pub emitter_template_index: u16,
    pub flags: u8,
    pub emitter_modifiers: Arc<Vec<u8>>,
    pub start_in_output_buffer: Arc<Vec<u32>>,
}

impl AdfTypeInfo for EffectRTEmitter {
    const NAME: &str = "EffectRTEmitter";
    const HASH: u32 = 2669023116;
    const SIZE: u64 = 40;
    const ALIGN: u64 = 8;
}

impl AdfRead for EffectRTEmitter {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        Ok(Self {
            emitter_template_index: AdfRead::read(reader, references)?,
            flags: AdfRead::read(reader, references)?,
            emitter_modifiers: AdfRead::read(reader, references)?,
            start_in_output_buffer: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for EffectRTEmitter {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        self.emitter_template_index.write(writer, references)?;
        self.flags.write(writer, references)?;
        self.emitter_modifiers.write(writer, references)?;
        self.start_in_output_buffer.write(writer, references)?;
        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
pub struct EffectRTModifier {
    pub parameters: EffectRTParameters,
    pub type_hash: u32,
    pub flags: u32,
}

impl AdfTypeInfo for EffectRTModifier {
    const NAME: &str = "EffectRTModifier";
    const HASH: u32 = 420215461;
    const SIZE: u64 = 80;
    const ALIGN: u64 = 8;
}

impl AdfRead for EffectRTModifier {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        Ok(Self {
            parameters: AdfRead::read(reader, references)?,
            type_hash: AdfRead::read(reader, references)?,
            flags: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for EffectRTModifier {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        self.parameters.write(writer, references)?;
        self.type_hash.write(writer, references)?;
        self.flags.write(writer, references)?;
        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
pub struct EffectRTParameters {
    pub float_param_indices: Arc<Vec<u16>>,
    pub int32_params: Arc<Vec<i32>>,
    pub local_params: Arc<Vec<EffectRTLocalParam>>,
    pub local_params_size: u32,
    pub timeline_connections: Arc<Vec<u32>>,
}

impl AdfTypeInfo for EffectRTParameters {
    const NAME: &str = "EffectRTParameters";
    const HASH: u32 = 365161415;
    const SIZE: u64 = 72;
    const ALIGN: u64 = 8;
}

impl AdfRead for EffectRTParameters {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        Ok(Self {
            float_param_indices: AdfRead::read(reader, references)?,
            int32_params: AdfRead::read(reader, references)?,
            local_params: AdfRead::read(reader, references)?,
            local_params_size: AdfRead::read(reader, references)?,
            timeline_connections: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for EffectRTParameters {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        self.float_param_indices.write(writer, references)?;
        self.int32_params.write(writer, references)?;
        self.local_params.write(writer, references)?;
        self.local_params_size.write(writer, references)?;
        self.timeline_connections.write(writer, references)?;
        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
pub struct EffectRTTimeline {
    pub control_points_y: [i8; 16],
    pub start_x: [f32; 4],
    pub end_x: [f32; 4],
    pub x_scale_recip: [f32; 4],
}

impl AdfTypeInfo for EffectRTTimeline {
    const NAME: &str = "EffectRTTimeline";
    const HASH: u32 = 3227539342;
    const SIZE: u64 = 64;
    const ALIGN: u64 = 4;
}

impl AdfRead for EffectRTTimeline {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        Ok(Self {
            control_points_y: AdfRead::read(reader, references)?,
            start_x: AdfRead::read(reader, references)?,
            end_x: AdfRead::read(reader, references)?,
            x_scale_recip: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for EffectRTTimeline {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        self.control_points_y.write(writer, references)?;
        self.start_x.write(writer, references)?;
        self.end_x.write(writer, references)?;
        self.x_scale_recip.write(writer, references)?;
        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
pub struct EffectRTInstantiator {
    pub parameters: EffectRTParameters,
    pub render_infos: Arc<Vec<EffectRTRenderInfo>>,
    pub type_hash: u32,
    pub extra_mem_per_particle: u16,
    pub flags: u16,
}

impl AdfTypeInfo for EffectRTInstantiator {
    const NAME: &str = "EffectRTInstantiator";
    const HASH: u32 = 1931526390;
    const SIZE: u64 = 96;
    const ALIGN: u64 = 8;
}

impl AdfRead for EffectRTInstantiator {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        Ok(Self {
            parameters: AdfRead::read(reader, references)?,
            render_infos: AdfRead::read(reader, references)?,
            type_hash: AdfRead::read(reader, references)?,
            extra_mem_per_particle: AdfRead::read(reader, references)?,
            flags: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for EffectRTInstantiator {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        self.parameters.write(writer, references)?;
        self.render_infos.write(writer, references)?;
        self.type_hash.write(writer, references)?;
        self.extra_mem_per_particle.write(writer, references)?;
        self.flags.write(writer, references)?;
        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
pub struct EffectRTRenderInfo {
    pub render_block_type_hash: u32,
    pub vertex_buffer_header_size: u16,
    pub particle_vertex_memory_size: u16,
    pub render_block_data: Arc<Vec<u32>>,
}

impl AdfTypeInfo for EffectRTRenderInfo {
    const NAME: &str = "EffectRTRenderInfo";
    const HASH: u32 = 3296386548;
    const SIZE: u64 = 24;
    const ALIGN: u64 = 8;
}

impl AdfRead for EffectRTRenderInfo {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        Ok(Self {
            render_block_type_hash: AdfRead::read(reader, references)?,
            vertex_buffer_header_size: AdfRead::read(reader, references)?,
            particle_vertex_memory_size: AdfRead::read(reader, references)?,
            render_block_data: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for EffectRTRenderInfo {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        self.render_block_type_hash.write(writer, references)?;
        self.vertex_buffer_header_size.write(writer, references)?;
        self.particle_vertex_memory_size.write(writer, references)?;
        self.render_block_data.write(writer, references)?;
        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
pub struct EffectRTSpecialEffect {
    pub float_param_indices: Arc<Vec<u16>>,
    pub emitter_timeline_connections: Arc<Vec<u32>>,
    pub modifiers: Arc<Vec<u8>>,
    pub params: [i32; 29],
    pub hash: u32,
}

impl AdfTypeInfo for EffectRTSpecialEffect {
    const NAME: &str = "EffectRTSpecialEffect";
    const HASH: u32 = 828991875;
    const SIZE: u64 = 168;
    const ALIGN: u64 = 8;
}

impl AdfRead for EffectRTSpecialEffect {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        Ok(Self {
            float_param_indices: AdfRead::read(reader, references)?,
            emitter_timeline_connections: AdfRead::read(reader, references)?,
            modifiers: AdfRead::read(reader, references)?,
            params: AdfRead::read(reader, references)?,
            hash: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for EffectRTSpecialEffect {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        self.float_param_indices.write(writer, references)?;
        self.emitter_timeline_connections
            .write(writer, references)?;
        self.modifiers.write(writer, references)?;
        self.params.write(writer, references)?;
        self.hash.write(writer, references)?;
        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
pub struct EffectRTParamHandler {
    pub float_param_indices: Arc<Vec<u16>>,
    pub timelines: Arc<Vec<u32>>,
    pub age: f32,
    pub param_hash: Arc<Vec<u32>>,
}

impl AdfTypeInfo for EffectRTParamHandler {
    const NAME: &str = "EffectRTParamHandler";
    const HASH: u32 = 162524244;
    const SIZE: u64 = 56;
    const ALIGN: u64 = 8;
}

impl AdfRead for EffectRTParamHandler {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        Ok(Self {
            float_param_indices: AdfRead::read(reader, references)?,
            timelines: AdfRead::read(reader, references)?,
            age: AdfRead::read(reader, references)?,
            param_hash: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for EffectRTParamHandler {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        self.float_param_indices.write(writer, references)?;
        self.timelines.write(writer, references)?;
        self.age.write(writer, references)?;
        self.param_hash.write(writer, references)?;
        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
pub struct EffectRTOutputBufferDescriptor {
    pub output_buffer_size: u32,
    pub batch_descriptors: Arc<Vec<EffectRTBatchDescriptor>>,
    pub special_effect_data: Arc<Vec<EffectRTSpecialEffectData>>,
}

impl AdfTypeInfo for EffectRTOutputBufferDescriptor {
    const NAME: &str = "EffectRTOutputBufferDescriptor";
    const HASH: u32 = 583442330;
    const SIZE: u64 = 40;
    const ALIGN: u64 = 8;
}

impl AdfRead for EffectRTOutputBufferDescriptor {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        Ok(Self {
            output_buffer_size: AdfRead::read(reader, references)?,
            batch_descriptors: AdfRead::read(reader, references)?,
            special_effect_data: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for EffectRTOutputBufferDescriptor {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        self.output_buffer_size.write(writer, references)?;
        self.batch_descriptors.write(writer, references)?;
        self.special_effect_data.write(writer, references)?;
        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
pub struct EffectRTBatchDescriptor {
    pub output_buffer_offset: u32,
    pub render_block_type_hash: u32,
    pub out_buffer_header_size: u32,
    pub constant_render_block_data: Option<Arc<[u32; 29]>>,
}

impl AdfTypeInfo for EffectRTBatchDescriptor {
    const NAME: &str = "EffectRTBatchDescriptor";
    const HASH: u32 = 4282977304;
    const SIZE: u64 = 24;
    const ALIGN: u64 = 8;
}

impl AdfRead for EffectRTBatchDescriptor {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        Ok(Self {
            output_buffer_offset: AdfRead::read(reader, references)?,
            render_block_type_hash: AdfRead::read(reader, references)?,
            out_buffer_header_size: AdfRead::read(reader, references)?,
            constant_render_block_data: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for EffectRTBatchDescriptor {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        self.output_buffer_offset.write(writer, references)?;
        self.render_block_type_hash.write(writer, references)?;
        self.out_buffer_header_size.write(writer, references)?;
        self.constant_render_block_data.write(writer, references)?;
        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
pub struct EffectRTSpecialEffectData {
    pub constant_data: Arc<Vec<u32>>,
    pub hash: u32,
}

impl AdfTypeInfo for EffectRTSpecialEffectData {
    const NAME: &str = "EffectRTSpecialEffectData";
    const HASH: u32 = 2251641537;
    const SIZE: u64 = 24;
    const ALIGN: u64 = 8;
}

impl AdfRead for EffectRTSpecialEffectData {
    #[inline]
    fn read<R: Read + Seek>(
        reader: &mut R,
        references: &mut AdfReaderReferences,
    ) -> Result<Self, AdfReadWriteError> {
        Ok(Self {
            constant_data: AdfRead::read(reader, references)?,
            hash: AdfRead::read(reader, references)?,
        })
    }
}

impl AdfWrite for EffectRTSpecialEffectData {
    #[inline]
    fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        references: &mut AdfWriterReferences,
    ) -> Result<(), AdfReadWriteError> {
        self.constant_data.write(writer, references)?;
        self.hash.write(writer, references)?;
        Ok(())
    }
}
