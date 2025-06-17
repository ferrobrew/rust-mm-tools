macro_rules! emitters {
    ($name:ident: [$($parameter:ident),* $(,)?]) => {
        #[allow(non_snake_case)]
        pub mod $name {
            #[allow(unused)]
            pub const NAME: &'static str = stringify!($name);

            #[allow(unused)]
            pub const HASH: u32 = mm_hashing::hash_little32(NAME.as_bytes());

            #[allow(unused)]
            pub const PARAMETERS: &'static [Parameter] = &[
                $(Parameter::$parameter,)*
            ];

            #[allow(unused)]
            #[derive(Debug, Clone, Copy)]
            pub enum Parameter {
                $($parameter,)*
            }

            impl Into<usize> for Parameter {
                fn into(self) -> usize {
                    self as usize
                }
            }
        }
    };
    ($($name:ident: [$($parameter:ident),* $(,)?]),+ $(,)?) => {
        $(
            emitters!($name: [$($parameter,)*]);
        )*
    };
}

emitters!(
    AdjustByPositionModifier: [
        FalloffCurve,
        RadiusInverse,
        Scale,
        ColorR,
        ColorG,
        ColorB,
        ColorA,
        ColorBrightness,
        DepthScale,
        AngleOpacity,
        AngleScale,
    ],
    CameraVelocityEmitterModifier: [],
    ColorModulateModifier: [
        ColorR,
        ColorG,
        ColorB,
        ColorA,
        ColorBrightness,
    ],
    ColorOpacityModifier: [
        ColorR,
        ColorG,
        ColorB,
        ColorA,
        ColorBrightness,
    ],
    ContinuesProjectToTerrainModifier: [
        Offset,
        Saturation,
        Value,
        TerrainColorAmount,
        MinClampY,
        MaxClampY,
        Reflection,
        ReflectionOmega,
        HorizontalDamping,
        VerticalDamping,
    ],
    DampingAngularVelocityModifier: [
        Damping,
    ],
    DampingModifier: [
        Damping,
    ],
    EmitterFeedbackModifier: [
        MinIn,
        MaxIn,
        MaxOut,
    ],
    FlareIrisModifier: [
        Distance,
        DistanceSpread,
        OffsetX,
        OffsetXSpread,
        OffsetY,
        OffsetYSpread,
        FadeStart,
        FadeDistanceInverse,
    ],
    GravitationModifier: [
        Gravity,
        GravitySpread,
    ],
    GravityPointModifier: [
        Strength,
        RadiusReciprocal,
        PositionX,
        PositionY,
        PositionZ,
    ],
    HueSatLumModulateModifier: [
        Hue,
        Saturation,
        MaxOut,
        Opacity,
    ],
    InheritVelocityEmitterModifier: [
        Multiplier,
        MultiplierSpread,
    ],
    LocalWindModifier: [
        VelocityMultiplier,
        AngularVelocityMultiplier,
    ],
    MaterialEmitterModifier: [],
    NoiseModifier: [
        Scale,
        Intensity,
        Drag,
        NoiseX,
        NoiseY,
        NoiseZ,
        Animation,
    ],
    OffsetEmitterModifier: [
        PositionX,
        PositionY,
        PositionZ,
        RotationX,
        RotationY,
        RotationZ,
    ],
    OnBirthProjectToTerrainModifier: [
        Offset,
        Saturation,
        Value,
        TerrainColorAmount,
        MinClampY,
        MaxClampY,
        Reflection,
        ReflectionOmega,
        HorizontalDamping,
        VerticalDamping,
    ],
    ParticleFadeBoxModifier: [
        Softness,
        Margin,
    ],
    ParticleFeedbackModifier: [
        MinIn,
        MaxIn,
        MaxOut,
    ],
    PlaneCollisionModifier: [
        PositionX,
        PositionY,
        PositionZ,
        Thickness,
        VelocityMultiplier,
        AngularVelocityMultiplier,
    ],
    RotationModifier: [
        AngularVelocityX,
        AngularVelocityY,
        AngularVelocityZ,
    ],
    SizeModifier: [
        Size,
    ],
    SphereCollisionModifier: [
        PositionX,
        PositionY,
        PositionZ,
        InnerRadius,
        VelocityMultiplier,
        OuterRadius,
        SoftForce,
    ],
    SplinePositionModifier: [],
    VariableDisableEmitterModifier: [
        MinIn,
        MaxIn,
    ],
    VortexModifier: [
        PositionX,
        PositionY,
        PositionZ,
        OmegaX,
        OmegaY,
        OmegaZ,
        Radius,
        Falloff,
        Speed,
    ],
    WindModifier: [
        VelocityMultiplier,
        AngularVelocityMultiplier,
    ],
);
