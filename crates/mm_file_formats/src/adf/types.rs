use std::io::{BufReader, Cursor};

use binrw::BinRead;

use super::AdfFile;

macro_rules! adf_type_lib {
    ($extension:expr, $path:expr) => {
        AdfTypeLib {
            extension: $extension,
            library: include_bytes!(concat!("data/", $path)),
        }
    };
}

macro_rules! adf_type_libs {
    ($(($extension:expr, $path:expr)),* $(,)?) => {
        [
            $(
                adf_type_lib!($extension, $path),
            )*
        ]
    };
}

// Unused by game:
// RuntimeFormat.adf
// CommonMaterialTypes.adf
// MaterialType.adf
// Material.adf
// Model.adf
// Tm2.adf

pub static BUILT_IN_TYPE_LIBRARY: &'static AdfTypeLib = &adf_type_lib!("", "builtin_types.adf");

pub static TYPE_LIBRARIES: &'static [AdfTypeLib] = &adf_type_libs!(
    ("abfc", "abf_types.adf"),
    ("accomplishment_rulesc", "AccomplishmentRules.adf"),
    ("ai_constants_c", "AiConstantsProfilesTypes.adf"),
    ("aifleec", "AiConstantsProfilesTypes.adf"),
    ("bioinfosc", "bioinfo.adf"),
    ("ccdirectorc", "CarCombatDirector.adf"),
    ("ccenemyc", "CarCombatEnemy.adf"),
    ("ccmapc", "CarCombatMap.adf"),
    ("ccscenarioc", "CarCombatScenario.adf"),
    ("ccsequencec", "CarCombatSequence.adf"),
    ("createdxlsfiles", "xls_types.adf"),
    ("createdxlsfiles", "ItemLibraryData.adf"),
    ("dyn_obcc", "Occluder.adf"),
    ("economyresourcesc", "economyresource_public_types.adf"),
    ("economyresourcesc", "economyresource_types.adf"),
    ("effc_link", "effect_adf.adf"),
    ("effc_link", "game_effect_adf.adf"),
    ("effc", "effect_adf.adf"),
    ("effc", "game_effect_adf.adf"),
    ("effect_xmlc", "effect_adf.adf"),
    ("effect_xmlc", "game_effect_adf.adf"),
    ("erl", "effect_adf.adf"),
    ("erl", "game_effect_adf.adf"),
    ("encounterspawnpointsc", "encounterspawning_types.adf"),
    ("encvehupgdefc", "EncampmentVehicleUpgradeDefinitions.adf"),
    ("featuresc", "featuremenu_filter.adf"),
    ("gatingc", "gating_types.adf"),
    ("gsrc", "graphadf.adf"),
    ("guimsh", "gui_mesh_adf.adf"),
    ("guiroadmeshc", "gui_road_mesh.adf"),
    ("guistatmappingc", "gui_stats_mapping.adf"),
    ("guistreamertexturelistc", "guistreamertexturelist.adf"),
    ("guixc", "gui_adf.adf"),
    ("intentstablec", "ConditionalDialogData.adf"),
    ("light_infoc", "all_light_objects.adf"),
    ("locationinfoc", "locationinfo_public_types.adf"),
    ("locationinfoc", "locationinfo_types.adf"),
    ("mapiconsc", "mapicon_types.adf"),
    ("missionsc", "mission_types.adf"),
    ("racetrophiesc", "xls_types.adf"),
    ("regioninfoc", "regioninfo_public_types.adf"),
    ("regioninfoc", "regioninfo_types.adf"),
    ("relicsetc", "relicset.adf"),
    ("resourcesetsc", "resourcesets.adf"),
    ("resourcesetsc", "SpawnResources.adf"),
    ("restartpointsc", "restartpoint_types.adf"),
    ("roadgraphc", "RoadGraphData.adf"),
    ("shader_bundle", "shader_library_format.adf"),
    ("sideramc", "SideramDefinition.adf"),
    ("spawndebugc", "resourcesets.adf"),
    ("spawndebugc", "SpawnResources.adf"),
    ("spawnresourcesc", "SpawnResources.adf"),
    ("stringlookup", "StringLookup.adf"),
    ("trackedobjectdatac", "tracked_object_types.adf"),
    ("trim", "effect_adf.adf"),
    ("trim", "game_effect_adf.adf"),
    ("upgradedefinitionsc", "VehicleUpgradeDefinitions.adf"),
    ("vehupgrexc", "xls_types.adf"),
    ("venginec", "VehicleEngineSound.adf"),
    ("vpgeneralc", "VehiclePhysicsGeneral.adf"),
    ("vpsolverc", "VehiclePhysicsSolver.adf"),
    ("xlsc", "xls_types.adf"),
    ("xvmc", "xvm_adf.adf"),
);

pub struct AdfTypeLib {
    pub extension: &'static str,
    pub library: &'static [u8],
}

impl AdfTypeLib {
    pub fn load(&self) -> binrw::BinResult<AdfFile> {
        let mut reader = BufReader::new(Cursor::new(self.library));
        Ok(AdfFile::read_le(&mut reader)?)
    }
}
