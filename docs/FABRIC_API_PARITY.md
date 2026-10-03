# Fabric API parity

What a Fabric mod's server-side code can use, and whether a Pumpkin WASM plugin can do the same.
Checked against Fabric API 0.161.0+26.3 (`refs/fabric-api`) and the WIT in
`crates/pumpkin-plugin-wit/v0.1`. Something counts as done when a plugin can do it, whether the
host API is in `modded.wit` or was already in Pumpkin's plugin API.

Status:
- ✅ done: a plugin can do this today.
- 🟡 partial: some of it works; the note says what's missing.
- ❌ missing.
- ➖ not needed: client-only, dev-time only, or resolved when the Extractor dumps the mod's data.

Summary (110 rows in the tables below, not counting the 13 marked ➖):

| Done ✅ | Partial 🟡 | Missing ❌ |
|:--|:--|:--|
| 72 (65 %) | 15 (14 %) | 23 (21 %) |

API, where the feature belongs:
- **core**: Pumpkin's normal plugin API. Useful on a vanilla server too, so it can go upstream.
- **modded**: `modded.wit`. Only makes sense with mod content or Fabric clients.
- **host**: server internals (`pumpkin-fabric`, the dump, registries). No plugin API needed.
- **core + modded**: a general event in core, plus a per-block or per-item hook in modded.

**SD** marks what Storage Drawers 26.3.0.1 needs.

## What `modded.wit` has

Only what needs mod content stays in `modded.wit`: the block and item hooks, and modded menu
types. The rest moved to core; block-entity data works the same for vanilla block entities and
for a mod's, whose data only lives in the chunk as NBT:

| Function | Now in |
|:--|:--|
| `register-block-hooks`, `register-item-hooks`, block and item calls | `modded` |
| `register-component-stream-codec` | `modded` |
| `get/set-block-entity-data`, `remove-block-entity` | `world` |
| `get/set/remove-component-by-id` | `item-stack` |
| `to-nbt`, `from-nbt` | `item-stack` |
| `give-item` | `player` |
| `drop-item` | `world` |
| `schedule-block-tick` | `world` |

## Fabric API modules

### fabric-api-base
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| `Event`, `EventFactory`, `TriState`, `EventResult` | ➖ | - | Pumpkin's own event system (`register-event`) | |

### fabric-networking-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Fabric handshake, `c:register` / `minecraft:register` channels | ✅ | host | `pumpkin-fabric` | ✅ |
| `ServerPlayNetworking.send` (custom payload to a player) | ✅ | core | `player.send-custom-payload` | ✅ |
| `ServerPlayNetworking.registerGlobalReceiver` | ✅ | core | `player-custom-payload-event` | |
| `PayloadTypeRegistry` (payload codecs) | ➖ | - | the plugin encodes its own bytes | |
| `ServerPlayConnectionEvents` INIT / JOIN / DISCONNECT | ✅ | core | `player-join-event`, `player-leave-event` | |
| `ServerConfigurationNetworking`, configuration tasks | 🟡 | core | data first: `context.register-configuration-payload` is sent to every client in configuration, replies fire `player-configuration-payload-event` (cancel to disconnect); no tasks that hold configuration until a reply | |
| `ServerLoginNetworking`, login queries | ✅ | core | `player-pre-login-event`, `player-login-event`; data first: `context.register-login-query(channel, payload)` is sent to every client before login success, answers in `player-login-query-response-event` (cancel to disconnect) | |
| `EntityTrackingEvents` START / STOP_TRACKING | ✅ | core | `player-start-tracking-event`, `player-stop-tracking-event` | |
| `PlayerLookup.tracking(...)` (players watching a chunk, block entity or entity) | ✅ | core | `server.get-players-tracking-chunk`, `get-players-tracking-entity` | ✅ `count_update` |

### fabric-registry-sync-v0
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Static registry sync (`fabric:registry/sync`) | ✅ | host | `pumpkin-fabric`, byte identical | ✅ |
| Modded blocks, items, block entity types from the dump | ✅ | host | `pumpkin-registry-ext` | ✅ |
| `DynamicRegistries` (modded synced dynamic registries) | ❌ | host | not checked yet | |
| `RegistryEntryAddedCallback`, `RegistryIdRemapCallback` | ➖ | - | registries are fixed after load | |

### fabric-events-interaction-v0
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| `UseBlockCallback` | ✅ | core + modded | `player-interact-event`; per block: modded block `use` hook | ✅ |
| `AttackBlockCallback` | ✅ | core + modded | `player-interact-event`, `block-damage-event`; per block: `attack` hook | ✅ |
| `UseItemCallback` | ✅ | core + modded | `player-interact-event`; per item: modded item `use` hook | ✅ |
| `UseEntityCallback`, `AttackEntityCallback` | ✅ | core | `player-interact-entity-event`, `entity-damage-by-entity-event` | |
| `PlayerBlockBreakEvents` BEFORE / AFTER / CANCELED | ✅ | core | `block-break-event` (before, cancellable), `block-broken-event` (after), `block-break-canceled-event` | |
| `PlayerPickItemEvents` (middle click block/entity) | ✅ | core | `player-pick-item-block-event`, `player-pick-item-entity-event`: set `item` to replace the pick, cancel for none | |
| `FakePlayer` | ❌ | core | | |

### fabric-lifecycle-events-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| `ServerTickEvents` START / END_SERVER_TICK | ✅ | core | `server-tick-start-event`, `server-tick-end-event` | |
| `ServerTickEvents` per level | 🟡 | core | `server-tick-start-event` / `server-tick-end-event` cover all worlds; no per-world event (data first: prefer scheduled ticks) | |
| `ServerLifecycleEvents` SERVER_STARTED | ✅ | core | `server-load-event`; plugin `on-load` | |
| `ServerLifecycleEvents` STOPPING / STOPPED, datapack reload | 🟡 | core | `server-stopping-event`; plugin `on-unload` is the last call (plugins unload before worlds save); no reload event | |
| `ServerLevelEvents` LOAD / UNLOAD | ✅ | core | `world-load-event`, `world-unload-event` | |
| `ServerChunkEvents` LOAD / UNLOAD | ✅ | core | `chunk-load-event`, `chunk-unload-event` | |
| `ServerEntityEvents` LOAD / UNLOAD | ✅ | core | `entity-spawn-event`, `entities-load-event`, `entity-remove-event` | |
| `ServerBlockEntityEvents` LOAD / UNLOAD | 🟡 | core | `block-entity-load-event`, `block-entity-unload-event` for block entities Pumpkin implements; they load the first time they are used, not with the chunk. A mod's block entities (chunk NBT only) fire the load event once each time their chunk loads and ticks | ✅ drawers and controllers (`onEntityLoad`) |
| `CommonLifecycleEvents` TAGS_LOADED | ➖ | - | tags are fixed after load | |

### fabric-entity-events-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| `ServerLivingEntityEvents` ALLOW_DAMAGE / AFTER_DAMAGE / ALLOW_DEATH / AFTER_DEATH | ✅ | core | `entity-damage-event`, `entity-death-event`, `entity-resurrect-event` | |
| `ServerPlayerEvents` JOIN / LEAVE / AFTER_RESPAWN / COPY_FROM | ✅ | core | join, leave, respawn events; COPY_FROM is not needed: Pumpkin keeps the same player (and its custom data) across respawn | |
| `ServerEntityCombatEvents` AFTER_KILLED_OTHER_ENTITY | 🟡 | core | `entity-death-event` has the killer | |
| `ServerEntityLevelChangeEvents` | ✅ | core | `player-change-world-event`, `entity-portal-event` | |
| `EntitySleepEvents` | 🟡 | core | bed enter/leave; `player-sleep-check-event` (ALLOW_SLEEP_TIME, ALLOW_NEARBY_MONSTERS); no bed direction, wake-up position or respawn-anchor hooks | |
| `EntityElytraEvents` ALLOW / CUSTOM | 🟡 | core | `entity-toggle-glide-event`; no custom elytra | |
| `ServerMobEffectEvents` | ✅ | core | `entity-potion-effect-event` | |

### fabric-item-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Item `useOn` / `use` for modded items | ✅ | modded | modded item hooks | ✅ |
| Read / write item components by id | ✅ | core | `item-stack.get/set/remove-component-by-id` | ✅ |
| Keep unknown modded components (e.g. `storagedrawers:drawer_count`) | ✅ | host + modded | `ItemStack.unknown_patch`: raw NBT on disk; on the network as network NBT (vanilla's default stream codec), or in the mod's own format for types with `networkSynchronized(...)`, from a stream codec description the plugin gives once with `modded.register-component-stream-codec` (composites of primitives, item stacks, lists, optionals) | ✅ GUI, drops, `frame_data`, `controller_binding` |
| `DefaultItemComponentEvents` | ➖ | - | default components come from the dump | |
| `FabricItem` (recipe remainder, attribute modifiers, reequip animation) | ❌ | modded | | |
| `CustomDamageHandler` | ❌ | modded | | |
| `EquipmentSlotProvider` | ❌ | modded | | |
| `EnchantmentEvents` ALLOW_ENCHANTING / MODIFY | 🟡 | core | `prepare-item-enchant-event`, `enchant-item-event`; no per-item allow | |
| `ItemClickBehaviorCallback` / `Item.overrideOtherStackedOnMe` / `overrideStackedOnOther` (click one stack onto another in a menu) | ✅ | modded | `stacked-on-me` and `stacked-on-other` item hooks, called on pickup clicks in any menu (`tryItemClickBehaviourOverride`, carried item first); the reply sets the slot and cursor stacks | ✅ keyring: add or take out keys in the inventory |
| `Item.onDestroyed` (the item entity of a stack is destroyed) | ✅ | modded | `destroyed` item hook, with the item entity, when its health drops to 0 (fire, lava, explosions, cactus) | ✅ keyring: keys spill out when a keyring burns |
| `Item.canFitInsideContainerItems` (bundles, shulker boxes) | ✅ | modded | `not-in-containers` item hook flag, held by the host (no call); bundles and shulker box menus (now `ShulkerBoxSlot`, as vanilla) refuse such items and shulker boxes | ✅ `canStoreInContainers` for filled and detached drawers |
| Item tags of a stack (`ItemStack.is(TagKey)`, `getTags`) | ✅ | core | `item-stack.get-tags`, `item-stack.has-tag`, with the entries mods add | ✅ conversion upgrade tag allow and deny lists |
| `BlockTransformerEvents` (strip, till, flatten...) | ❌ | core | | |
| `ItemComponentTooltipProviderRegistry` | ➖ | - | tooltips are drawn by the client | |

### fabric-menu-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Vanilla menu types | ✅ | core | `gui` resource, `player.open-gui` | |
| `ExtendedMenuType` / `ExtendedMenuProvider` (modded menu, `fabric-menu-api-v1:open_screen` with extra data) | ✅ | modded | `modded.open-menu` | ✅ drawer GUI |
| Plugin-defined slot layout and quick-move rules | ✅ | core | `menu` interface, `handle-menu-call` export | ✅ drawer GUI |
| Menu contents and `stillValid` without per-tick calls | ✅ | core | host keeps the slots (`menu-definition.contents`, `menu.update-slot`), checks a `menu-anchor` itself, `menu.close`; the plugin is only called on clicks and close | ✅ drawer GUI |

### fabric-object-builder-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| `FabricBlockEntityTypeBuilder` | ✅ | host + modded | types from the dump; data through `world.set-block-entity-data` | ✅ |
| `WoodTypeBuilder`, `BlockSetTypeBuilder` | ➖ | - | resolved in the dump | |
| `FabricEntityType`, modded entities | ❌ | modded | | detached drawers? |
| `FabricDefaultAttributeRegistry` | ❌ | modded | | |
| `FabricEntityDataRegistry` (custom synced entity data) | ❌ | modded | | |
| `PoiHelper` | ❌ | modded | | |
| `MinecartComparatorLogicRegistry` | ❌ | modded | | |

### fabric-data-attachment-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Persistent data on entities and block entities | 🟡 | core | `set-custom-data` on entities and block entities; not Fabric's NBT layout | |
| Data on chunks and levels | ✅ | core | `chunk.set-custom-data`, `world.set-custom-data` | |
| `AttachmentSyncPredicate` (sync to clients, `fabric:attachment_sync`) | ❌ | modded | | |

### fabric-lookup-api-v1 and fabric-transfer-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| `BlockApiLookup` / `ItemApiLookup` / `EntityApiLookup` | ❌ | modded | | ➖ SD's capabilities are read inside the plugin |
| `ItemStorage.SIDED`, `Storage<ItemVariant>` for hoppers | 🟡 | core | data first: `world.set-item-storage` puts host-held slots on a plugin block entity (count, capacity in stacks, insert/extract, accept-new, keep-item, void, slots sharing one pool at a per-slot rate); hoppers use them without calling the plugin; `item-storage-changed-event` once per tick. No sides, no transactions, no plugin-to-plugin transfer | ✅ drawers, compacting drawers, controller and controller I/O (the network's slots mirrored) |
| `FluidStorage` | ❌ | modded | | |

Hoppers see a storage slot as at most one stack, one item short of full while there is room,
so they keep inserting into drawers holding thousands of items. An item that stacks to 1 shows
as a full slot while any is stored, so hoppers only take it out.

### fabric-loot-api-v3
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Mod loot tables | ✅ | host | imported from the dump | ✅ |
| Custom drops from code (`getDrops`) | ✅ | modded | modded block `drops` hook | ✅ |
| `LootTableEvents` REPLACE / MODIFY / MODIFY_DROPS | 🟡 | core | `loot-generate-event`; block drops can be replaced in `block-drop-item-event`; tables themselves can't be changed | |

### fabric-recipe-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Mod recipes | ✅ | host + core | imported from the dump; recipes of custom serializer types (a mod's `CustomRecipe` code) are skipped and crafted by a plugin with `context.register-crafting-handler` | ✅ add_upgrade, add_detached_upgrade, keyring, personal_key_cycle, remote_group_upgrade |
| Register shaped / shapeless / cooking recipes from code | ✅ | core | `recipe.register-*` | |
| `CustomIngredient` (`fabric:all_of`, `any_of`, `components`, `difference`) | 🟡 | host | all four parse and match in mod and datapack recipes; `components` checks only its base item, as recipes match items, not stacks | |
| Look up recipes from code (`RecipeManager`) | ✅ | core | `recipe-manager.match-crafting` (vanilla, datapack, mod and plugin recipes), `match-cooking` (vanilla recipes, as furnaces do), `crafting-recipes-for` (crafting recipes by result, with shape and ingredients, tags resolved) | ✅ compacting drawers (`findLowerTier` over shaped recipes) |
| `RecipeSynchronization` (send recipes to the client) | ❌ | modded | | |

### fabric-command-api-v2
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| `CommandRegistrationCallback` | ✅ | core | `register-command` | |
| `ArgumentTypeRegistry` (modded argument types) | ❌ | modded | | |
| `EntitySelectorOptionRegistry` | ❌ | core | | |

### fabric-content-registries-v0
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Flammable, oxidizable, strippable, compostable, fuel, villager interactions, path types, vibration frequencies | 🟡 | core | flammable: `server.set-flammable`; fuel and compostable are item components (a mod's come with the dump, per stack with `item-stack.set-component-by-id`); the rest are hardcoded | |
| `FluidBehavior`, `EntityFluidInteractionRegistry` | ❌ | modded | | |

### fabric-game-rule-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Read and set vanilla game rules | ✅ | core | `world.get-game-rule`, `set-game-rule` | |
| `GameRuleBuilder` (modded game rules), `GameRuleEvents` | ❌ | core | | |

### fabric-permission-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Permission nodes and checks | ✅ | core | `register-permission`, `player.has-permission` | |
| `PermissionEvents` | ✅ | core | `player-permission-check-event` | |

### fabric-message-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| `ServerMessageEvents` ALLOW / CHAT / COMMAND / GAME | ✅ | core | `player-chat-event`, `player-command-preprocess-event`, `server-broadcast-event` | |
| `ServerMessageDecoratorEvent` | ✅ | core | `async-player-chat-event` changes the message and its `format` | |

### fabric-advancement-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Mod advancements | 🟡 | host | loaded with the datapack if dumped; not checked | |
| Grant / revoke advancements, `AdvancementEvents` | ✅ | core | `player.award-advancement`, `revoke-advancement`, `player-advancement-done-event` | |

### fabric-biome-api-v1 and fabric-dimensions-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| `BiomeModifications` (add features and spawns) | ❌ | core | | |
| `NetherBiomes`, `TheEndBiomes` | ❌ | core | | |
| `DimensionEvents`, teleport across dimensions | ✅ | core | `teleport-world`, `entity-portal-event` | |

### fabric-particles-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Spawn vanilla particles | ✅ | core | `player.spawn-particles` | |
| Spawn modded particle types | ❌ | modded | `particle` is a vanilla-only enum | |

### Data-only and client-only modules
| Module | Status | API | Why |
|:--|:--|:--|:--|
| `fabric-convention-tags-v2`, `fabric-tag-api-v1` | ✅ | host | every non-`minecraft` tag is in the dump |
| `fabric-resource-conditions-api-v1` | ➖ | host | conditions are resolved when the dump runs |
| `fabric-resource-loader-v1` | 🟡 | host | mod data comes from the dump; no reload listeners |
| `fabric-creative-tab-api-v1` | ➖ | - | client side; the registry is synced |
| `fabric-block-api-v1` | ➖ | - | block appearance is client side |
| `fabric-block-getter-api-v2` | ✅ | modded | render data is block entity data, sent by `world.set-block-entity-data` |
| `fabric-serialization-api-v1` | ➖ | - | Java codec helpers |
| `fabric-data-generation-api-v1`, `fabric-gametest-api-v1`, `fabric-debug-api-v1` | ➖ | - | dev time |
| rendering, model loading, screen, key mapping, sound, client gametest | ➖ | - | client only |

## Vanilla overrides mods rely on

Most mod server logic overrides vanilla `Block` and `Item` methods rather than using Fabric API.
These are the `modded.wit` hooks: behaviour for blocks and items a plugin defines. Vanilla blocks
keep their native behaviour.

| Vanilla method | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| `Block.getStateForPlacement` | ✅ | modded | `placement-state` | ✅ |
| `Block.setPlacedBy` | ✅ | modded | `placed` | ✅ |
| `Block.useWithoutItem` / `useItemOn` | ✅ | modded | `use` | ✅ |
| `Block.attack` | ✅ | modded | `attack` | ✅ |
| `Block.getDrops` | ✅ | modded | `drops` | ✅ |
| `Block.affectNeighborsAfterRemoval` / `onRemove` | ✅ | modded | `removed` | ✅ |
| `Block.tick` (scheduled) | ✅ | modded | `scheduled-tick` hook; scheduled with `world.schedule-block-tick` | ✅ |
| `Block.getSignal` / `getDirectSignal` / `isSignalSource` | ✅ | modded | data first: `signal-source` hook flag + `world.set-redstone-output(weak, strong)` or `set-redstone-output-sides` (per side), held by the host | ✅ redstone upgrade (with `analogOutput` off): weak on every side, strong to the block below |
| `Block.getAnalogOutputSignal` (comparator) | ✅ | modded | data first: `analog-output` hook flag + `world.set-comparator-output`, held by the host | ✅ |
| `Block.neighborChanged` | ✅ | modded | `neighbor-changed` hook (opt-in per block) | |
| `Block.updateShape` | ✅ | modded | `update-shape` hook (opt-in per block), replies with the new state | ✅ key buttons |
| `Block.randomTick` | ✅ | modded | `random-tick` hook, only for states the mod marks as randomly ticking (from the dump) | |
| `Block.entityInside`, `stepOn` | ✅ | modded | opt-in `entity-inside` and `step-on` hooks: called every tick, only for blocks registered with them | |
| `Block.getShape` / `getCollisionShape` that depends on the block entity | ✅ | modded | data first: `modded.set-block-collision-shape` per position, held and saved by the host, used for entity movement | ✅ hopper upgrade (the hole in the drawer top) |
| `BlockEntity` ticker | ✅ | modded | opt-in `ticker` block hook: every tick for each block entity of the block in ticking chunks; worlds skip the scan until a plugin registers one. Prefer `world.schedule-block-tick` and host-held data where they fit | ➖ SD uses scheduled ticks |
| `Item.useOn` / `use` | ✅ | modded | item hooks | ✅ |
| `Item.inventoryTick` | ✅ | modded | opt-in `inventory-tick` item hook: called every tick for each stack in a player's inventory, only for items registered with it | ✅ bound remote upgrades, heavy drawers (`PlayerEventListener`) |
| Find entities in an area | ✅ | core | `world.get-entities-in-box`, `entity.get-nearby-entities` | |
| Read and change an item entity's stack (`ItemEntity.getItem` / `setItem`) | ✅ | core | `entity.get-item-stack`, `entity.set-item-stack` (none or empty removes the entity), on entities from `world.get-entities-in-box` | ✅ hopper and magnet upgrades (`BlockEntityDrawers.addItemEntity`) |

## Open items before an upstream PR

- A packet comparison of `fabric-menu-api-v1:open_screen` against a real Fabric server.
