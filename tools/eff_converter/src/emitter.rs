macro_rules! emitters {
    ($name:ident: [$($parameter:ident),+ $(,)?]) => {
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
    ($($name:ident: [$($parameter:ident),+ $(,)?]),+ $(,)?) => {
        $(
            emitters!($name: [$($parameter,)*]);
        )*
    };
}

emitters!(
    BoxEmitter: [
        // Base Parameters
        SpawnRate,
        SpawnRateSpread,
        Lifetime,
        LifetimeSpread,
        Size,
        SizeSpread,
        Velocity,
        VelocitySpread,
        RotationX,
        RotationXSpread,
        RotationY,
        RotationYSpread,
        RotationZ,
        RotationZSpread,
        AngularVelocityX,
        AngularVelocityXSpread,
        AngularVelocityY,
        AngularVelocityYSpread,
        AngularVelocityZ,
        AngularVelocityZSpread,
        ColorR,
        ColorRSpread,
        ColorG,
        ColorGSpread,
        ColorB,
        ColorBSpread,
        ColorA,
        ColorASpread,
        ColorBrightness,
        ColorBrightnessSpread,
        VelocityX,
        VelocityXSpread,
        VelocityY,
        VelocityYSpread,
        VelocityZ,
        VelocityZSpread,
        UseInstanceRotation,
        // Custom Parameters
        Width,
        WidthSpread,
        Height,
        HeightSpread,
        Length,
        LengthSpread,
        PositiveX,
        PositiveY,
        PositiveZ,
        NegativeX,
        NegativeY,
        NegativeZ,
    ],
    CylinderEmitter: [
        // Base Parameters
        SpawnRate,
        SpawnRateSpread,
        Lifetime,
        LifetimeSpread,
        Size,
        SizeSpread,
        Velocity,
        VelocitySpread,
        RotationX,
        RotationXSpread,
        RotationY,
        RotationYSpread,
        RotationZ,
        RotationZSpread,
        AngularVelocityX,
        AngularVelocityXSpread,
        AngularVelocityY,
        AngularVelocityYSpread,
        AngularVelocityZ,
        AngularVelocityZSpread,
        ColorR,
        ColorRSpread,
        ColorG,
        ColorGSpread,
        ColorB,
        ColorBSpread,
        ColorA,
        ColorASpread,
        ColorBrightness,
        ColorBrightnessSpread,
        VelocityX,
        VelocityXSpread,
        VelocityY,
        VelocityYSpread,
        VelocityZ,
        VelocityZSpread,
        UseInstanceRotation,
        // Custom Parameters
        Radius,
        RadiusSpread,
        Yaw,
        YawSpread,
        Height,
        HeightSpread,
        UseRadiusMax,
        RadiusMax,
    ],
    FlareEmitter: [
        // Base Parameters
        SpawnRate,
        SpawnRateSpread,
        Lifetime,
        LifetimeSpread,
        Size,
        SizeSpread,
        // Custom Parameters
        RotationZ,
        RotationZSpread,
        AngularVelocityZ,
        AngularVelocityZSpread,
        ColorR,
        ColorRSpread,
        ColorG,
        ColorGSpread,
        ColorB,
        ColorBSpread,
        ColorA,
        ColorASpread,
        ColorBrightness,
        ColorBrightnessSpread,
    ],
    SpecialEffectEmitter: [
        // Base Parameters
        SpawnRate,
        SpawnRateSpread,
        Lifetime,
        LifetimeSpread,
    ],
    SphericalEmitter: [
        // Base Parameters
        SpawnRate,
        SpawnRateSpread,
        Lifetime,
        LifetimeSpread,
        Size,
        SizeSpread,
        Velocity,
        VelocitySpread,
        RotationX,
        RotationXSpread,
        RotationY,
        RotationYSpread,
        RotationZ,
        RotationZSpread,
        AngularVelocityX,
        AngularVelocityXSpread,
        AngularVelocityY,
        AngularVelocityYSpread,
        AngularVelocityZ,
        AngularVelocityZSpread,
        ColorR,
        ColorRSpread,
        ColorG,
        ColorGSpread,
        ColorB,
        ColorBSpread,
        ColorA,
        ColorASpread,
        ColorBrightness,
        ColorBrightnessSpread,
        VelocityX,
        VelocityXSpread,
        VelocityY,
        VelocityYSpread,
        VelocityZ,
        VelocityZSpread,
        UseInstanceRotation,
        // Custom Parameters
        Radius,
        RadiusSpread,
        Yaw,
        YawSpread,
        Pitch,
        PitchSpread,
    ],
    SplineEmitter: [
        // Base Parameters
        SpawnRate,
        SpawnRateSpread,
        Lifetime,
        LifetimeSpread,
        Size,
        SizeSpread,
        Velocity,
        VelocitySpread,
        RotationX,
        RotationXSpread,
        RotationY,
        RotationYSpread,
        RotationZ,
        RotationZSpread,
        AngularVelocityX,
        AngularVelocityXSpread,
        AngularVelocityY,
        AngularVelocityYSpread,
        AngularVelocityZ,
        AngularVelocityZSpread,
        ColorR,
        ColorRSpread,
        ColorG,
        ColorGSpread,
        ColorB,
        ColorBSpread,
        ColorA,
        ColorASpread,
        Unused28,
        Unused29,
        VelocityX,
        VelocityXSpread,
        VelocityY,
        VelocityYSpread,
        VelocityZ,
        VelocityZSpread,
        Unused36,
        // Custom Parameters
        SplineStart,
        SplineEnd,
        EndSize,
        EndAngularVelocityX,
        EndAngularVelocityY,
        EndAngularVelocityZ,
        EndColorR,
        EndColorG,
        EndColorB,
        EndColorA,
    ],
);
