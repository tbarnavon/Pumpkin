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

Upstream declares about 290 Bukkit-style events; 56 of them are never fired by the server yet
(below). Fired ones are counted as ✅.

### Content and registries

| What mods do | Fabric API | NeoForge | Pumpkin | Status | Need | Notes |
|:--|:--|:--|:--|:--|:--|:--|
| Blocks, items, block-entity types, tags, components | `Registry.register` | `DeferredRegister` | Mod data dump, runtime overlay | ✅ | A |  |
| Menus | `ExtendedMenuType` | `IMenuTypeExtension` | `menu`, `modded.open-menu` | ✅ | A |  |
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

1. **A:** item use over time (`finish-using`, `release-using`, use duration) for plugin items;
   death and kill details (killer, drops, xp) in an event; sided item storage with
   transactions, and a fluid storage next to it.
2. **A:** loot table changes (add pools or entries to a table by id, like `LootTableEvents.MODIFY`);
   biome feature additions (ores and plants placed by the generator).
3. **A:** modded entity types with default attributes and custom synced entity data (large:
   needs the entity registry overlay, spawning, tracking and AI).
4. **B:** per-world tick events; datapack reload and tags loaded events; equipment change
   event; copy data on respawn; `can-survive` block hook; destroy speed and harvest check;
   projectile hit on a plugin block; item hit and mine hooks; use on entity for plugin items.
5. **B:** fuel, strip/till/flatten/wax and brewing; modded mob effects and particles;
   modded spawn entries; hooks for vanilla blocks next to their own behaviour.
6. **C:** the rest of the C rows (fall on, ladder, piston reaction, light emission from data,
   oxidation and path-type registries, modded game rules, sleep details, knockback and fall).
