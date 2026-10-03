# Hooks and APIs added by the fork

Every hook, event and host function the `modded` branch adds to the plugin API (WIT in
`crates/pumpkin-plugin-wit/v0.1`), with the Java it stands for and the commit that added it.
Upstream Pumpkin's own API is not listed.

When you add or change a hook, add or update its row in the same commit. `FORK_CHANGES.md`
lists the fork's changes by area; this file lists the API surface a mod plugin can use.

## Block hooks (`modded.block-hooks`, `context.register-block-hooks`)

Called through the plugin export `handle-block-hook`. Hooks marked *no call* are flags the host
answers from data the plugin set.

| Hook | Java | Commit |
|:--|:--|:--|
| `placement-state` | `Block.getStateForPlacement` | `add68fe15` |
| `placed` | `Block.setPlacedBy` | `add68fe15` |
| `use` | `Block.useItemOn` / `useWithoutItem` | `add68fe15` |
| `attack` | `Block.attack` | `add68fe15` |
| `drops` | `Block.getDrops` | `add68fe15` |
| `removed` | `BlockEntity.preRemoveSideEffects` | `add68fe15` |
| `scheduled-tick` | `Block.tick` | `add68fe15` |
| `neighbor-changed` | `Block.neighborChanged` | `e51c7666c` |
| `random-tick` | `Block.randomTick` | `95105a9ab` |
| `update-shape` | `Block.updateShape` | `342715c3c` |
| `signal-source` (no call) | `Block.isSignalSource`, `getSignal`, `getDirectSignal` | `a6f9674ba` |
| `analog-output` (no call) | `Block.hasAnalogOutputSignal`, `getAnalogOutputSignal` | `a6f9674ba` |
| `entity-inside` (opt-in, batched) | `Block.entityInside` | `044588b3a`, batched in `1fa249829` |
| `step-on` (opt-in, batched) | `Block.stepOn` | `044588b3a`, batched in `1fa249829` |
| `ticker` (opt-in, batched) | `EntityBlock.getTicker` | `dfe6c259c`, batched in `1fa249829` |

## Item hooks (`modded.item-hooks`, `context.register-item-hooks`)

Called through the plugin export `handle-item-hook`.

| Hook | Java | Commit |
|:--|:--|:--|
| `use-on-block` | `Item.useOn` | `0084f1557` |
| `use` | `Item.use` | `0084f1557` |
| `inventory-tick` (opt-in, batched) | `Item.inventoryTick` | `c3b1024a1`, batched in `1fa249829` |
| `stacked-on-me` | `Item.overrideOtherStackedOnMe` | `13b9d100c` |
| `stacked-on-other` | `Item.overrideStackedOnOther` | `13b9d100c` |
| `destroyed` | `Item.onDestroyed` | `13b9d100c` |
| `not-in-containers` (no call) | `Item.canFitInsideContainerItems` | `13b9d100c` |

Hooks marked *batched* run every tick. The host queues them while the world ticks and sends
each plugin one `handle-tick-batch` call per world after block entities tick (see below).

## Other plugin exports and registrations

| API | What it does | Commit |
|:--|:--|:--|
| `scheduler.spawn-job` + exports `run-job`, `handle-job-result` | Runs `run-job` on a worker instance (own memory, no server access) on a background thread pool; the output reaches `handle-job-result` on the main instance at the start of a later tick | `81f35ff24` |
| export `handle-tick-batch` (`modded.tick-batch`) | One call per plugin, world and tick with all its batched per-tick hooks, in vanilla order: inventories, entity contacts, block entities | `1fa249829` |
| `context.register-crafting-handler` + export `handle-crafting` | Crafts a grid for recipes defined in code (a mod's `CustomRecipe`) | `981c23443` |
| `menu.open` / `update-slot` / `close` + export `handle-menu-call` | Menus whose slots the plugin defines, held by the host | `cddec52a8`, `31377bbf9` |
| `modded.open-menu` | Opens a mod's own menu type | `cddec52a8` |
| `context.register-login-query` | Login-phase custom query to each client | `c19760f9c` |
| `context.register-configuration-payload` | Configuration-phase payload to each client | `d75320470` |
| `modded.register-component-stream-codec` | A modded component's network codec, described once | `728d5c4e5` |

## Events

| Event | Java / Fabric | Commit |
|:--|:--|:--|
| `item-storage-changed-event` | Hoppers changed a plugin item storage (once per tick) | `b9be9ddc0` |
| `block-broken-event`, `block-break-canceled-event` | `PlayerBlockBreakEvents.AFTER` / `CANCELED` | `defeddb36` |
| `player-pick-item-block-event`, `player-pick-item-entity-event` | `PlayerPickItemEvents` | `d719b608c` |
| `player-start-tracking-event`, `player-stop-tracking-event` | `EntityTrackingEvents` | `9bf64aded` |
| `block-entity-load-event`, `block-entity-unload-event` | `ServerBlockEntityEvents`; also fires for a mod's block entities when their chunk loads (`fcad77c09`) | `729862d48` |
| `server-stopping-event` | `ServerLifecycleEvents.SERVER_STOPPING` | `ada7f58d6` |
| `player-login-query-response-event` | `ServerLoginNetworking` replies | `c19760f9c` |
| `player-sleep-check-event` | `EntitySleepEvents.ALLOW_SLEEP_TIME` / `ALLOW_NEARBY_MONSTERS` | `e8ceb3e1e` |
| `player-configuration-payload-event` | `ServerConfigurationNetworking` replies | `d75320470` |

## Host functions

| Function | What it does | Commit |
|:--|:--|:--|
| `world.get/set-block-entity-data`, `remove-block-entity` | A mod's block-entity data, kept as chunk NBT | `add68fe15`, moved to `world` in `fb5343e2e` |
| `world.schedule-block-tick` | `Level.scheduleTick` | `add68fe15`, moved in `25b0b7b63` |
| `world.drop-item` | `Block.popResource` | `8c64bfb06`, moved in `25b0b7b63` |
| `world.set-item-storage` / `get-item-storage` | Host-held storage slots hoppers use; slots can share a pool at a per-slot rate (`acdbb3c6a`) | `b9be9ddc0` |
| `world.set-redstone-output`, `set-comparator-output` | Redstone and comparator values held by the host | `a6f9674ba` |
| `world.set-redstone-output-sides` | Weak and strong power per side | `7b1acc125` |
| `world.get-entities-in-box` | Entities in a box | `4c0cc6ae9` |
| `world.get-block-state-ids`, `get-block-states-in-box`, `set-block-states`, `get-block-entity-data-list` | Bulk block and block-entity reads and writes in one call (whole inventories already exist upstream: `inventory.get-all-items` / `set-all-items`) | `5d96d025f` |
| `modded.set-block-collision-shape` | Per-position collision boxes held by the host | `d36eaec65` |
| `server.get-players-tracking-chunk`, `get-players-tracking-entity` | `PlayerLookup.tracking` | `4c0cc6ae9` |
| `server.set-flammable` | `FlammableBlockRegistry.add` | `786ca3cd8` |
| `player.give-item` | `Inventory.add` | `8c64bfb06`, moved in `25b0b7b63` |
| `entity.get-item-stack` / `set-item-stack` | An item entity's stack | `1d73c14ae` |
| `item-stack.get/set/remove-component-by-id` | Components by id, modded ones as NBT | `bd7f4c35e`, `b45e6363d` |
| `item-stack.to-nbt` / `from-nbt` | Saved form of a stack | `bd7f4c35e` |
| `item-stack.get-tags` / `has-tag` | The item's tags | `907bef135` |
| `recipe-manager.match-crafting` / `match-cooking` | `RecipeManager.getRecipeFor` | `f86ab3824` |
| `recipe-manager.crafting-recipes-for` | Crafting recipes by result, with shape and ingredients | `a490d007e` |

## Hook catalog

What mods hook into on the server, next to what a plugin can use today (upstream Pumpkin's
WIT plus this fork). Sources: every `Event` field in the Fabric API 0.161.0 source and its
content registries; every event class in NeoForge 26.3.x (`net.neoforged.neoforge.event`) and
its block and item extension methods; the Bukkit/Paper event set upstream Pumpkin mirrors in
`event.wit`; and the vanilla methods content mods most often reach with Mixins when no loader
hook exists.

Status: ✅ done (a plugin can do it), 🟡 partial, ❌ missing, ➖ not needed (client side, or
covered by the mod data dump). Need: **A** most content mods, **B** many, **C** few. The need
column is a judgement from what each kind of mod does (machines, storage, worldgen, mobs,
magic, food, tools), not a count over a mod corpus.

The tables below summarise by topic; "Full lists" goes through every hook of every source one
by one. Upstream declares about 290 Bukkit-style events; 56 of them are never fired by the server yet
(below). Fired ones are counted as ✅.

### Content and registries

| What mods do | Fabric API | NeoForge | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|:--|:--|
| Blocks, items, block-entity types, tags, components | `Registry.register` | `DeferredRegister` | Mod data dump, runtime overlay | ✅ | A |  |
| Menus | `ExtendedMenuType` | `IMenuTypeExtension` | `menu`, `modded.open-menu` | 🟡 | A | no data slots (progress bars), no menu buttons |
| Recipes and recipe types | `RecipeSerializer` | same | Dump; `register-crafting-handler` for code-defined crafting | 🟡 | B | no custom recipe types the host matches |
| Entity types | `EntityType.Builder` | same | none | ❌ | A |  |
| Default entity attributes | `FabricDefaultAttributeRegistry` | `EntityAttributeCreationEvent`, `EntityAttributeModificationEvent` | none | ❌ | A | with entity types |
| Loot table changes | `LootTableEvents.MODIFY`, `REPLACE`, `MODIFY_DROPS` | `LootTableLoadEvent`, global loot modifiers | `drops` block hook; `loot-generate-event` can only cancel | 🟡 | A |  |
| Biome and feature changes (ores, plants) | `BiomeModifications` | `BiomeModifier` | `set-chunk-generator` replaces the whole generator | ❌ | A |  |
| Structures | datapack + `StructureModifier` | same | none | ❌ | B |  |
| Mob effects, potions, brewing | `Registry.register`; brewing through Mixins | `PotionBrewEvent`, `MobEffectEvent` | `player.add-effect` with vanilla effects | ❌ | B | for modded ones |
| Fuel | no API in 26.3 | no API in 26.3 | none | ❌ | B | check how 26.3 defines fuel before designing |
| Strip, till, flatten, wax | `BlockTransformerEvents` | `BlockEvent.BlockToolModificationEvent`, data maps `TRANSFORMABLES`, `WAXABLES` | none | ❌ | B |  |
| Flammability | `FlammableBlockRegistry` | `IBlockExtension.getFlammability` | `server.set-flammable` | ✅ | B |  |
| Oxidation, path types, vibration frequencies, villager interactions | `OxidizableBlocksRegistry`, `LandPathTypeRegistry`, `VibrationFrequencyRegistry`, `VillagerInteractionRegistries` | data maps `OXIDIZABLES`, `VIBRATION_FREQUENCIES`, `VILLAGER_COMPOSTABLES` | none | ❌ | C |  |
| Particles and sounds | `FabricParticleTypes`, `SoundEvent` | same | `play-custom-sound`, `spawn-particle` (vanilla particle types) | 🟡 | B | modded particle types |
| Game rules | `GameRuleBuilder` | `RegisterGameRuleCategoryEvent`, `GameRuleChangedEvent` | `get/set-game-rule` for vanilla rules | ❌ | C | for modded rules |
| Commands | `CommandRegistrationCallback`, `ArgumentTypeRegistry` | `RegisterCommandsEvent` | `register-command` | 🟡 | B | modded argument types |
| Enchantments and other datapack registries | `DynamicRegistries` | `NewDatapackRegistryEvent` | none for modded entries | ❌ | B |  |
| Creative tabs, tooltips, models | client side | client side | | ➖ | |  |

### Block behaviour

| Hook | Java | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|:--|
| Place, use, attack, drops, removed, neighbour and shape updates, scheduled and random ticks, block-entity ticker, entity inside, step on, redstone and comparator output, collision shape | `Block.*`, `EntityBlock.getTicker` | `modded.block-hooks` (above) | ✅ | A |  |
| Projectile hits the block | `Block.onProjectileHit` | `projectile-hit-event` (global) | 🟡 | B |  |
| Can survive / can be placed | `BlockBehaviour.canSurvive` | none | ❌ | B |  |
| Player will destroy | `Block.playerWillDestroy`, `IBlockExtension.onDestroyedByPlayer` | `block-break-event` | ✅ | B |  |
| Destroy speed, harvest check | `BlockBehaviour.getDestroyProgress`, `PlayerEvent.BreakSpeed`, `PlayerEvent.HarvestCheck` | none | ❌ | B |  |
| Fall on, bounce | `Block.fallOn`, `updateEntityMovementAfterFallOn` | none | ❌ | C |  |
| Explosion resistance and reaction | `IBlockExtension.getExplosionResistance`, `onBlockExploded` | static resistance from the dump | 🟡 | C |  |
| Light emission from state or data | `IBlockExtension.getLightEmission` | static from the dump | 🟡 | C |  |
| Ladder, piston reaction, enchant power, redstone connection | `IBlockExtension.isLadder`, `getPistonPushReaction`, `getEnchantPowerBonus`, `canConnectRedstone` | none | ❌ | C |  |
| Hooks for vanilla blocks | Mixins into vanilla block classes | `register-block-hooks` refuses vanilla blocks | ❌ | B |  |
| Animate tick, render shape | client side | | ➖ | |  |

### Item behaviour

| Hook | Java | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|:--|
| Use on block, use, inventory tick, stacked clicks, destroyed, container rules | `Item.*` | `modded.item-hooks` (above) | ✅ | A |  |
| Use over time: finish, release, use duration | `Item.finishUsingItem`, `releaseUsing`, `getUseDuration`, `LivingEntityUseItemEvent` | `player-item-consume-event` | 🟡 | A | no hooks for plugin items |
| Use on entity | `Item.interactLivingEntity`, `UseEntityCallback` | `player-interact-entity-event` | 🟡 | B | no item hook |
| Hit and mine with the item | `Item.hurtEnemy`, `postHurtEnemy`, `mineBlock` | none | ❌ | B |  |
| Recipe remainder, enchantability, attribute modifiers | `FabricItem`, `IItemExtension` | components from the dump | 🟡 | C |  |
| Item entity tick | `IItemExtension.onEntityItemUpdate` | none | ❌ | C |  |
| Crafted by | `Item.onCraftedBy` | `craft-item-event` | ✅ | C |  |
| Tooltips, foil | client side | | ➖ | |  |

### Entities and players

| What mods do | Fabric API | NeoForge | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|:--|:--|
| Damage: allow, change, after | `ServerLivingEntityEvents.ALLOW_DAMAGE`, `AFTER_DAMAGE` | `LivingIncomingDamageEvent`, `LivingDamageEvent` | `entity-damage-event`, `entity-damage-by-entity-event` | ✅ | A |  |
| Death and kills | `ALLOW_DEATH`, `AFTER_DEATH`, `AFTER_KILLED_OTHER_ENTITY` | `LivingDeathEvent`, `LivingDropsEvent`, `LivingExperienceDropEvent` | `entity-death-event` (id and xp only), `player-death-event` | 🟡 | A | no killer, no drops |
| Per-entity data | data attachments | attachments | `set/get-custom-data` on entities, worlds, chunks | 🟡 | A | no sync, no copy on respawn |
| Copy data on respawn or dimension change | `ServerPlayerEvents.COPY_FROM`, `AFTER_RESPAWN` | `PlayerEvent.Clone` | `player-respawn-event` | 🟡 | B |  |
| Join, leave, respawn, change world | `ServerPlayConnectionEvents`, `ServerEntityLevelChangeEvents` | `PlayerEvent.PlayerLoggedIn` and others | player events | ✅ | A |  |
| Entity load and unload | `ServerEntityEvents.ENTITY_LOAD`, `ENTITY_UNLOAD` | `EntityJoinLevelEvent`, `EntityLeaveLevelEvent` | `entity-spawn-event`, `entities-load-event`, `entities-unload-event` | ✅ | B |  |
| Equipment change | `ServerEntityEvents.EQUIPMENT_CHANGE` | `LivingEquipmentChangeEvent` | none | ❌ | B |  |
| Player tick | `END_SERVER_TICK` loops | `PlayerTickEvent` | `server-tick-*` events, scheduler | 🟡 | B |  |
| Mob spawning rules and finalize | `BiomeModifications.addSpawn` | `MobSpawnEvent`, `FinalizeSpawnEvent`, `RegisterSpawnPlacementsEvent` | `creature-spawn-event` | 🟡 | B | no modded spawn entries |
| Sleep | `EntitySleepEvents` (11 events) | `CanPlayerSleepEvent`, `PlayerWakeUpEvent`, `SleepFinishedTimeEvent` | `player-sleep-check-event`, `player-bed-enter/leave-event`, `time-skip-event` | 🟡 | C | no bed direction or wake-up position |
| Elytra | `EntityElytraEvents` | none | `entity-toggle-glide-event` | 🟡 | C |  |
| Knockback, fall, breathe, heal, totem | none | `LivingKnockBackEvent`, `LivingFallEvent`, `LivingBreatheEvent`, `LivingHealEvent`, `LivingUseTotemEvent` | `entity-regain-health-event`, `entity-resurrect-event`, `entity-air-change-event`; knockback declared, not fired | 🟡 | C |  |
| Attack, critical hits, sweep | `AttackEntityCallback` | `AttackEntityEvent`, `CriticalHitEvent`, `SweepAttackEvent` | `entity-damage-by-entity-event` | 🟡 | C |  |
| Mob conversion | `MOB_CONVERSION` | `LivingConversionEvent` | `entity-transform-event` | ✅ | C |  |
| AI goals | Mixins | Mixins | `add-ai-goal`, `add-custom-ai-goal` | ✅ | B |  |
| Custom synced entity data | `FabricEntityDataRegistry` | `NeoForgeRegistries.ENTITY_DATA_SERIALIZERS` | none | ❌ | B | with entity types |

### World, server and data

| What mods do | Fabric API | NeoForge | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|:--|:--|
| Server start, stop, tick | `ServerLifecycleEvents`, `ServerTickEvents` | `ServerStartingEvent` and others, `ServerTickEvent` | `server-load-event`, `server-stopping-event`, `server-tick-start/end-event` | ✅ | A |  |
| Per-world tick | `START_LEVEL_TICK`, `END_LEVEL_TICK` | `LevelTickEvent` | none | ❌ | B |  |
| Datapack reload, tags loaded, data sync | `START/END_DATA_PACK_RELOAD`, `SYNC_DATA_PACK_CONTENTS`, `CommonLifecycleEvents.TAGS_LOADED` | `AddServerReloadListenersEvent`, `TagsUpdatedEvent`, `OnDatapackSyncEvent` | none | ❌ | B |  |
| World load, unload, save | `ServerLevelEvents` | `LevelEvent` | `world-load/unload/save-event` | ✅ | B |  |
| Chunk load, unload, generate | `ServerChunkEvents` | `ChunkEvent`, `ChunkDataEvent` | `chunk-load/unload/save-event`, `chunk-populate-event` | ✅ | B |  |
| Block-entity load and unload | `ServerBlockEntityEvents` | `ChunkEvent` + `onLoad` | `block-entity-load/unload-event` | ✅ | A |  |
| Block break and place | `PlayerBlockBreakEvents`, `UseBlockCallback` | `BlockEvent.BreakEvent`, `EntityPlaceEvent` | `block-break-event`, `block-broken-event`, `block-place-event` | ✅ | A |  |
| Explosions, pistons, crops, note blocks, bonemeal | none | `ExplosionEvent`, `PistonEvent`, `CropGrowEvent`, `NoteBlockEvent`, `BonemealEvent` | matching Bukkit events | ✅ | B |  |
| Fluid source creation, fluid behaviour | `FluidStorage` | `CreateFluidSourceEvent`, `FluidType` | none | ❌ | B |  |
| Item, fluid and energy access from neighbours (pipes, hoppers) | `ItemStorage.SIDED`, `FluidStorage.SIDED`, `BlockApiLookup` | `Capabilities.ItemHandler`, `FluidHandler`, `EnergyStorage` | `world.set-item-storage` (hoppers, no sides, no transactions) | 🟡 | A |  |
| Chat and messages | `ServerMessageEvents` | `ServerChatEvent` | `player-chat-event`, `server-broadcast-event` | ✅ | C |  |
| Permissions | `PermissionEvents.ON_REQUEST` | `PermissionGatherEvent` | `player-permission-check-event` | ✅ | C |  |
| Advancements | `fabric-advancement-api-v1` | `AdvancementEvent` | `player-advancement-done-event` | ✅ | C |  |
| Background work (autocrafting, pathfinding) | threads | threads | none | ❌ | B | `ROADMAP.md`, item 3 |

### Full lists

Every server-side hook in each source, one row per method or event (or per group of
events that map to the same Pumpkin hook). Client-only ones (rendering, particles shown
by the client, tooltips, creative tabs, sounds played locally) are left out.

#### Vanilla `Block` and `BlockBehaviour` methods

Methods a mod's block overrides (60 in `BlockBehaviour`, 33 in `Block`, from the 26.3 jar). Getters for registry data (`asItem`, `getName`, `properties`, state definition) are covered by the mod data dump.

| Method | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|
| `getStateForPlacement` | `placement-state` block hook | ✅ | A |  |
| `setPlacedBy` | `placed` block hook | ✅ | A |  |
| `useItemOn`, `useWithoutItem` | `use` block hook | ✅ | A |  |
| `attack` | `attack` block hook | ✅ | B |  |
| `getDrops` | `drops` block hook | ✅ | A |  |
| `tick` | `scheduled-tick` block hook, `world.schedule-block-tick` | ✅ | A |  |
| `randomTick`, `isRandomlyTicking` | `random-tick` block hook; random ticking from the mod data dump | ✅ | B |  |
| `neighborChanged` | `neighbor-changed` block hook | ✅ | A |  |
| `updateShape` | `update-shape` block hook | ✅ | A |  |
| `isSignalSource`, `getSignal`, `getDirectSignal` | `signal-source` flag, `world.set-redstone-output(-sides)` | ✅ | B | Host-held values, no call |
| `hasAnalogOutputSignal`, `getAnalogOutputSignal` | `analog-output` flag, `world.set-comparator-output` | ✅ | B | Host-held value, no call |
| `entityInside` | `entity-inside` block hook (batched) | ✅ | B |  |
| `stepOn` | `step-on` block hook (batched) | ✅ | C |  |
| `getCollisionShape`, `getShape`, `getInteractionShape`, `getBlockSupportShape`, `isCollisionShapeFullBlock` | shapes from the mod data dump; `modded.set-block-collision-shape` per position | 🟡 | B | Collision only; outline, interaction and support shapes are per state |
| `getEntityInsideCollisionShape` | none | ❌ | C |  |
| `affectNeighborsAfterRemoval` | `removed` block hook | 🟡 | B | Called before removal, as `BlockEntity.preRemoveSideEffects`; no hook for blocks without a block entity |
| `onPlace` | none | ❌ | B | Any placement (commands, pistons, worldgen), unlike `setPlacedBy` |
| `playerWillDestroy`, `playerDestroy`, `destroy` | `block-break-event`, `block-broken-event` | 🟡 | B | Global events, no per-block hook |
| `spawnAfterBreak`, `popExperience`, `tryDropExperience` | experience from the mod data dump | 🟡 | B | No hook for drops after break (silk touch checks, bonus items) |
| `onExplosionHit`, `wasExploded`, `dropFromExplosion` | none | ❌ | C |  |
| `onProjectileHit` | `projectile-hit-event` (global) | 🟡 | B |  |
| `canSurvive` | none | ❌ | B |  |
| `canBeReplaced` | replaceable from the dump's state flags | 🟡 | C | No per-context answer |
| `getDestroyProgress`, `defaultDestroyTime` | hardness from the mod data dump | 🟡 | B | No per-player override |
| `getExplosionResistance` | blast resistance from the mod data dump | ✅ | C |  |
| `getFriction`, `getSpeedFactor`, `getJumpFactor` | from the mod data dump | ✅ | C |  |
| `fallOn`, `bounceOn`, `getFallDistanceReduction`, `getBounceRestitution` | none | ❌ | C | Slime-like and hay-like blocks |
| `getLightDampening`, `propagatesSkylightDown`, `getOcclusionShape`, `useShapeForLightOcclusion` | opacity and shapes from the mod data dump | ✅ | C |  |
| `getCloneItemStack` | `player-pick-item-block-event` | ✅ | C |  |
| `getMenuProvider` | `menu.open`, `modded.open-menu` from `use` | ✅ | A |  |
| `triggerEvent` | none | ❌ | C | Block events (chest lid, note block) sent to clients |
| `isPathfindable` | none | ❌ | C |  |
| `rotate`, `mirror` | none | ❌ | C | Structure placement |
| `shouldChangedStateKeepBlockEntity` | none | ❌ | C |  |
| `handlePrecipitation` | none | ❌ | C | Cauldron-like filling in rain |
| `isPossibleToRespawnInThis` | none | ❌ | C |  |
| `shouldRedstoneWireConnectTo` | none | ❌ | C |  |
| `defaultMapColor` | from the mod data dump | ✅ | C |  |
| `getSoundType` | from the mod data dump (block sounds are vanilla packets) | ✅ | C |  |
| `getFluidState` | none | ❌ | B | Waterlogged modded blocks, modded fluids |
| `getMaxHorizontalOffset`, `getMaxVerticalOffset` | none | ❌ | C | Offset shapes (like flowers) |
| `showAsInteractableInSpectatorMode` | none | ❌ | C |  |

#### Vanilla `Item` methods

Overridable methods of `Item` (39). Tooltip and bar display methods are client-only.

| Method | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|
| `useOn` | `use-on-block` item hook | ✅ | A |  |
| `use` | `use` item hook | ✅ | A |  |
| `inventoryTick` | `inventory-tick` item hook (batched) | ✅ | B |  |
| `overrideOtherStackedOnMe`, `overrideStackedOnOther` | `stacked-on-me`, `stacked-on-other` item hooks | ✅ | B |  |
| `onDestroyed` | `destroyed` item hook | ✅ | C |  |
| `canFitInsideContainerItems` | `not-in-containers` flag | ✅ | C |  |
| `finishUsingItem` | `player-item-consume-event` | 🟡 | A | No hook for plugin items (food, potions, custom use) |
| `releaseUsing` | none | ❌ | A | Bows, tridents, charged items |
| `onUseTick` | none | ❌ | B |  |
| `getUseDuration`, `getUseAnimation` | the `consumable` component | 🟡 | B | No code-defined duration |
| `useOnRelease` | none | ❌ | C |  |
| `interactLivingEntity` | `player-interact-entity-event` | 🟡 | B | No item hook |
| `hurtEnemy`, `postHurtEnemy` | none | ❌ | B | Weapons with on-hit effects |
| `mineBlock` | none | ❌ | B | Tools with on-break effects |
| `getDestroySpeed`, `isCorrectToolForDrops`, `canDestroyBlock` | the `tool` component | 🟡 | B | No code-defined answer |
| `getAttackDamageBonus`, `getItemDamageSource` | none | ❌ | C |  |
| `onCraftedBy`, `onCraftedPostProcess` | `craft-item-event` | 🟡 | C | No item hook |
| `getDefaultMaxStackSize`, `components`, `getDefaultInstance` | from the mod data dump | ✅ | B |  |
| `shouldPrintOpWarning` | none | ➖ |  | Tooltip |

#### Vanilla block entities

`BlockEntity` (33 overridable methods) and the block-entity side of `EntityBlock`.

| Method | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|
| `EntityBlock.getTicker` | `ticker` block hook (batched) | ✅ | A |  |
| `loadAdditional`, `saveAdditional` | `world.get/set-block-entity-data` (kept as chunk NBT) | ✅ | A |  |
| `preRemoveSideEffects` | `removed` block hook | ✅ | A |  |
| `setChanged` | `world.set-block-entity-data` | ✅ | A |  |
| `getUpdateTag`, `getUpdatePacket` | `world.set-block-entity-data` sends the data | 🟡 | B | Sends all data; no client-only subset |
| `applyImplicitComponents`, `collectImplicitComponents`, `removeComponentsFromTag` | none | ❌ | B | Item components copied into and out of the block entity; plugins do it in `placed` and `drops` |
| `triggerEvent` | none | ❌ | C |  |
| `isValidBlockState` | none | ❌ | C |  |
| `onLoad`, `onChunkUnloaded` (NeoForge) | `block-entity-load-event`, `block-entity-unload-event` | ✅ | A |  |
| Container block entities (`BaseContainerBlockEntity`, `WorldlyContainer`) | `world.set-item-storage` (hopper slots) | 🟡 | A | No sides, no transactions |

#### Vanilla menus

`AbstractContainerMenu` (43 overridable methods) and `Slot`.

| Method | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|
| `quickMoveStack` | `quick-move` menu call | ✅ | A |  |
| `Slot.mayPlace`, `mayPickup`, `getMaxStackSize`, `set` | menu calls | ✅ | A |  |
| `removed` | `closed` menu call | ✅ | A |  |
| `stillValid` | menu anchor | ✅ | A |  |
| `addDataSlot`, `setData` (progress bars, furnace-like data) | none | ❌ | A | Machines show progress through data slots |
| `clickMenuButton` | none | ❌ | B | Buttons in menus (enchanting, stonecutter, mod GUIs) |
| `clicked` (all click types) | host handles clicks; `stacked-on-*` item hooks | 🟡 | C |  |
| `slotsChanged` | `set-item` menu call | ✅ | B |  |
| `canTakeItemForPickAll`, `canDragTo` | none | ❌ | C |  |

#### Vanilla entities and other content classes

A mod adding an entity, effect, fluid or recipe type overrides these; none of these types can be added yet, so each class is one row.

| Class | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|
| `Entity` (tick, hurt, interact, save/load, collision, passengers, portals: 408 overridable methods) | none for modded entity types | ❌ | A | Vanilla entities: upstream entity API and events |
| `LivingEntity` (damage, death loot, effects, equipment, travel: 311 methods) | none for modded types | ❌ | A |  |
| `Mob` (`registerGoals`, `mobInteract`, `finalizeSpawn`, `checkSpawnRules`, `removeWhenFarAway`: 131 methods) | `add-ai-goal`, `add-custom-ai-goal` on vanilla mobs | 🟡 | A | No modded mob types |
| `Projectile` (`onHitBlock`, `onHitEntity`, `shoot`, `canHitEntity`: 40 methods) | `projectile-hit-event`, `projectile-launch-event` | 🟡 | B | No modded projectiles |
| `Player` overrides (`attack`, `interactOn`, `getDestroySpeed`, `hasCorrectToolForDrops`) | player events | 🟡 | B | Through events only |
| `MobEffect` (`applyEffectTick`, `onEffectStarted`, `onEffectAdded`, `onMobHurt`, `onMobRemoved`, attribute modifiers) | `player.add-effect` (vanilla effects) | ❌ | B | No modded effects |
| `Fluid` / `FlowingFluid` (`tick`, `randomTick`, `entityInside`, spread, `canBeReplacedWith`) | none | ❌ | B | No modded fluids |
| `Enchantment` effects | datapack enchantments (vanilla effect components) | 🟡 | B | No modded enchantment effect types |
| `Recipe` / `RecipeSerializer` (custom matching and assembly) | `register-crafting-handler` (crafting grid only) | 🟡 | B | No custom recipe types for other stations |

#### NeoForge `IBlockExtension` methods

Default methods NeoForge adds to every block (`common/extensions/IBlockExtension`). Client-only ones (`addLandingEffects`, `addRunningEffects`, `getAppearance`, `hidesNeighborFace`, `shouldDisplayFluidOverlay`, `shouldHideAdjacentFluidFace`, `supportsExternalFaceHiding`, `getStateAtViewpoint`, `getMapColor` on the client) are left out.

| Method | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|
| `canHarvestBlock` | none | ❌ | B |  |
| `onDestroyedByPlayer` | `block-break-event` | 🟡 | B |  |
| `getExpDrop` | experience from the mod data dump | 🟡 | B |  |
| `getCloneItemStack` | `player-pick-item-block-event` | ✅ | C |  |
| `getExplosionResistance` (per position) | static value | 🟡 | C |  |
| `onBlockExploded`, `canDropFromExplosion` | none | ❌ | C |  |
| `getLightEmission`, `hasDynamicLightEmission` | static per state | 🟡 | C |  |
| `getFriction` (per position) | static value | 🟡 | C |  |
| `isLadder`, `isScaffolding`, `makesOpenTrapdoorAboveClimbable` | none | ❌ | C |  |
| `getFlammability`, `getFireSpreadSpeed`, `isFlammable`, `onCaughtFire`, `isFireSource`, `isBurning`, `ignitedByLava` | `server.set-flammable` | 🟡 | C | Static values only |
| `canSustainPlant`, `isFertile`, `canBeHydrated`, `onTreeGrow` | none | ❌ | C |  |
| `getToolModifiedState` | none | ❌ | B | Strip, till, flatten |
| `getEnchantPowerBonus` | none | ❌ | C |  |
| `getRespawnPosition` | none | ❌ | C |  |
| `isStickyBlock`, `canStickTo`, `getPistonPushReaction` | piston behaviour from the mod data dump | 🟡 | C |  |
| `getBlockPathType`, `getAdjacentBlockPathType` | none | ❌ | C |  |
| `onNeighborChange`, `getWeakChanges`, `shouldCheckWeakPower` | `neighbor-changed` block hook | 🟡 | C | Comparator-style neighbour reads |
| `onBlockStateChange` | none | ❌ | C |  |
| `canEntityDestroy` | none | ❌ | C |  |
| `isSlimeBlock`, `getBounceRestitution` | none | ❌ | C |  |
| `getBeaconColorMultiplier` | none | ❌ | C |  |
| `playFallSound`, `playStepSound`, `getSoundType` | sound type from the dump | 🟡 | C |  |
| `getBubbleColumnDirection` | none | ❌ | C |  |
| `collisionExtendsVertically` | none | ❌ | C |  |
| `getRelocability` | none | ❌ | C |  |
| `rotate` | none | ❌ | C |  |

#### NeoForge `IItemExtension` methods

Client-only ones (`getHighlightTip`, `shouldCauseReequipAnimation`, `shouldCauseBlockBreakReset`, `getCreatorModId`) are left out.

| Method | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|
| `onItemUseFirst` | none | ❌ | B | Before the block is used |
| `onLeftClickEntity`, `onEntitySwing` | none | ❌ | B |  |
| `onDroppedByPlayer` | `player-drop-item-event` | ✅ | C |  |
| `onEntityItemUpdate`, `getEntityLifespan`, `hasCustomEntity`, `createEntity` | none | ❌ | C | Item entities |
| `canPerformAction` (item abilities) | none | ❌ | B |  |
| `getCraftingRemainder` | from the mod data dump | 🟡 | C | No code-defined remainder |
| `damageItem`, `isDamageable`, `getMaxDamage`, `setDamage`, `canBeHurtBy` | components | 🟡 | C |  |
| `getDefaultAttributeModifiers` | components | ✅ | C |  |
| `canEquip`, `getEquipmentSlot` | the `equippable` component | ✅ | C |  |
| `isPiglinCurrency`, `makesPiglinsNeutral`, `isGazeDisguise`, `canWalkOnPowderedSnow` | none | ❌ | C |  |
| `onStopUsing`, `canContinueUsing` | none | ❌ | B |  |
| `supportsEnchantment`, `applyEnchantments`, `getEnchantmentLevel`, `getAllEnchantments` | enchantment tags | 🟡 | C |  |
| `canGrindstoneRepair`, `getXpRepairRatio` | none | ❌ | C |  |
| `onAnimalArmorTick` | none | ❌ | C |  |
| `onGlideDamage` | none | ❌ | C |  |
| `doesSneakBypassUse` | none | ❌ | C |  |
| `isPrimaryItemFor` | enchantment tags | ✅ | C |  |
| `getSweepHitBox` | none | ❌ | C |  |
| `isNotReplaceableByPickAction` | none | ❌ | C |  |

#### Other NeoForge extensions, capabilities and data maps



| Extension | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|
| `IEntityExtension`: fluid interaction, multipart entities, `sendPairingData` | none | ❌ | B | With modded entities |
| `ILivingEntityExtension`: `onDamageTaken`, swimming in modded fluids | none | ❌ | C |  |
| `IPlayerExtension`: `mayFly`, `openMenu` with extra data, `isFakePlayer` | `modded.open-menu` | 🟡 | B | No fake players |
| `IFluidExtension` / `FluidType`: density, viscosity, drowning, extinguishing, boats, placement | none | ❌ | B | Modded fluids |
| `IBaseRailBlockExtension`: `canMakeSlopes`, `isValidRailShape` | none | ❌ | C |  |
| `IFallableExtension.fallingTick` | none | ❌ | C |  |
| `IBucketPickupExtension`, `IDispensibleContainerItemExtension` | none | ❌ | C | Modded buckets |
| `IAbstractBoatExtension.canBoatInFluid` | none | ❌ | C |  |
| `IMobEffectExtension` | none | ❌ | C |  |
| `IMenuProviderExtension.writeClientSideData` | `modded.open-menu` data | ✅ | A |  |
| Capabilities: `Capabilities.Item`, `Fluid`, `Energy` for blocks, entities and items, plus `ENTITY_AUTOMATION` | `world.set-item-storage` (block items only) | 🟡 | A | No fluid, energy, entity or item capabilities; no sides |
| Data maps (`NeoForgeDataMaps`: compostables, oxidizables, waxables, transformables, vibration frequencies, villager types, parrot imitations, raid hero gifts, monster room mobs) | none | ❌ | C |  |

#### Fabric API events

Every `Event` field in the Fabric API 0.161.0 main sources (client sources left out), by class.

| Event | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|
| `ServerLifecycleEvents`: `SERVER_STARTING`, `SERVER_STARTED`, `SERVER_STOPPING`, `SERVER_STOPPED` | `server-load-event`, `server-stopping-event` | 🟡 | A | No separate starting and stopped events |
| `ServerLifecycleEvents`: `SYNC_DATA_PACK_CONTENTS`, `START_DATA_PACK_RELOAD`, `END_DATA_PACK_RELOAD` | none | ❌ | B |  |
| `ServerLifecycleEvents`: `BEFORE_SAVE`, `AFTER_SAVE` | `world-save-event` | 🟡 | C |  |
| `ServerTickEvents`: `START_SERVER_TICK`, `END_SERVER_TICK` | `server-tick-start-event`, `server-tick-end-event` | ✅ | A |  |
| `ServerTickEvents`: `START_LEVEL_TICK`, `END_LEVEL_TICK` | none | ❌ | B |  |
| `CommonLifecycleEvents.TAGS_LOADED` | none | ❌ | C |  |
| `ServerLevelEvents`: `LOAD`, `UNLOAD` | `world-load-event`, `world-unload-event` | ✅ | B |  |
| `ServerChunkEvents`: `CHUNK_LOAD`, `CHUNK_UNLOAD`, `CHUNK_GENERATE` | `chunk-load-event`, `chunk-unload-event`, `chunk-populate-event` | ✅ | B |  |
| `ServerChunkEvents.FULL_CHUNK_STATUS_CHANGE` | none | ❌ | C |  |
| `ServerBlockEntityEvents`: `BLOCK_ENTITY_LOAD`, `BLOCK_ENTITY_UNLOAD` | `block-entity-load-event`, `block-entity-unload-event` | ✅ | A |  |
| `ServerEntityEvents`: `ENTITY_LOAD`, `ENTITY_UNLOAD` | `entity-spawn-event`, `entities-load-event`, `entities-unload-event` | ✅ | B |  |
| `ServerEntityEvents.ALLOW_LOAD` | none | ❌ | C | Refuse an entity loaded from disk |
| `ServerEntityEvents.EQUIPMENT_CHANGE` | none | ❌ | B |  |
| `ServerEntityLevelChangeEvents`: `AFTER_ENTITY_CHANGE_LEVEL`, `AFTER_PLAYER_CHANGE_LEVEL` | `player-change-world-event`; `entity-portal-event` | 🟡 | C | Non-player entities: before, not after |
| `ServerLivingEntityEvents`: `ALLOW_DAMAGE`, `AFTER_DAMAGE` | `entity-damage-event` | ✅ | A |  |
| `ServerLivingEntityEvents`: `ALLOW_DEATH`, `AFTER_DEATH` | `entity-death-event`, `player-death-event` | 🟡 | A | Death can't be cancelled for non-players; no damage source |
| `ServerLivingEntityEvents.MOB_CONVERSION` | `entity-transform-event` | ✅ | C |  |
| `ServerEntityCombatEvents.AFTER_KILLED_OTHER_ENTITY` | none | ❌ | B |  |
| `ServerMobEffectEvents`: `ALLOW_ADD`, `BEFORE_ADD`, `AFTER_ADD`, `ALLOW_EARLY_REMOVE`, `BEFORE_REMOVE`, `AFTER_REMOVE` | `entity-potion-effect-event` | 🟡 | B | One event for add and remove |
| `ServerPlayerEvents`: `JOIN`, `LEAVE`, `AFTER_RESPAWN` | `player-join-event`, `player-leave-event`, `player-respawn-event` | ✅ | A |  |
| `ServerPlayerEvents.COPY_FROM` | none | ❌ | B | Copy data to the respawned player |
| `ServerPlayerEvents.ALLOW_DEATH` | `player-death-event` (cancellable) | ✅ | B |  |
| `EntitySleepEvents` (10 events) | `player-sleep-check-event`, `player-bed-enter/leave-event` | 🟡 | C | No bed direction, occupation or wake-up position |
| `EntityElytraEvents`: `ALLOW`, `CUSTOM` | `entity-toggle-glide-event` | 🟡 | C |  |
| `AttackBlockCallback` | `attack` block hook; `block-damage-event` | ✅ | B |  |
| `AttackEntityCallback` | `entity-damage-by-entity-event` | 🟡 | B | After damage starts, not before the attack |
| `UseBlockCallback`, `BlockEvents.USE_ITEM_ON`, `USE_WITHOUT_ITEM` | `player-interact-event` | ✅ | A |  |
| `UseItemCallback`, `ItemEvents.USE`, `USE_ON` | `player-interact-event` | ✅ | A |  |
| `UseEntityCallback` | `player-interact-entity-event` | ✅ | B |  |
| `PlayerBlockBreakEvents`: `BEFORE`, `AFTER`, `CANCELED` | `block-break-event`, `block-broken-event`, `block-break-canceled-event` | ✅ | A |  |
| `PlayerPickItemEvents`: `BLOCK`, `ENTITY` | `player-pick-item-block/entity-event` | ✅ | C |  |
| `ItemClickBehaviorCallback` | `stacked-on-*` item hooks | 🟡 | C | Only for plugin items |
| `BlockTransformerEvents.MODIFY` | none | ❌ | B | Strip, till, flatten |
| `DefaultItemComponentEvents.MODIFY` | none | ❌ | B | Change vanilla items' default components |
| `EnchantmentEvents`: `ALLOW_ENCHANTING`, `MODIFY`, `MODIFY_WITH_LOOKUP` | `prepare-item-enchant-event`, `enchant-item-event` | 🟡 | C | No enchantment definition changes |
| `LootTableEvents`: `REPLACE`, `MODIFY`, `ALL_LOADED`, `MODIFY_DROPS` | `loot-generate-event` (cancel only) | 🟡 | A |  |
| `AdvancementEvents`: `REPLACE`, `MODIFY`, `ALL_LOADED` | none | ❌ | C |  |
| `FabricDefaultAttributeRegistry.MODIFY` | none | ❌ | B |  |
| `FluidFlowEvents.ALLOW` | `block-from-to-event` | ✅ | C |  |
| `DimensionEvents.MODIFY_ATTRIBUTES` | none | ❌ | C |  |
| `ServerMessageEvents` (6 events), `ServerMessageDecoratorEvent` | `player-chat-event`, `server-broadcast-event`, `player-command-send-event` | 🟡 | C | No decorator |
| `CommandRegistrationCallback` | `context.register-command` | ✅ | A |  |
| `PermissionEvents`: `ON_REQUEST`, `PREPARE_OFFLINE_PLAYER` | `player-permission-check-event` | 🟡 | C |  |
| `EntityTrackingEvents`: `START_TRACKING`, `STOP_TRACKING` | `player-start/stop-tracking-event` | ✅ | B |  |
| `ServerPlayConnectionEvents`: `INIT`, `JOIN`, `DISCONNECT` | `player-login-event`, `player-join-event`, `player-leave-event` | ✅ | A |  |
| `ServerConfigurationConnectionEvents`: `BEFORE_CONFIGURE`, `CONFIGURE`, `DISCONNECT` | `register-configuration-payload` | 🟡 | B | No configuration tasks that wait |
| `ServerLoginConnectionEvents`: `INIT`, `QUERY_START`, `DISCONNECT` | `register-login-query` | 🟡 | C |  |
| `DynamicRegistrySetupCallback` | none | ❌ | C |  |
| `CreativeModeTabEvents.MODIFY_OUTPUT_ALL` | none | ➖ |  | Client display |

#### Fabric API interfaces, registries and helpers

The non-event API of each server-side Fabric API module.

| API | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|
| `FabricItem`: `getCraftingRemainder`, `canBeEnchantedWith`, `allowContinuingBlockBreaking`, `allowComponentsUpdateAnimation` | none | ❌ | C |  |
| `FabricBlock`: `getProvidedEnchantmentPower` | none | ❌ | C |  |
| `FabricMobEffect`: `onEffectAdded`, `onEffectStarted`, `onEffectRemoved` | none | ❌ | B | Modded effects |
| `CustomDamageHandler.hurtAndBreak` | none | ❌ | C |  |
| `EquipmentSlotProvider` | the `equippable` component | ✅ | C |  |
| `BlockAttackInteractionAware.onAttackInteraction` | `attack` block hook | ✅ | C |  |
| `MinecartComparatorLogic` | none | ❌ | C |  |
| `FluidBehavior`, `EntityFluidInteractionRegistry` | none | ❌ | B | Modded fluids |
| Transfer API: `ItemStorage.SIDED`, `FluidStorage.SIDED`, `ITEM`, `Transaction`, `SidedStorageBlockEntity`, `ContainerItemContext`, `PlayerInventoryStorage` | `world.set-item-storage` | 🟡 | A | Block items only, no sides, no transactions, no fluids |
| API lookup: `BlockApiLookup`, `ItemApiLookup`, `EntityApiLookup` | none | ❌ | A | Generic lookups other mods expose (energy, fluids) |
| Data attachments: `AttachmentRegistry`, `AttachmentType`, `GlobalAttachments`, `AttachmentSyncPredicate` | `set/get-custom-data` on entities, worlds, chunks, block entities | 🟡 | A | No copy rules, no sync |
| `BiomeModifications`, `BiomeSelectors`, `NetherBiomes`, `TheEndBiomes` | none | ❌ | A |  |
| `FabricBlockEntityTypeBuilder`, `FabricEntityType`, `BlockSetTypeBuilder`, `WoodTypeBuilder`, `PoiHelper` | block entities and blocks from the mod data dump | 🟡 | B | No entity types, no POI types |
| `FabricEntityDataRegistry` | none | ❌ | B |  |
| `FabricParticleTypes`, `FabricBlockParticleOption` | none | ❌ | C |  |
| `ArgumentTypeRegistry`, `EntitySelectorOptionRegistry` | none | ❌ | C |  |
| `GameRuleBuilder`, `GameRuleEvents` | none | ❌ | C |  |
| `FlammableBlockRegistry` | `server.set-flammable` | ✅ | B |  |
| `OxidizableBlocksRegistry`, `LandPathTypeRegistry`, `VibrationFrequencyRegistry`, `VillagerInteractionRegistries` | none | ❌ | C |  |
| `CustomIngredient`, `DefaultCustomIngredients` | Fabric custom ingredients load (`2717a2ac3`) | ✅ | B |  |
| `FabricRecipeManager`, `RecipeSynchronization` | `recipe-manager.match-*` | 🟡 | B | No recipe sync |
| `ResourceConditions` | none | ❌ | C | Datapack conditions |
| `ResourceLoader`, `SimpleReloadListener`, `DataResourceLoader` | none | ❌ | B | Mod data files reloaded with datapacks |
| `FabricLootTableBuilder`, `FabricLootPoolBuilder` | none | ❌ | A | With loot table changes |
| `DynamicRegistries`, `FabricRegistryBuilder`, `RegistryEntryAddedCallback` | mod data dump | 🟡 | B | No modded datapack registries |
| `PlayerLookup` | `server.get-players-tracking-*` | ✅ | B |  |
| `FakePlayer` | none | ❌ | B | Machines that act as a player |
| `MoreCodecs`, `FabricValueInput/Output` | none | ➖ |  | Java-side serialization helpers |

#### NeoForge events

Every event class in NeoForge 26.3.x outside its client package (267 classes, nested ones included; abstract bases folded into their subclasses).

| Event | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|
| `ServerAboutToStartEvent`, `ServerStartingEvent`, `ServerStartedEvent`, `ServerStoppingEvent`, `ServerStoppedEvent` | `server-load-event`, `server-stopping-event` | 🟡 | A |  |
| `ServerTickEvent.Pre/Post` | `server-tick-start/end-event` | ✅ | A |  |
| `LevelTickEvent.Pre/Post` | none | ❌ | B |  |
| `PlayerTickEvent.Pre/Post` | none | ❌ | B |  |
| `EntityTickEvent.Pre/Post` | none | ❌ | C |  |
| `LevelEvent.Load/Unload/Save` | `world-load/unload/save-event` | ✅ | B |  |
| `LevelEvent.CreateSpawnPosition`, `PotentialSpawns` | `spawn-change-event` | 🟡 | C |  |
| `ChunkEvent.Load/Unload`, `ChunkDataEvent.Load/Save` | `chunk-load/unload/save-event` | ✅ | B |  |
| `ChunkWatchEvent.Watch/Sent/UnWatch` | `chunk-send-event` | 🟡 | C |  |
| `ChunkTicketLevelUpdatedEvent`, `RegisterTicketControllersEvent` | none | ❌ | C | Chunk loaders |
| `BlockEvent.BreakEvent` (`BreakBlockEvent`), `EntityPlaceEvent`, `EntityMultiPlaceEvent` | `block-break-event`, `block-place-event` | 🟡 | A | Multi-place not fired |
| `BlockEvent.NeighborNotifyEvent` | `block-physics-event` | ✅ | C |  |
| `BlockEvent.FluidPlaceBlockEvent` | `block-form-event` | ✅ | C |  |
| `BlockEvent.FarmlandTrampleEvent` | none | ❌ | C |  |
| `BlockEvent.PortalSpawnEvent` | `portal-create-event` | ✅ | C |  |
| `BlockEvent.BlockToolModificationEvent` | none | ❌ | B |  |
| `BlockDropsEvent` | none | ❌ | B | Change drops of any block |
| `BlockGrowFeatureEvent` | `structure-grow-event` | ✅ | C |  |
| `CropGrowEvent.Pre/Post` | `block-grow-event` | ✅ | C |  |
| `CreateFluidSourceEvent` | none | ❌ | C |  |
| `AlterGroundEvent` | none | ❌ | C |  |
| `ExplosionEvent.Start/Detonate`, `ExplosionKnockbackEvent` | `entity-explode-event`, `block-explode-event` | 🟡 | B | No knockback event |
| `PistonEvent.Pre/Post` | `block-piston-extend/retract-event` | ✅ | C |  |
| `NoteBlockEvent.Play/Change` | `note-play-event` | 🟡 | C | No change event |
| `SleepFinishedTimeEvent` | `time-skip-event` | ✅ | C |  |
| `GameRuleChangedEvent` | none | ❌ | C |  |
| `ModifyCustomSpawnersEvent` | none | ❌ | C |  |
| `VillageSiegeEvent` | none | ❌ | C |  |
| `EntityJoinLevelEvent`, `EntityLeaveLevelEvent` | `entity-spawn-event`, `entities-unload-event` | ✅ | B |  |
| `EntityEvent.EntityConstructing`, `EnteringSection`, `Size` | none | ❌ | C |  |
| `EntityInvulnerabilityCheckEvent` | none | ❌ | C |  |
| `EntityMobGriefingEvent` | none | ❌ | C |  |
| `EntityMountEvent` | `entity-mount-event`, `entity-dismount-event` | ✅ | C |  |
| `EntityStruckByLightningEvent` | `lightning-strike-event` | 🟡 | C | Per world, not per entity |
| `EntityTeleportEvent` (+ `TeleportCommand`, `SpreadPlayersCommand`, `EnderEntity`, `EnderPearl`, `ItemConsumption`) | `entity-teleport-event`, `player-teleport-event` | ✅ | C |  |
| `EntityTravelToDimensionEvent` | `entity-portal-event` | ✅ | C |  |
| `ItemExpireEvent`, `ItemTossEvent` | `item-despawn-event`, `player-drop-item-event` | ✅ | C |  |
| `XpOrbTargetingEvent` | none | ❌ | C |  |
| `ProjectileImpactEvent` | `projectile-hit-event` | ✅ | B |  |
| `LivingIncomingDamageEvent`, `LivingDamageEvent.Pre/Post`, `ArmorHurtEvent` | `entity-damage-event` | 🟡 | A | No separate after-armor step |
| `LivingDeathEvent` | `entity-death-event` | 🟡 | A |  |
| `LivingDropsEvent`, `LivingExperienceDropEvent` | none | ❌ | A |  |
| `LivingHealEvent` | `entity-regain-health-event` | ✅ | C |  |
| `LivingKnockBackEvent` | `entity-knockback-event` (declared, not fired) | ❌ | C |  |
| `LivingFallEvent`, `PlayerFlyableFallEvent` | none | ❌ | C |  |
| `LivingEvent.LivingJumpEvent` | none | ❌ | C |  |
| `LivingEvent.LivingVisibilityEvent` | none | ❌ | C |  |
| `LivingBreatheEvent`, `LivingDrownEvent` | `entity-air-change-event` | 🟡 | C |  |
| `LivingChangeTargetEvent` | `entity-target-event` | ✅ | C |  |
| `LivingConversionEvent.Pre/Post` | `entity-transform-event` | ✅ | C |  |
| `LivingDestroyBlockEvent` | `entity-change-block-event` | ✅ | C |  |
| `LivingEntityUseItemEvent.Start/Tick/Stop/Finish` | `player-item-consume-event` | 🟡 | A | Finish only |
| `LivingEquipmentChangeEvent` | none | ❌ | B |  |
| `LivingGetProjectileEvent` | none | ❌ | C |  |
| `LivingShieldBlockEvent` | none | ❌ | C |  |
| `LivingSwapItemsEvent.Hands` | `player-swap-hands-event` | ✅ | C |  |
| `LivingUseTotemEvent` | `entity-resurrect-event` | ✅ | C |  |
| `MobEffectEvent.Added/Applicable/Remove/Expired` | `entity-potion-effect-event` | 🟡 | B |  |
| `MobSpawnEvent.SpawnPlacementCheck/PositionCheck`, `FinalizeSpawnEvent`, `SpawnClusterSizeEvent` | `creature-spawn-event` | 🟡 | B |  |
| `MobDespawnEvent` | none | ❌ | C |  |
| `MobSplitEvent` | `slime-split-event` (declared, not fired) | ❌ | C |  |
| `BabyEntitySpawnEvent` | `entity-breed-event` | ✅ | C |  |
| `AnimalTameEvent` | `entity-tame-event` | ✅ | C |  |
| `EndermanAngerEvent` | none | ❌ | C |  |
| `EffectParticleModificationEvent` | none | ❌ | C |  |
| `AttackEntityEvent` | `entity-damage-by-entity-event` | 🟡 | B |  |
| `CriticalHitEvent`, `SweepAttackEvent` | none | ❌ | C |  |
| `PlayerInteractEvent.RightClickBlock/RightClickItem/RightClickEmpty/LeftClickBlock/LeftClickEmpty/EntityInteract`, `UseItemOnBlockEvent` | `player-interact-event`, `player-interact-entity-event` | ✅ | A |  |
| `PlayerEvent.BreakSpeed`, `HarvestCheck` | none | ❌ | B |  |
| `PlayerEvent.Clone` | none | ❌ | B |  |
| `PlayerEvent.StartTracking/StopTracking` | `player-start/stop-tracking-event` | ✅ | B |  |
| `PlayerEvent.LoadFromFile/SaveToFile` | none | ❌ | C | Per-player mod data files |
| `PlayerEvent.ItemCraftedEvent/ItemSmeltedEvent` | `craft-item-event`, `furnace-extract-event` | ✅ | C |  |
| `PlayerEvent.PlayerLoggedIn/LoggedOut/Respawn/ChangedDimension/ChangeGameMode` | player events | ✅ | A |  |
| `PlayerEvent.NameFormat`, `TabListNameFormat` | none | ❌ | C |  |
| `PlayerContainerEvent.Open/Close` | `inventory-open/close-event` | ✅ | B |  |
| `PlayerDestroyItemEvent` | `player-item-break-event` | ✅ | C |  |
| `PlayerEnchantItemEvent`, `EnchantmentLevelSetEvent`, `GetEnchantmentLevelEvent` | `enchant-item-event`, `prepare-item-enchant-event` | 🟡 | C |  |
| `PlayerRespawnPositionEvent`, `PlayerSetSpawnEvent` | `player-spawn-change-event` | 🟡 | C |  |
| `PlayerSpawnPhantomsEvent` | none | ❌ | C |  |
| `PlayerSwitchHotbarSlotEvent.Pre/Post` | `player-item-held-event` | ✅ | C |  |
| `PlayerWakeUpEvent`, `CanPlayerSleepEvent`, `CanContinueSleepingEvent` | `player-bed-leave-event`, `player-sleep-check-event` | 🟡 | C |  |
| `PlayerXpEvent.PickupXp/XpChange/LevelChange` | `player-exp-change-event`, `player-level-change-event` | ✅ | C |  |
| `ArrowLooseEvent`, `ArrowNockEvent` | `entity-shoot-bow-event` | 🟡 | C |  |
| `BonemealEvent` | `block-fertilize-event` | ✅ | C |  |
| `ItemEntityPickupEvent.Pre/Post` | `entity-pickup-item-event` | ✅ | B |  |
| `ItemFishedEvent` | `player-fish-event` | ✅ | C |  |
| `TradeWithVillagerEvent` | `trade-select-event` | 🟡 | C |  |
| `AdvancementEvent.AdvancementEarnEvent/AdvancementProgressEvent` | `player-advancement-done-event` | 🟡 | C |  |
| `AnvilUpdateEvent`, `AnvilCraftEvent.Pre/Post` | `prepare-anvil-event` | 🟡 | C |  |
| `GrindstoneEvent.OnPlaceItem/OnTakeItem` | `prepare-grindstone-event` | 🟡 | C |  |
| `PotionBrewEvent.Pre/Post`, `PlayerBrewedPotionEvent` | `brew-event` | 🟡 | C |  |
| `ItemStackedOnOtherEvent` | `stacked-on-*` item hooks | 🟡 | C |  |
| `ItemAttributeModifierEvent` | none | ❌ | C |  |
| `EnchantedBlockLootEvent`, `EnchantedEntityLootEvent` | none | ❌ | C |  |
| `StatAwardEvent` | `player-statistic-increment-event` | ✅ | C |  |
| `CommandEvent`, `ServerChatEvent` | `server-command-event`, `player-chat-event` | ✅ | C |  |
| `DifficultyChangeEvent` | none | ❌ | C |  |
| `PlayLevelSoundEvent.AtEntity/AtPosition` | none | ❌ | C |  |
| `VanillaGameEvent` | `generic-game-event` | ✅ | C |  |
| `CustomClickActionEvent` | `dialog-click-action-event` | ✅ | C |  |
| `PermissionsChangedEvent`, `PermissionGatherEvent.Handler/Nodes` | `register-permission` | 🟡 | C |  |
| `PlayerNegotiationEvent`, `ClientInformationUpdatedEvent` | `player-locale-change-event` | 🟡 | C |  |
| `TagsUpdatedEvent.ServerDataLoad`, `OnDatapackSyncEvent`, `AddServerReloadListenersEvent`, `SortedReloadListenerEvent` | none | ❌ | B |  |
| `LootTableLoadEvent` | `loot-generate-event` (cancel only) | 🟡 | A |  |
| `RegisterCommandsEvent` | `context.register-command` | ✅ | A |  |
| `RegisterEvent`, `NewRegistryEvent`, `NewDatapackRegistryEvent`, `ModifyRegistriesEvent`, `RegisterDataMapTypesEvent`, `DataMapsUpdatedEvent` | mod data dump | 🟡 | B | Static content only |
| `EntityAttributeCreationEvent`, `EntityAttributeModificationEvent` | none | ❌ | A |  |
| `RegisterSpawnPlacementsEvent` | none | ❌ | B |  |
| `ModifyDefaultComponentsEvent`, `DefaultDataComponentsBoundEvent` | none | ❌ | B |  |
| `BlockEntityTypeAddBlocksEvent` | none | ❌ | C |  |
| `RegisterCapabilitiesEvent` | `world.set-item-storage` | 🟡 | A |  |
| `RegisterCauldronInteractionEvent`, `RegisterCauldronFluidContentEvent` | none | ❌ | C |  |
| `RegisterRecipePropertiesEvent` | none | ❌ | C |  |
| `RegisterStructureConversionsEvent`, `ExtendPoiTypesEvent` | none | ❌ | C |  |
| `RegisterGameRuleCategoryEvent` | none | ❌ | C |  |
| `RegisterPayloadHandlersEvent`, `RegisterConfigurationTasksEvent` | `player-custom-payload-event`, `register-configuration-payload` | 🟡 | B |  |
| `ModMismatchEvent`, `GameShuttingDownEvent`, `RegisterGameTestsEvent`, `GatherDataEvent`, `AddPackFindersEvent` | none | ➖ |  | Loader, data generation or test tooling |
| `RegisterRpcSchemaEvent` | none | ➖ |  | Management protocol |
| `BuildCreativeModeTabContentsEvent`, `AddAttributeTooltipsEvent`, `GatherSkippedAttributeTooltipsEvent`, `RegisterTooltipAppendersEvent`, `ItemTooltipEvent`, `FluidTooltipEvent`, `TagsUpdatedEvent.ClientPacketReceived` |  | ➖ |  | Client display |

#### Paper events Pumpkin doesn't declare

Paper (26.3, `paper-api`) has 465 event classes; Pumpkin's WIT declares the Bukkit set and some of Paper's. These 199 are not declared at all (abstract bases left out), all ❌, need C for content mods (they matter for server plugins more than for mods). Some have a Pumpkin equivalent under another name: `PlayerQuitEvent` is `player-leave-event`, `PlayerGameModeChangeEvent` is `player-gamemode-change-event`, `PlayerPickItemEvent` is `player-pick-item-block/entity-event`, `PlayerTrackEntityEvent` and `PlayerUntrackEntityEvent` are `player-start/stop-tracking-event`, `ChunkLoadEvent` is `chunk-load-event`.

| Group | Events |
|:--|:--|
| block | `AnvilDamagedEvent`, `BeaconActivatedEvent`, `BeaconDeactivatedEvent`, `BeaconEffectEvent`, `BellRevealRaiderEvent`, `BlockBreakBlockEvent`, `BlockBreakProgressUpdateEvent`, `BlockDestroyEvent`, `BlockFailedDispenseEvent`, `BlockLockCheckEvent`, `BlockPreDispenseEvent`, `CompostItemEvent`, `DragonEggFormEvent`, `PlayerShearBlockEvent`, `TargetHitEvent`, `VaultChangeStateEvent` |
| entity | `CreeperIgniteEvent`, `ElderGuardianAppearanceEvent`, `EnderDragonFireballHitEvent`, `EnderDragonFlameEvent`, `EnderDragonShootFireballEvent`, `EndermanAttackPlayerEvent`, `EndermanEscapeEvent`, `EntityAddToWorldEvent`, `EntityAttemptSmashAttackEvent`, `EntityAttemptSpinAttackEvent`, `EntityBreakByEntityEvent`, `EntityBreakEvent`, `EntityCollideWithEntityEvent`, `EntityCompostItemEvent`, `EntityConstructEvent`, `EntityCreatePortalEvent`, `EntityDamageItemEvent`, `EntityEffectTickEvent`, `EntityEquipmentChangedEvent`, `EntityFertilizeEggEvent`, `EntityIgniteEvent`, `EntityInsideBlockEvent`, `EntityJumpEvent`, `EntityLoadCrossbowEvent`, `EntityLungeEvent`, `EntityMoveEvent`, `EntityPathfindEvent`, `EntityPortalReadyEvent`, `EntityPushedByEntityAttackEvent`, `EntityRemoveFromWorldEvent`, `EntityTeleportEndGatewayEvent`, `EntityToggleSitEvent`, `EntityZapEvent`, `ExperienceOrbMergeEvent`, `FishHookStateChangeEvent`, `ItemTransportingEntityValidateTargetEvent`, `PhantomPreSpawnEvent`, `PlayerNaturallySpawnCreaturesEvent`, `PreCreatureSpawnEvent`, `PreSpawnerSpawnEvent`, `ProjectileCollideEvent`, `PufferFishStateChangeEvent`, `ShulkerDuplicateEvent`, `SkeletonHorseTrapEvent`, `SlimeChangeDirectionEvent`, `SlimePathfindEvent`, `SlimeSwimEvent`, `SlimeTargetLivingEntityEvent`, `SlimeWanderEvent`, `SulfurCubeSwallowItemEvent`, `TameableDeathMessageEvent`, `ThrownEggHatchEvent`, `TurtleGoHomeEvent`, `TurtleLayEggEvent`, `TurtleStartDiggingEvent`, `WaterBottleSplashEvent`, `WitchConsumePotionEvent`, `WitchReadyPotionEvent`, `WitchThrowPotionEvent` |
| player | `AsyncChatEvent`, `AsyncChatDecorateEvent`, `AsyncChatCommandDecorateEvent`, `AsyncPlayerSpawnLocationEvent`, `CartographyItemEvent`, `IllegalPacketEvent`, `PlayerAdvancementCriterionGrantEvent`, `PlayerArmSwingEvent`, `PlayerArmorChangeEvent`, `PlayerAttackEntityCooldownResetEvent`, `PlayerAttemptPickupItemEvent`, `PlayerBedFailEnterEvent`, `PlayerBucketFishEvent`, `PlayerChangeBeaconEffectEvent`, `PlayerClientLoadedWorldEvent`, `PlayerClientOptionsChangeEvent`, `PlayerConnectionCloseEvent`, `PlayerCustomClickEvent`, `PlayerDeepSleepEvent`, `PlayerFailMoveEvent`, `PlayerFlowerPotManipulateEvent`, `PlayerHandshakeEvent`, `PlayerInsertLecternBookEvent`, `PlayerInventorySlotChangeEvent`, `PlayerItemCooldownEvent`, `PlayerItemFrameChangeEvent`, `PlayerItemGroupCooldownEvent`, `PlayerJumpEvent`, `PlayerLaunchProjectileEvent`, `PlayerLecternPageChangeEvent`, `PlayerLoomPatternSelectEvent`, `PlayerMapFilledEvent`, `PlayerPickupExperienceEvent`, `PlayerPostRespawnEvent`, `PlayerPurchaseEvent`, `PlayerReadyArrowEvent`, `PlayerServerFullCheckEvent`, `PlayerSetSpawnEvent`, `PlayerShieldDisableEvent`, `PlayerSignCommandPreprocessEvent`, `PlayerStartSpectatingEntityEvent`, `PlayerStonecutterRecipeSelectEvent`, `PlayerStopSpectatingEntityEvent`, `PlayerStopUsingItemEvent`, `PlayerSwapWithEquipmentSlotEvent`, `PlayerTeleportEndGatewayEvent`, `PlayerToggleEntityAgeLockEvent`, `PlayerTradeEvent`, `PlayerUseUnknownEntityEvent`, `PrePlayerAttackEntityEvent` |
| connection and configuration | `AsyncPlayerConnectionConfigureEvent`, `PlayerCodeOfConductSendEvent`, `PlayerConnectionInitialConfigureEvent`, `PlayerConnectionReconfigureEvent`, `PlayerConnectionValidateLoginEvent`, `PlayerChunkLoadEvent`, `PlayerChunkUnloadEvent`, `ClientTickEndEvent`, `UncheckedSignChangeEvent` |
| server, world and commands | `AsyncPlayerSendCommandsEvent`, `AsyncPlayerSendSuggestionsEvent`, `CommandRegisteredEvent`, `UnknownCommandEvent`, `AsyncTabCompleteEvent`, `AsyncServerDataFixerRemoveBlockEntityEvent`, `GS4QueryEvent`, `ServerExceptionEvent`, `ServerResourcesReloadedEvent`, `WhitelistStateUpdateEvent`, `WhitelistToggleEvent`, `ClockTimeSkipEvent`, `StructuresLocateEvent`, `WorldDifficultyChangeEvent`, `WorldGameRuleChangeEvent`, `WorldBorderBoundsChangeEvent`, `WorldBorderBoundsChangeFinishEvent`, `WorldBorderCenterChangeEvent`, `ItemCraftedEvent` |
| profiles and registries | `FillProfileEvent`, `LookupProfileEvent`, `PreFillProfileEvent`, `PreLookupProfileEvent`, `ProfileWhitelistVerifyEvent`, `RegistryComposeEvent`, `RegistryEntryAddEvent` |

### Upstream events declared but never fired

A plugin can register for these, but nothing fires them yet: `area-effect-cloud-apply`,
`arrow-body-count-change`, `async-player-chat`, `bat-toggle-sleep`, `bell-resonate`,
`block-dispense-armor`, `block-dispense-loot`, `block-fade`, `block-multi-place`,
`block-receive-game`, `block-shear-entity`, `creeper-power`, `dialog-clear`, `dialog-show`,
`ender-dragon-change-phase`, `entity-block-form`, `entity-combust-by-block`,
`entity-combust-by-entity`, `entity-knockback`, `entity-knockback-by-entity`,
`entity-portal-enter`, `entity-portal-exit`, `entity-remove`, `entity-spell-cast`,
`entity-target-block`, `entity-target-living-entity`, `exp-bottle`, `fluid-level-change`,
`hanging-break`, `hanging-break-by-entity`, `hanging-place`, `horse-jump`,
`inventory-block-start`, `map-initialize`, `pig-zap`, `pig-zombie-anger`,
`player-armor-stand-manipulate`, `player-command-preprocess`, `player-exp-cooldown-change`,
`player-hide-entity`, `player-name-entity`, `player-portal`, `player-pre-login`,
`player-show-entity`, `player-take-lectern-book`, `sculk-bloom`, `sheep-dye-wool`,
`sheep-regrow-wool`, `slime-split`, `strider-temperature-change`, `vault-display-item`,
`villager-acquire-trade`, `villager-career-change`, `villager-replenish-trade`,
`villager-reputation-change`, `warden-anger-change` (each with the `-event` suffix). These are
upstream's to fire; listed so a mod port doesn't count on them.

### Gaps by need

Implementation order for `ROADMAP.md` item 6, most needed first. Items in one line ship together.
Every ❌ and 🟡 row in the tables above falls in one of these batches.

1. **A, items and drops:** item use over time for plugin items (`finish-using`, `release-using`,
   use ticks, stop using); a drops event for any block (`BlockDropsEvent`) and for entities
   (`LivingDropsEvent`, experience), and killer and damage source in the death event.
2. **A, menus and storage:** menu data slots (progress bars) and menu buttons
   (`clickMenuButton`); sided item storage with transactions, a fluid storage, and a generic
   lookup so plugins can expose and find each other's storages (energy included).
3. **A, data and loot:** loot table changes (add pools or entries by id); entity and player
   attachments that copy on respawn (`COPY_FROM`, `PlayerEvent.Clone`); per-player data files.
4. **A, world generation:** biome feature additions (ores, plants) and spawn entries.
5. **A, entities:** modded entity types with default attributes, custom synced entity data,
   projectiles and AI (large: entity registry overlay, spawning, tracking).
6. **B, ticks and lifecycle:** per-world and per-player tick events; datapack reload, tags
   loaded and data sync events, and reload listeners for mod data; separate server
   starting/started/stopped events; equipment change event; mob effect add/remove events.
7. **B, block and item hooks:** `on-place` (any placement), `can-survive`, destroy speed and
   harvest check, projectile hit, after-break drops (`spawnAfterBreak`), explosion hooks;
   `use-first` (`onItemUseFirst`), hit and mine hooks, left click on entity, use on entity, item
   abilities (`canPerformAction`); hooks for vanilla blocks next to their own behaviour.
8. **B, content registries:** default component changes for vanilla items; strip, till,
   flatten and wax; fuel and brewing; modded mob effects, particles and fluids; fake players.
9. **C:** everything else marked C (fall and bounce, ladders, pistons, light and friction per
   position, path types, data maps, compost and oxidation, modded game rules and argument
   types, sleep details, knockback, critical hits, the Paper events Pumpkin doesn't declare,
   and upstream's never-fired events).
