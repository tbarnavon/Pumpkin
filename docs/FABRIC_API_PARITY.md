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

API, where the feature belongs:
- **core**: Pumpkin's normal plugin API. Useful on a vanilla server too, so it can go upstream.
- **modded**: `modded.wit`. Only makes sense with mod content or Fabric clients.
- **host**: server internals (`pumpkin-fabric`, the dump, registries). No plugin API needed.
- **core + modded**: a general event in core, plus a per-block or per-item hook in modded.

**SD** marks what Storage Drawers 26.3.0.1 needs.

## What `modded.wit` has today

Only the hooks are modded. The rest works on vanilla blocks and items too, and belongs in core:

| `modded.wit` function | API | Belongs in |
|:--|:--|:--|
| `register-block-hooks`, `register-item-hooks`, block and item calls | modded | `modded` |
| `get/set/remove-block-entity-data` (full block entity NBT) | core | `block-entity` or `world` |
| `get/set/remove-item-component` (by component id) | core | `item-stack` |
| `item-to-nbt`, `item-from-nbt` | core | `item-stack` |
| `give-item` (insert into inventory, drop the rest) | core | `player` |
| `drop-item` (item entity at a block) | core | `world` |
| `schedule-block-tick` | core | `world` |

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
| `ServerConfigurationNetworking`, configuration tasks | ❌ | core | no plugin hook in the configuration phase | |
| `ServerLoginNetworking`, login queries | 🟡 | core | `player-pre-login-event`, `player-login-event`; no login query/response | |
| `EntityTrackingEvents` START / STOP_TRACKING | ❌ | core | | |
| `PlayerLookup.tracking(...)` (players watching a chunk, block entity or entity) | ❌ | core | only all players or players in a world | ✅ `count_update` |

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
| `PlayerBlockBreakEvents` BEFORE / AFTER / CANCELED | 🟡 | core | `block-break-event` (before, cancellable); no after/canceled | |
| `PlayerPickItemEvents` (middle click block/entity) | ❌ | core | | |
| `FakePlayer` | ❌ | core | | |

### fabric-lifecycle-events-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| `ServerTickEvents` START / END_SERVER_TICK | ✅ | core | `server-tick-start-event`, `server-tick-end-event` | |
| `ServerTickEvents` per level | ❌ | core | | |
| `ServerLifecycleEvents` SERVER_STARTED | ✅ | core | `server-load-event`; plugin `on-load` | |
| `ServerLifecycleEvents` STOPPING / STOPPED, datapack reload | 🟡 | core | plugin `on-unload`; no reload event | |
| `ServerLevelEvents` LOAD / UNLOAD | ✅ | core | `world-load-event`, `world-unload-event` | |
| `ServerChunkEvents` LOAD / UNLOAD | ✅ | core | `chunk-load-event`, `chunk-unload-event` | |
| `ServerEntityEvents` LOAD / UNLOAD | ✅ | core | `entity-spawn-event`, `entities-load-event`, `entity-remove-event` | |
| `ServerBlockEntityEvents` LOAD / UNLOAD | ❌ | core | | |
| `CommonLifecycleEvents` TAGS_LOADED | ➖ | - | tags are fixed after load | |

### fabric-entity-events-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| `ServerLivingEntityEvents` ALLOW_DAMAGE / AFTER_DAMAGE / ALLOW_DEATH / AFTER_DEATH | ✅ | core | `entity-damage-event`, `entity-death-event`, `entity-resurrect-event` | |
| `ServerPlayerEvents` JOIN / LEAVE / AFTER_RESPAWN / COPY_FROM | 🟡 | core | join, leave, respawn events; no COPY_FROM (data kept across respawn) | |
| `ServerEntityCombatEvents` AFTER_KILLED_OTHER_ENTITY | 🟡 | core | `entity-death-event` has the killer | |
| `ServerEntityLevelChangeEvents` | ✅ | core | `player-change-world-event`, `entity-portal-event` | |
| `EntitySleepEvents` | 🟡 | core | bed enter/leave; no ALLOW_SLEEP_TIME, bed direction or respawn-anchor hooks | |
| `EntityElytraEvents` ALLOW / CUSTOM | 🟡 | core | `entity-toggle-glide-event`; no custom elytra | |
| `ServerMobEffectEvents` | ✅ | core | `entity-potion-effect-event` | |

### fabric-item-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Item `useOn` / `use` for modded items | ✅ | modded | modded item hooks | ✅ |
| Read / write item components by id | ✅ | core | `modded.get/set/remove-item-component` | ✅ |
| Keep unknown modded components (e.g. `storagedrawers:drawer_count`) | ❌ | host | `ItemStack` drops them; needs a raw-NBT fallback | ✅ GUI, drops |
| `DefaultItemComponentEvents` | ➖ | - | default components come from the dump | |
| `FabricItem` (recipe remainder, attribute modifiers, reequip animation) | ❌ | modded | | |
| `CustomDamageHandler` | ❌ | modded | | |
| `EquipmentSlotProvider` | ❌ | modded | | |
| `EnchantmentEvents` ALLOW_ENCHANTING / MODIFY | 🟡 | core | `prepare-item-enchant-event`, `enchant-item-event`; no per-item allow | |
| `ItemClickBehaviorCallback` (click one stack onto another in a menu) | ❌ | core | `inventory-click-event` only | |
| `BlockTransformerEvents` (strip, till, flatten...) | ❌ | core | | |
| `ItemComponentTooltipProviderRegistry` | ➖ | - | tooltips are drawn by the client | |

### fabric-menu-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Vanilla menu types | ✅ | core | `gui` resource, `player.open-gui` | |
| `ExtendedMenuType` / `ExtendedMenuProvider` (modded menu, `fabric-menu-api-v1:open_screen` with extra data) | ❌ | modded | | ✅ drawer GUI |
| Modded menu slot layout and quick-move rules | ❌ | core | | ✅ drawer GUI |

### fabric-object-builder-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| `FabricBlockEntityTypeBuilder` | ✅ | host | types from the dump; `modded.set-block-entity-data` | ✅ |
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
| Data on chunks and levels | ❌ | core | | |
| `AttachmentSyncPredicate` (sync to clients, `fabric:attachment_sync`) | ❌ | modded | | |

### fabric-lookup-api-v1 and fabric-transfer-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| `BlockApiLookup` / `ItemApiLookup` / `EntityApiLookup` | ❌ | modded | | ✅ controller, hoppers |
| `ItemStorage.SIDED`, `Storage<ItemVariant>`, transactions | ❌ | modded | | ✅ hoppers into drawers |
| `FluidStorage` | ❌ | modded | | |

Vanilla hoppers moving items into a plugin block also need the host to ask the plugin for its
inventory.

### fabric-loot-api-v3
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Mod loot tables | ✅ | host | imported from the dump | ✅ |
| Custom drops from code (`getDrops`) | ✅ | modded | modded block `drops` hook | ✅ |
| `LootTableEvents` REPLACE / MODIFY / MODIFY_DROPS | 🟡 | core | `loot-generate-event`; can't change tables | |

### fabric-recipe-api-v1
| Feature | Status | API | Pumpkin | SD |
|:--|:--|:--|:--|:--|
| Mod recipes | 🟡 | host | imported from the dump; 5 recipes of custom serializer types are skipped | ✅ |
| Register shaped / shapeless / cooking recipes from code | ✅ | core | `recipe.register-*` | |
| `CustomIngredient` (`fabric:all_of`, `any_of`, `components`, `difference`) | ❌ | host | | |
| Look up recipes from code (`RecipeManager`) | ❌ | core | | ✅ compacting drawers |
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
| Flammable, oxidizable, strippable, compostable, fuel, villager interactions, path types, vibration frequencies | ❌ | core | vanilla values are hardcoded | |
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
| `ServerMessageDecoratorEvent` | ❌ | core | | |

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
| `fabric-block-getter-api-v2` | ✅ | modded | render data is block entity data, sent by `set-block-entity-data` |
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
| `Block.tick` (scheduled) | ✅ | modded | `scheduled-tick`, `schedule-block-tick` | ✅ |
| `Block.getSignal` / `getDirectSignal` / `isSignalSource` | ❌ | modded | | ✅ redstone upgrade |
| `Block.getAnalogOutputSignal` (comparator) | ❌ | modded | | ✅ |
| `Block.neighborChanged`, `updateShape` | ❌ | modded | | |
| `Block.randomTick` | ❌ | modded | | |
| `Block.entityInside`, `stepOn` | ❌ | modded | | |
| `BlockEntity` ticker | ❌ | modded | only scheduled ticks | ✅ hopper/magnet upgrades |
| `Item.useOn` / `use` | ✅ | modded | item hooks | ✅ |
| `Item.inventoryTick` | ❌ | modded | | |
| Find entities in an area | 🟡 | core | `entity.get-nearby-entities` (around an entity); no box query on a world | ✅ magnet |
