use color_eyre::eyre::{self, eyre};

pub fn component_name_to_id(name: &str) -> eyre::Result<i32> {
    match name {
        "ControllablePhysicsComponent" => Ok(1),
        "RenderComponent" => Ok(2),
        "SimplePhysicsComponent" => Ok(3),
        "CharacterComponent" => Ok(4),
        "ScriptComponent" => Ok(5),
        "BouncerComponent" => Ok(6),
        "DestructibleComponent" => Ok(7),
        "GhostComponent" => Ok(8),
        "SkillComponent" => Ok(9),
        "SpawnerComponent" => Ok(10),
        "ItemComponent" => Ok(11),
        "RebuildComponent" => Ok(12),
        "RebuildStartComponent" => Ok(13),
        "RebuildActivatorComponent" => Ok(14),
        "IconOnlyComponent" => Ok(15),
        "VendorComponent" => Ok(16),
        "InventoryComponent" => Ok(17),
        "ProjectilePhysicsComponent" => Ok(18),
        "ShootingGalleryComponent" => Ok(19),
        "RigidBodyPhantomPhysicsComponent" => Ok(20),
        "DropEffectComponent" => Ok(21),
        "ChestComponent" => Ok(22),
        "CollectibleComponent" => Ok(23),
        "BlueprintComponent" => Ok(24),
        "MovingPlatformComponent" => Ok(25),
        "PetComponent" => Ok(26),
        "PlatformBoundaryComponent" => Ok(27),
        "ModuleComponent" => Ok(28),
        "ArcadeComponent" => Ok(29),
        "VehiclePhysicsComponent" => Ok(30),
        "MovementAIComponent" => Ok(31),
        "ExhibitComponent" => Ok(32),
        "OverheadIconComponent" => Ok(33),
        "PetControlComponent" => Ok(34),
        "MinifigComponent" => Ok(35),
        "PropertyComponent" => Ok(36),
        "PetCreatorComponent" => Ok(37),
        "ModelBuilderComponent" => Ok(38),
        "ScriptedActivityComponent" => Ok(39),
        "PhantomPhysicsComponent" => Ok(40),
        "SpringpadComponent" => Ok(41),
        "B3BehaviorsComponent" => Ok(42),
        "PropertyEntranceComponent" => Ok(43),
        "FXComponent" => Ok(44),
        "PropertyManagementComponent" => Ok(45),
        "SecondVehiclePhysicsComponent" => Ok(46),
        "PhysicsSystemComponent" => Ok(47),
        "QuickBuildComponent" => Ok(48),
        "SwitchComponent" => Ok(49),
        "MinigameComponent" => Ok(50),
        "ChanglingComponent" => Ok(51),
        "ChoiceBuildComponent" => Ok(52),
        "PackageComponent" => Ok(53),
        "SoundRepeaterComponent" => Ok(54),
        "SoundAmbient2DComponent" => Ok(55),
        "SoundAmbient3DComponent" => Ok(56),
        "PreconditionComponent" => Ok(57),
        "PlayerFlagsComponent" => Ok(58),
        "CustomBuildAssemblyComponent" => Ok(59),
        "BaseCombatAIComponent" => Ok(60),
        "ModuleAssemblyComponent" => Ok(61),
        "ShowcaseModelHandlerComponent" => Ok(62),
        "RacingModuleComponent" => Ok(63),
        "GenericActivatorComponent" => Ok(64),
        "PropertyVendorComponent" => Ok(65),
        "HFLightDirectionGadgetComponent" => Ok(66),
        "RocketLaunchComponent" => Ok(67),
        "RocketLandingComponent" => Ok(68),
        "TriggerComponent" => Ok(69),
        "DroppedLootComponent" => Ok(70),
        "RacingControlComponent" => Ok(71),
        "FactionTriggerComponent" => Ok(72),
        "MissionNPCComponent" => Ok(73),
        "RacingStatsComponent" => Ok(74),
        "LUPExhibitComponent" => Ok(75),
        "BBBComponent" => Ok(76),
        "SoundTriggerComponent" => Ok(77),
        "ProximityMonitorComponent" => Ok(78),
        "RacingSoundTriggerComponent" => Ok(79),
        "ChatComponent" => Ok(80),
        "FriendsListComponent" => Ok(81),
        "GuildComponent" => Ok(82),
        "LocalSystemComponent" => Ok(83),
        "MissionComponent" => Ok(84),
        "MutableModelBehaviorsComponent" => Ok(85),
        "PathfindingControlComponent" => Ok(86),
        "PetTamingControlComponent" => Ok(87),
        "PropertyEditorComponent" => Ok(88),
        "SkinnedRenderComponent" => Ok(89),
        "SlashCommandComponent" => Ok(90),
        "StatusEffectComponent" => Ok(91),
        "TeamsComponent" => Ok(92),
        "TextEffectComponent" => Ok(93),
        "TradeComponent" => Ok(94),
        "UserControlComponent" => Ok(95),
        "IgnoreListComponent" => Ok(96),
        "LUPLaunchpadComponent" => Ok(97),
        "InteractionManagerComponent" => Ok(98),
        "DonationVendorComponent" => Ok(100),
        "CombatMediatorComponent" => Ok(101),
        "Component107" => Ok(107),
        "Possesable" => Ok(108),
        _ => Err(eyre!("Unknown component name {}", name)),
    }
}

pub fn mod_type_to_table_name(name: &str) -> String {
    match name {
        "npc" | "item" | "object" => "Objects".to_string(),
        "mission" => "Missions".to_string(),
        _ => {
            if name.ends_with("PhysicsComponent") {
                return String::from("PhysicsComponent");
            }
            String::from(name)
        }
    }
}
