/* This file is generated. Do not edit manually. */
use serde::{Deserialize, Serialize};
use std::fmt;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameRule {
    ShowAdvancementMessages,
    BlockExplosionDropDecay,
    CommandBlockOutput,
    MaxBlockModifications,
    DisableElytraMovementCheck,
    DisableRaids,
    AdvanceTime,
    EntityDrops,
    DoFireTick,
    ImmediateRespawn,
    SpawnPhantoms,
    LimitedCrafting,
    MobDrops,
    SpawnMobs,
    SpawnPatrols,
    BlockDrops,
    SpawnWanderingTraders,
    SpreadVines,
    SpawnWardens,
    AdvanceWeather,
    DrowningDamage,
    EnderPearlsVanishOnDeath,
    FallDamage,
    FireDamage,
    ForgiveDeadPlayers,
    FreezeDamage,
    GlobalSoundEvents,
    KeepInventory,
    LavaSourceConversion,
    LogAdminCommands,
    MaxCommandSequenceLength,
    MaxCommandForks,
    MaxEntityCramming,
    MobExplosionDropDecay,
    MobGriefing,
    NaturalHealthRegeneration,
    PlayersNetherPortalCreativeDelay,
    PlayersNetherPortalDefaultDelay,
    PlayersSleepingPercentage,
    ProjectilesCanBreakBlocks,
    RandomTickSpeed,
    ReducedDebugInfo,
    SendCommandFeedback,
    ShowDeathMessages,
    MaxSnowAccumulationHeight,
    SpawnChunkRadius,
    RespawnRadius,
    SpectatorsGenerateChunks,
    TntExplosionDropDecay,
    UniversalAnger,
    WaterSourceConversion,
}
impl GameRule {
    pub const fn all() -> &'static [Self] {
        &[
            Self::ShowAdvancementMessages,
            Self::BlockExplosionDropDecay,
            Self::CommandBlockOutput,
            Self::MaxBlockModifications,
            Self::DisableElytraMovementCheck,
            Self::DisableRaids,
            Self::AdvanceTime,
            Self::EntityDrops,
            Self::DoFireTick,
            Self::ImmediateRespawn,
            Self::SpawnPhantoms,
            Self::LimitedCrafting,
            Self::MobDrops,
            Self::SpawnMobs,
            Self::SpawnPatrols,
            Self::BlockDrops,
            Self::SpawnWanderingTraders,
            Self::SpreadVines,
            Self::SpawnWardens,
            Self::AdvanceWeather,
            Self::DrowningDamage,
            Self::EnderPearlsVanishOnDeath,
            Self::FallDamage,
            Self::FireDamage,
            Self::ForgiveDeadPlayers,
            Self::FreezeDamage,
            Self::GlobalSoundEvents,
            Self::KeepInventory,
            Self::LavaSourceConversion,
            Self::LogAdminCommands,
            Self::MaxCommandSequenceLength,
            Self::MaxCommandForks,
            Self::MaxEntityCramming,
            Self::MobExplosionDropDecay,
            Self::MobGriefing,
            Self::NaturalHealthRegeneration,
            Self::PlayersNetherPortalCreativeDelay,
            Self::PlayersNetherPortalDefaultDelay,
            Self::PlayersSleepingPercentage,
            Self::ProjectilesCanBreakBlocks,
            Self::RandomTickSpeed,
            Self::ReducedDebugInfo,
            Self::SendCommandFeedback,
            Self::ShowDeathMessages,
            Self::MaxSnowAccumulationHeight,
            Self::SpawnChunkRadius,
            Self::RespawnRadius,
            Self::SpectatorsGenerateChunks,
            Self::TntExplosionDropDecay,
            Self::UniversalAnger,
            Self::WaterSourceConversion,
        ]
    }
}
impl fmt::Display for GameRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ShowAdvancementMessages => write!(f, "announceAdvancements"),
            Self::BlockExplosionDropDecay => write!(f, "blockExplosionDropDecay"),
            Self::CommandBlockOutput => write!(f, "commandBlockOutput"),
            Self::MaxBlockModifications => write!(f, "commandModificationBlockLimit"),
            Self::DisableElytraMovementCheck => write!(f, "disableElytraMovementCheck"),
            Self::DisableRaids => write!(f, "disableRaids"),
            Self::AdvanceTime => write!(f, "doDaylightCycle"),
            Self::EntityDrops => write!(f, "doEntityDrops"),
            Self::DoFireTick => write!(f, "doFireTick"),
            Self::ImmediateRespawn => write!(f, "doImmediateRespawn"),
            Self::SpawnPhantoms => write!(f, "doInsomnia"),
            Self::LimitedCrafting => write!(f, "doLimitedCrafting"),
            Self::MobDrops => write!(f, "doMobLoot"),
            Self::SpawnMobs => write!(f, "doMobSpawning"),
            Self::SpawnPatrols => write!(f, "doPatrolSpawning"),
            Self::BlockDrops => write!(f, "doTileDrops"),
            Self::SpawnWanderingTraders => write!(f, "doTraderSpawning"),
            Self::SpreadVines => write!(f, "doVinesSpread"),
            Self::SpawnWardens => write!(f, "doWardenSpawning"),
            Self::AdvanceWeather => write!(f, "doWeatherCycle"),
            Self::DrowningDamage => write!(f, "drowningDamage"),
            Self::EnderPearlsVanishOnDeath => write!(f, "enderPearlsVanishOnDeath"),
            Self::FallDamage => write!(f, "fallDamage"),
            Self::FireDamage => write!(f, "fireDamage"),
            Self::ForgiveDeadPlayers => write!(f, "forgiveDeadPlayers"),
            Self::FreezeDamage => write!(f, "freezeDamage"),
            Self::GlobalSoundEvents => write!(f, "globalSoundEvents"),
            Self::KeepInventory => write!(f, "keepInventory"),
            Self::LavaSourceConversion => write!(f, "lavaSourceConversion"),
            Self::LogAdminCommands => write!(f, "logAdminCommands"),
            Self::MaxCommandSequenceLength => write!(f, "maxCommandChainLength"),
            Self::MaxCommandForks => write!(f, "maxCommandForkCount"),
            Self::MaxEntityCramming => write!(f, "maxEntityCramming"),
            Self::MobExplosionDropDecay => write!(f, "mobExplosionDropDecay"),
            Self::MobGriefing => write!(f, "mobGriefing"),
            Self::NaturalHealthRegeneration => write!(f, "naturalRegeneration"),
            Self::PlayersNetherPortalCreativeDelay => write!(f, "playersNetherPortalCreativeDelay"),
            Self::PlayersNetherPortalDefaultDelay => write!(f, "playersNetherPortalDefaultDelay"),
            Self::PlayersSleepingPercentage => write!(f, "playersSleepingPercentage"),
            Self::ProjectilesCanBreakBlocks => write!(f, "projectilesCanBreakBlocks"),
            Self::RandomTickSpeed => write!(f, "randomTickSpeed"),
            Self::ReducedDebugInfo => write!(f, "reducedDebugInfo"),
            Self::SendCommandFeedback => write!(f, "sendCommandFeedback"),
            Self::ShowDeathMessages => write!(f, "showDeathMessages"),
            Self::MaxSnowAccumulationHeight => write!(f, "snowAccumulationHeight"),
            Self::SpawnChunkRadius => write!(f, "spawnChunkRadius"),
            Self::RespawnRadius => write!(f, "spawnRadius"),
            Self::SpectatorsGenerateChunks => write!(f, "spectatorsGenerateChunks"),
            Self::TntExplosionDropDecay => write!(f, "tntExplosionDropDecay"),
            Self::UniversalAnger => write!(f, "universalAnger"),
            Self::WaterSourceConversion => write!(f, "waterSourceConversion"),
        }
    }
}
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct GameRuleRegistry {
    #[serde(rename = "announceAdvancements")]
    #[serde(default = "default_show_advancement_messages")]
    #[serde(with = "as_string")]
    pub show_advancement_messages: bool,
    #[serde(rename = "blockExplosionDropDecay")]
    #[serde(default = "default_block_explosion_drop_decay")]
    #[serde(with = "as_string")]
    pub block_explosion_drop_decay: bool,
    #[serde(rename = "commandBlockOutput")]
    #[serde(default = "default_command_block_output")]
    #[serde(with = "as_string")]
    pub command_block_output: bool,
    #[serde(rename = "commandModificationBlockLimit")]
    #[serde(default = "default_max_block_modifications")]
    #[serde(with = "as_string")]
    pub max_block_modifications: i64,
    #[serde(rename = "disableElytraMovementCheck")]
    #[serde(default = "default_disable_elytra_movement_check")]
    #[serde(with = "as_string")]
    pub disable_elytra_movement_check: bool,
    #[serde(rename = "disableRaids")]
    #[serde(default = "default_disable_raids")]
    #[serde(with = "as_string")]
    pub disable_raids: bool,
    #[serde(rename = "doDaylightCycle")]
    #[serde(default = "default_advance_time")]
    #[serde(with = "as_string")]
    pub advance_time: bool,
    #[serde(rename = "doEntityDrops")]
    #[serde(default = "default_entity_drops")]
    #[serde(with = "as_string")]
    pub entity_drops: bool,
    #[serde(rename = "doFireTick")]
    #[serde(default = "default_do_fire_tick")]
    #[serde(with = "as_string")]
    pub do_fire_tick: bool,
    #[serde(rename = "doImmediateRespawn")]
    #[serde(default = "default_immediate_respawn")]
    #[serde(with = "as_string")]
    pub immediate_respawn: bool,
    #[serde(rename = "doInsomnia")]
    #[serde(default = "default_spawn_phantoms")]
    #[serde(with = "as_string")]
    pub spawn_phantoms: bool,
    #[serde(rename = "doLimitedCrafting")]
    #[serde(default = "default_limited_crafting")]
    #[serde(with = "as_string")]
    pub limited_crafting: bool,
    #[serde(rename = "doMobLoot")]
    #[serde(default = "default_mob_drops")]
    #[serde(with = "as_string")]
    pub mob_drops: bool,
    #[serde(rename = "doMobSpawning")]
    #[serde(default = "default_spawn_mobs")]
    #[serde(with = "as_string")]
    pub spawn_mobs: bool,
    #[serde(rename = "doPatrolSpawning")]
    #[serde(default = "default_spawn_patrols")]
    #[serde(with = "as_string")]
    pub spawn_patrols: bool,
    #[serde(rename = "doTileDrops")]
    #[serde(default = "default_block_drops")]
    #[serde(with = "as_string")]
    pub block_drops: bool,
    #[serde(rename = "doTraderSpawning")]
    #[serde(default = "default_spawn_wandering_traders")]
    #[serde(with = "as_string")]
    pub spawn_wandering_traders: bool,
    #[serde(rename = "doVinesSpread")]
    #[serde(default = "default_spread_vines")]
    #[serde(with = "as_string")]
    pub spread_vines: bool,
    #[serde(rename = "doWardenSpawning")]
    #[serde(default = "default_spawn_wardens")]
    #[serde(with = "as_string")]
    pub spawn_wardens: bool,
    #[serde(rename = "doWeatherCycle")]
    #[serde(default = "default_advance_weather")]
    #[serde(with = "as_string")]
    pub advance_weather: bool,
    #[serde(rename = "drowningDamage")]
    #[serde(default = "default_drowning_damage")]
    #[serde(with = "as_string")]
    pub drowning_damage: bool,
    #[serde(rename = "enderPearlsVanishOnDeath")]
    #[serde(default = "default_ender_pearls_vanish_on_death")]
    #[serde(with = "as_string")]
    pub ender_pearls_vanish_on_death: bool,
    #[serde(rename = "fallDamage")]
    #[serde(default = "default_fall_damage")]
    #[serde(with = "as_string")]
    pub fall_damage: bool,
    #[serde(rename = "fireDamage")]
    #[serde(default = "default_fire_damage")]
    #[serde(with = "as_string")]
    pub fire_damage: bool,
    #[serde(rename = "forgiveDeadPlayers")]
    #[serde(default = "default_forgive_dead_players")]
    #[serde(with = "as_string")]
    pub forgive_dead_players: bool,
    #[serde(rename = "freezeDamage")]
    #[serde(default = "default_freeze_damage")]
    #[serde(with = "as_string")]
    pub freeze_damage: bool,
    #[serde(rename = "globalSoundEvents")]
    #[serde(default = "default_global_sound_events")]
    #[serde(with = "as_string")]
    pub global_sound_events: bool,
    #[serde(rename = "keepInventory")]
    #[serde(default = "default_keep_inventory")]
    #[serde(with = "as_string")]
    pub keep_inventory: bool,
    #[serde(rename = "lavaSourceConversion")]
    #[serde(default = "default_lava_source_conversion")]
    #[serde(with = "as_string")]
    pub lava_source_conversion: bool,
    #[serde(rename = "logAdminCommands")]
    #[serde(default = "default_log_admin_commands")]
    #[serde(with = "as_string")]
    pub log_admin_commands: bool,
    #[serde(rename = "maxCommandChainLength")]
    #[serde(default = "default_max_command_sequence_length")]
    #[serde(with = "as_string")]
    pub max_command_sequence_length: i64,
    #[serde(rename = "maxCommandForkCount")]
    #[serde(default = "default_max_command_forks")]
    #[serde(with = "as_string")]
    pub max_command_forks: i64,
    #[serde(rename = "maxEntityCramming")]
    #[serde(default = "default_max_entity_cramming")]
    #[serde(with = "as_string")]
    pub max_entity_cramming: i64,
    #[serde(rename = "mobExplosionDropDecay")]
    #[serde(default = "default_mob_explosion_drop_decay")]
    #[serde(with = "as_string")]
    pub mob_explosion_drop_decay: bool,
    #[serde(rename = "mobGriefing")]
    #[serde(default = "default_mob_griefing")]
    #[serde(with = "as_string")]
    pub mob_griefing: bool,
    #[serde(rename = "naturalRegeneration")]
    #[serde(default = "default_natural_health_regeneration")]
    #[serde(with = "as_string")]
    pub natural_health_regeneration: bool,
    #[serde(rename = "playersNetherPortalCreativeDelay")]
    #[serde(default = "default_players_nether_portal_creative_delay")]
    #[serde(with = "as_string")]
    pub players_nether_portal_creative_delay: i64,
    #[serde(rename = "playersNetherPortalDefaultDelay")]
    #[serde(default = "default_players_nether_portal_default_delay")]
    #[serde(with = "as_string")]
    pub players_nether_portal_default_delay: i64,
    #[serde(rename = "playersSleepingPercentage")]
    #[serde(default = "default_players_sleeping_percentage")]
    #[serde(with = "as_string")]
    pub players_sleeping_percentage: i64,
    #[serde(rename = "projectilesCanBreakBlocks")]
    #[serde(default = "default_projectiles_can_break_blocks")]
    #[serde(with = "as_string")]
    pub projectiles_can_break_blocks: bool,
    #[serde(rename = "randomTickSpeed")]
    #[serde(default = "default_random_tick_speed")]
    #[serde(with = "as_string")]
    pub random_tick_speed: i64,
    #[serde(rename = "reducedDebugInfo")]
    #[serde(default = "default_reduced_debug_info")]
    #[serde(with = "as_string")]
    pub reduced_debug_info: bool,
    #[serde(rename = "sendCommandFeedback")]
    #[serde(default = "default_send_command_feedback")]
    #[serde(with = "as_string")]
    pub send_command_feedback: bool,
    #[serde(rename = "showDeathMessages")]
    #[serde(default = "default_show_death_messages")]
    #[serde(with = "as_string")]
    pub show_death_messages: bool,
    #[serde(rename = "snowAccumulationHeight")]
    #[serde(default = "default_max_snow_accumulation_height")]
    #[serde(with = "as_string")]
    pub max_snow_accumulation_height: i64,
    #[serde(rename = "spawnChunkRadius")]
    #[serde(default = "default_spawn_chunk_radius")]
    #[serde(with = "as_string")]
    pub spawn_chunk_radius: i64,
    #[serde(rename = "spawnRadius")]
    #[serde(default = "default_respawn_radius")]
    #[serde(with = "as_string")]
    pub respawn_radius: i64,
    #[serde(rename = "spectatorsGenerateChunks")]
    #[serde(default = "default_spectators_generate_chunks")]
    #[serde(with = "as_string")]
    pub spectators_generate_chunks: bool,
    #[serde(rename = "tntExplosionDropDecay")]
    #[serde(default = "default_tnt_explosion_drop_decay")]
    #[serde(with = "as_string")]
    pub tnt_explosion_drop_decay: bool,
    #[serde(rename = "universalAnger")]
    #[serde(default = "default_universal_anger")]
    #[serde(with = "as_string")]
    pub universal_anger: bool,
    #[serde(rename = "waterSourceConversion")]
    #[serde(default = "default_water_source_conversion")]
    #[serde(with = "as_string")]
    pub water_source_conversion: bool,
    #[serde(skip, default = "default_allow_entering_nether_using_portals")]
    pub allow_entering_nether_using_portals: bool,
    #[serde(skip, default = "default_command_blocks_work")]
    pub command_blocks_work: bool,
    #[serde(skip, default = "default_fire_spread_radius_around_player")]
    pub fire_spread_radius_around_player: i64,
    #[serde(skip, default = "default_locator_bar")]
    pub locator_bar: bool,
    #[serde(skip, default = "default_player_movement_check")]
    pub player_movement_check: bool,
    #[serde(skip, default = "default_pvp")]
    pub pvp: bool,
    #[serde(skip, default = "default_spawn_monsters")]
    pub spawn_monsters: bool,
    #[serde(skip, default = "default_spawner_blocks_work")]
    pub spawner_blocks_work: bool,
    #[serde(skip, default = "default_tnt_explodes")]
    pub tnt_explodes: bool,
}
pub enum GameRuleValue<I, B> {
    Int(I),
    Bool(B),
}
impl<I: fmt::Display, B: fmt::Display> fmt::Display for GameRuleValue<I, B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{v}"),
            Self::Bool(v) => write!(f, "{v}"),
        }
    }
}
impl GameRuleRegistry {
    pub fn get(&self, rule: &GameRule) -> GameRuleValue<&i64, &bool> {
        match rule {
            GameRule::ShowAdvancementMessages => {
                GameRuleValue::Bool(&self.show_advancement_messages)
            }
            GameRule::BlockExplosionDropDecay => {
                GameRuleValue::Bool(&self.block_explosion_drop_decay)
            }
            GameRule::CommandBlockOutput => GameRuleValue::Bool(&self.command_block_output),
            GameRule::MaxBlockModifications => GameRuleValue::Int(&self.max_block_modifications),
            GameRule::DisableElytraMovementCheck => {
                GameRuleValue::Bool(&self.disable_elytra_movement_check)
            }
            GameRule::DisableRaids => GameRuleValue::Bool(&self.disable_raids),
            GameRule::AdvanceTime => GameRuleValue::Bool(&self.advance_time),
            GameRule::EntityDrops => GameRuleValue::Bool(&self.entity_drops),
            GameRule::DoFireTick => GameRuleValue::Bool(&self.do_fire_tick),
            GameRule::ImmediateRespawn => GameRuleValue::Bool(&self.immediate_respawn),
            GameRule::SpawnPhantoms => GameRuleValue::Bool(&self.spawn_phantoms),
            GameRule::LimitedCrafting => GameRuleValue::Bool(&self.limited_crafting),
            GameRule::MobDrops => GameRuleValue::Bool(&self.mob_drops),
            GameRule::SpawnMobs => GameRuleValue::Bool(&self.spawn_mobs),
            GameRule::SpawnPatrols => GameRuleValue::Bool(&self.spawn_patrols),
            GameRule::BlockDrops => GameRuleValue::Bool(&self.block_drops),
            GameRule::SpawnWanderingTraders => GameRuleValue::Bool(&self.spawn_wandering_traders),
            GameRule::SpreadVines => GameRuleValue::Bool(&self.spread_vines),
            GameRule::SpawnWardens => GameRuleValue::Bool(&self.spawn_wardens),
            GameRule::AdvanceWeather => GameRuleValue::Bool(&self.advance_weather),
            GameRule::DrowningDamage => GameRuleValue::Bool(&self.drowning_damage),
            GameRule::EnderPearlsVanishOnDeath => {
                GameRuleValue::Bool(&self.ender_pearls_vanish_on_death)
            }
            GameRule::FallDamage => GameRuleValue::Bool(&self.fall_damage),
            GameRule::FireDamage => GameRuleValue::Bool(&self.fire_damage),
            GameRule::ForgiveDeadPlayers => GameRuleValue::Bool(&self.forgive_dead_players),
            GameRule::FreezeDamage => GameRuleValue::Bool(&self.freeze_damage),
            GameRule::GlobalSoundEvents => GameRuleValue::Bool(&self.global_sound_events),
            GameRule::KeepInventory => GameRuleValue::Bool(&self.keep_inventory),
            GameRule::LavaSourceConversion => GameRuleValue::Bool(&self.lava_source_conversion),
            GameRule::LogAdminCommands => GameRuleValue::Bool(&self.log_admin_commands),
            GameRule::MaxCommandSequenceLength => {
                GameRuleValue::Int(&self.max_command_sequence_length)
            }
            GameRule::MaxCommandForks => GameRuleValue::Int(&self.max_command_forks),
            GameRule::MaxEntityCramming => GameRuleValue::Int(&self.max_entity_cramming),
            GameRule::MobExplosionDropDecay => GameRuleValue::Bool(&self.mob_explosion_drop_decay),
            GameRule::MobGriefing => GameRuleValue::Bool(&self.mob_griefing),
            GameRule::NaturalHealthRegeneration => {
                GameRuleValue::Bool(&self.natural_health_regeneration)
            }
            GameRule::PlayersNetherPortalCreativeDelay => {
                GameRuleValue::Int(&self.players_nether_portal_creative_delay)
            }
            GameRule::PlayersNetherPortalDefaultDelay => {
                GameRuleValue::Int(&self.players_nether_portal_default_delay)
            }
            GameRule::PlayersSleepingPercentage => {
                GameRuleValue::Int(&self.players_sleeping_percentage)
            }
            GameRule::ProjectilesCanBreakBlocks => {
                GameRuleValue::Bool(&self.projectiles_can_break_blocks)
            }
            GameRule::RandomTickSpeed => GameRuleValue::Int(&self.random_tick_speed),
            GameRule::ReducedDebugInfo => GameRuleValue::Bool(&self.reduced_debug_info),
            GameRule::SendCommandFeedback => GameRuleValue::Bool(&self.send_command_feedback),
            GameRule::ShowDeathMessages => GameRuleValue::Bool(&self.show_death_messages),
            GameRule::MaxSnowAccumulationHeight => {
                GameRuleValue::Int(&self.max_snow_accumulation_height)
            }
            GameRule::SpawnChunkRadius => GameRuleValue::Int(&self.spawn_chunk_radius),
            GameRule::RespawnRadius => GameRuleValue::Int(&self.respawn_radius),
            GameRule::SpectatorsGenerateChunks => {
                GameRuleValue::Bool(&self.spectators_generate_chunks)
            }
            GameRule::TntExplosionDropDecay => GameRuleValue::Bool(&self.tnt_explosion_drop_decay),
            GameRule::UniversalAnger => GameRuleValue::Bool(&self.universal_anger),
            GameRule::WaterSourceConversion => GameRuleValue::Bool(&self.water_source_conversion),
        }
    }
    pub fn get_mut(&mut self, rule: &GameRule) -> GameRuleValue<&mut i64, &mut bool> {
        match rule {
            GameRule::ShowAdvancementMessages => {
                GameRuleValue::Bool(&mut self.show_advancement_messages)
            }
            GameRule::BlockExplosionDropDecay => {
                GameRuleValue::Bool(&mut self.block_explosion_drop_decay)
            }
            GameRule::CommandBlockOutput => GameRuleValue::Bool(&mut self.command_block_output),
            GameRule::MaxBlockModifications => {
                GameRuleValue::Int(&mut self.max_block_modifications)
            }
            GameRule::DisableElytraMovementCheck => {
                GameRuleValue::Bool(&mut self.disable_elytra_movement_check)
            }
            GameRule::DisableRaids => GameRuleValue::Bool(&mut self.disable_raids),
            GameRule::AdvanceTime => GameRuleValue::Bool(&mut self.advance_time),
            GameRule::EntityDrops => GameRuleValue::Bool(&mut self.entity_drops),
            GameRule::DoFireTick => GameRuleValue::Bool(&mut self.do_fire_tick),
            GameRule::ImmediateRespawn => GameRuleValue::Bool(&mut self.immediate_respawn),
            GameRule::SpawnPhantoms => GameRuleValue::Bool(&mut self.spawn_phantoms),
            GameRule::LimitedCrafting => GameRuleValue::Bool(&mut self.limited_crafting),
            GameRule::MobDrops => GameRuleValue::Bool(&mut self.mob_drops),
            GameRule::SpawnMobs => GameRuleValue::Bool(&mut self.spawn_mobs),
            GameRule::SpawnPatrols => GameRuleValue::Bool(&mut self.spawn_patrols),
            GameRule::BlockDrops => GameRuleValue::Bool(&mut self.block_drops),
            GameRule::SpawnWanderingTraders => {
                GameRuleValue::Bool(&mut self.spawn_wandering_traders)
            }
            GameRule::SpreadVines => GameRuleValue::Bool(&mut self.spread_vines),
            GameRule::SpawnWardens => GameRuleValue::Bool(&mut self.spawn_wardens),
            GameRule::AdvanceWeather => GameRuleValue::Bool(&mut self.advance_weather),
            GameRule::DrowningDamage => GameRuleValue::Bool(&mut self.drowning_damage),
            GameRule::EnderPearlsVanishOnDeath => {
                GameRuleValue::Bool(&mut self.ender_pearls_vanish_on_death)
            }
            GameRule::FallDamage => GameRuleValue::Bool(&mut self.fall_damage),
            GameRule::FireDamage => GameRuleValue::Bool(&mut self.fire_damage),
            GameRule::ForgiveDeadPlayers => GameRuleValue::Bool(&mut self.forgive_dead_players),
            GameRule::FreezeDamage => GameRuleValue::Bool(&mut self.freeze_damage),
            GameRule::GlobalSoundEvents => GameRuleValue::Bool(&mut self.global_sound_events),
            GameRule::KeepInventory => GameRuleValue::Bool(&mut self.keep_inventory),
            GameRule::LavaSourceConversion => GameRuleValue::Bool(&mut self.lava_source_conversion),
            GameRule::LogAdminCommands => GameRuleValue::Bool(&mut self.log_admin_commands),
            GameRule::MaxCommandSequenceLength => {
                GameRuleValue::Int(&mut self.max_command_sequence_length)
            }
            GameRule::MaxCommandForks => GameRuleValue::Int(&mut self.max_command_forks),
            GameRule::MaxEntityCramming => GameRuleValue::Int(&mut self.max_entity_cramming),
            GameRule::MobExplosionDropDecay => {
                GameRuleValue::Bool(&mut self.mob_explosion_drop_decay)
            }
            GameRule::MobGriefing => GameRuleValue::Bool(&mut self.mob_griefing),
            GameRule::NaturalHealthRegeneration => {
                GameRuleValue::Bool(&mut self.natural_health_regeneration)
            }
            GameRule::PlayersNetherPortalCreativeDelay => {
                GameRuleValue::Int(&mut self.players_nether_portal_creative_delay)
            }
            GameRule::PlayersNetherPortalDefaultDelay => {
                GameRuleValue::Int(&mut self.players_nether_portal_default_delay)
            }
            GameRule::PlayersSleepingPercentage => {
                GameRuleValue::Int(&mut self.players_sleeping_percentage)
            }
            GameRule::ProjectilesCanBreakBlocks => {
                GameRuleValue::Bool(&mut self.projectiles_can_break_blocks)
            }
            GameRule::RandomTickSpeed => GameRuleValue::Int(&mut self.random_tick_speed),
            GameRule::ReducedDebugInfo => GameRuleValue::Bool(&mut self.reduced_debug_info),
            GameRule::SendCommandFeedback => GameRuleValue::Bool(&mut self.send_command_feedback),
            GameRule::ShowDeathMessages => GameRuleValue::Bool(&mut self.show_death_messages),
            GameRule::MaxSnowAccumulationHeight => {
                GameRuleValue::Int(&mut self.max_snow_accumulation_height)
            }
            GameRule::SpawnChunkRadius => GameRuleValue::Int(&mut self.spawn_chunk_radius),
            GameRule::RespawnRadius => GameRuleValue::Int(&mut self.respawn_radius),
            GameRule::SpectatorsGenerateChunks => {
                GameRuleValue::Bool(&mut self.spectators_generate_chunks)
            }
            GameRule::TntExplosionDropDecay => {
                GameRuleValue::Bool(&mut self.tnt_explosion_drop_decay)
            }
            GameRule::UniversalAnger => GameRuleValue::Bool(&mut self.universal_anger),
            GameRule::WaterSourceConversion => {
                GameRuleValue::Bool(&mut self.water_source_conversion)
            }
        }
    }
}
impl Default for GameRuleRegistry {
    fn default() -> Self {
        Self {
            show_advancement_messages: true,
            block_explosion_drop_decay: true,
            command_block_output: true,
            max_block_modifications: 32768i64,
            disable_elytra_movement_check: false,
            disable_raids: false,
            advance_time: true,
            entity_drops: true,
            do_fire_tick: true,
            immediate_respawn: false,
            spawn_phantoms: true,
            limited_crafting: false,
            mob_drops: true,
            spawn_mobs: true,
            spawn_patrols: true,
            block_drops: true,
            spawn_wandering_traders: true,
            spread_vines: true,
            spawn_wardens: true,
            advance_weather: true,
            drowning_damage: true,
            ender_pearls_vanish_on_death: true,
            fall_damage: true,
            fire_damage: true,
            forgive_dead_players: true,
            freeze_damage: true,
            global_sound_events: true,
            keep_inventory: false,
            lava_source_conversion: false,
            log_admin_commands: true,
            max_command_sequence_length: 65536i64,
            max_command_forks: 65536i64,
            max_entity_cramming: 24i64,
            mob_explosion_drop_decay: true,
            mob_griefing: true,
            natural_health_regeneration: true,
            players_nether_portal_creative_delay: 1i64,
            players_nether_portal_default_delay: 80i64,
            players_sleeping_percentage: 100i64,
            projectiles_can_break_blocks: true,
            random_tick_speed: 3i64,
            reduced_debug_info: false,
            send_command_feedback: true,
            show_death_messages: true,
            max_snow_accumulation_height: 1i64,
            spawn_chunk_radius: 2i64,
            respawn_radius: 10i64,
            spectators_generate_chunks: true,
            tnt_explosion_drop_decay: false,
            universal_anger: false,
            water_source_conversion: true,
            allow_entering_nether_using_portals: true,
            command_blocks_work: true,
            fire_spread_radius_around_player: -1i64,
            locator_bar: false,
            player_movement_check: true,
            pvp: true,
            spawn_monsters: true,
            spawner_blocks_work: true,
            tnt_explodes: true,
        }
    }
}
fn default_show_advancement_messages() -> bool {
    GameRuleRegistry::default().show_advancement_messages
}
fn default_block_explosion_drop_decay() -> bool {
    GameRuleRegistry::default().block_explosion_drop_decay
}
fn default_command_block_output() -> bool {
    GameRuleRegistry::default().command_block_output
}
fn default_max_block_modifications() -> i64 {
    GameRuleRegistry::default().max_block_modifications
}
fn default_disable_elytra_movement_check() -> bool {
    GameRuleRegistry::default().disable_elytra_movement_check
}
fn default_disable_raids() -> bool {
    GameRuleRegistry::default().disable_raids
}
fn default_advance_time() -> bool {
    GameRuleRegistry::default().advance_time
}
fn default_entity_drops() -> bool {
    GameRuleRegistry::default().entity_drops
}
fn default_do_fire_tick() -> bool {
    GameRuleRegistry::default().do_fire_tick
}
fn default_immediate_respawn() -> bool {
    GameRuleRegistry::default().immediate_respawn
}
fn default_spawn_phantoms() -> bool {
    GameRuleRegistry::default().spawn_phantoms
}
fn default_limited_crafting() -> bool {
    GameRuleRegistry::default().limited_crafting
}
fn default_mob_drops() -> bool {
    GameRuleRegistry::default().mob_drops
}
fn default_spawn_mobs() -> bool {
    GameRuleRegistry::default().spawn_mobs
}
fn default_spawn_patrols() -> bool {
    GameRuleRegistry::default().spawn_patrols
}
fn default_block_drops() -> bool {
    GameRuleRegistry::default().block_drops
}
fn default_spawn_wandering_traders() -> bool {
    GameRuleRegistry::default().spawn_wandering_traders
}
fn default_spread_vines() -> bool {
    GameRuleRegistry::default().spread_vines
}
fn default_spawn_wardens() -> bool {
    GameRuleRegistry::default().spawn_wardens
}
fn default_advance_weather() -> bool {
    GameRuleRegistry::default().advance_weather
}
fn default_drowning_damage() -> bool {
    GameRuleRegistry::default().drowning_damage
}
fn default_ender_pearls_vanish_on_death() -> bool {
    GameRuleRegistry::default().ender_pearls_vanish_on_death
}
fn default_fall_damage() -> bool {
    GameRuleRegistry::default().fall_damage
}
fn default_fire_damage() -> bool {
    GameRuleRegistry::default().fire_damage
}
fn default_forgive_dead_players() -> bool {
    GameRuleRegistry::default().forgive_dead_players
}
fn default_freeze_damage() -> bool {
    GameRuleRegistry::default().freeze_damage
}
fn default_global_sound_events() -> bool {
    GameRuleRegistry::default().global_sound_events
}
fn default_keep_inventory() -> bool {
    GameRuleRegistry::default().keep_inventory
}
fn default_lava_source_conversion() -> bool {
    GameRuleRegistry::default().lava_source_conversion
}
fn default_log_admin_commands() -> bool {
    GameRuleRegistry::default().log_admin_commands
}
fn default_max_command_sequence_length() -> i64 {
    GameRuleRegistry::default().max_command_sequence_length
}
fn default_max_command_forks() -> i64 {
    GameRuleRegistry::default().max_command_forks
}
fn default_max_entity_cramming() -> i64 {
    GameRuleRegistry::default().max_entity_cramming
}
fn default_mob_explosion_drop_decay() -> bool {
    GameRuleRegistry::default().mob_explosion_drop_decay
}
fn default_mob_griefing() -> bool {
    GameRuleRegistry::default().mob_griefing
}
fn default_natural_health_regeneration() -> bool {
    GameRuleRegistry::default().natural_health_regeneration
}
fn default_players_nether_portal_creative_delay() -> i64 {
    GameRuleRegistry::default().players_nether_portal_creative_delay
}
fn default_players_nether_portal_default_delay() -> i64 {
    GameRuleRegistry::default().players_nether_portal_default_delay
}
fn default_players_sleeping_percentage() -> i64 {
    GameRuleRegistry::default().players_sleeping_percentage
}
fn default_projectiles_can_break_blocks() -> bool {
    GameRuleRegistry::default().projectiles_can_break_blocks
}
fn default_random_tick_speed() -> i64 {
    GameRuleRegistry::default().random_tick_speed
}
fn default_reduced_debug_info() -> bool {
    GameRuleRegistry::default().reduced_debug_info
}
fn default_send_command_feedback() -> bool {
    GameRuleRegistry::default().send_command_feedback
}
fn default_show_death_messages() -> bool {
    GameRuleRegistry::default().show_death_messages
}
fn default_max_snow_accumulation_height() -> i64 {
    GameRuleRegistry::default().max_snow_accumulation_height
}
fn default_spawn_chunk_radius() -> i64 {
    GameRuleRegistry::default().spawn_chunk_radius
}
fn default_respawn_radius() -> i64 {
    GameRuleRegistry::default().respawn_radius
}
fn default_spectators_generate_chunks() -> bool {
    GameRuleRegistry::default().spectators_generate_chunks
}
fn default_tnt_explosion_drop_decay() -> bool {
    GameRuleRegistry::default().tnt_explosion_drop_decay
}
fn default_universal_anger() -> bool {
    GameRuleRegistry::default().universal_anger
}
fn default_water_source_conversion() -> bool {
    GameRuleRegistry::default().water_source_conversion
}
fn default_allow_entering_nether_using_portals() -> bool {
    true
}
fn default_command_blocks_work() -> bool {
    true
}
fn default_fire_spread_radius_around_player() -> i64 {
    -1i64
}
fn default_locator_bar() -> bool {
    false
}
fn default_player_movement_check() -> bool {
    true
}
fn default_pvp() -> bool {
    true
}
fn default_spawn_monsters() -> bool {
    true
}
fn default_spawner_blocks_work() -> bool {
    true
}
fn default_tnt_explodes() -> bool {
    true
}
mod as_string {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::{fmt::Display, str::FromStr};
    pub fn serialize<T: Display, S: Serializer>(
        value: &T,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        D: Deserializer<'de>,
        <T as FromStr>::Err: Display,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<T>().map_err(serde::de::Error::custom)
    }
}
