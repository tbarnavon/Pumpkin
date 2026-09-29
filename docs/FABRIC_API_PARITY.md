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

**SD** marks what Storage Drawers 26.3.0.1 needs.

## Fabric API modules

### fabric-api-base
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| `Event`, `EventFactory`, `TriState`, `EventResult` | ➖ | Pumpkin's own event system (`register-event`) | |

### fabric-networking-api-v1
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| Fabric handshake, `c:register` / `minecraft:register` channels | ✅ | `pumpkin-fabric` | ✅ |
| `ServerPlayNetworking.send` (custom payload to a player) | ✅ | `player.send-custom-payload` | ✅ |
| `ServerPlayNetworking.registerGlobalReceiver` | ✅ | `player-custom-payload-event` | |
| `PayloadTypeRegistry` (payload codecs) | ➖ | the plugin encodes its own bytes | |
| `ServerPlayConnectionEvents` INIT / JOIN / DISCONNECT | ✅ | `player-join-event`, `player-leave-event` | |
| `ServerConfigurationNetworking`, configuration tasks | ❌ | no plugin hook in the configuration phase | |
| `ServerLoginNetworking`, login queries | 🟡 | `player-pre-login-event`, `player-login-event`; no login query/response | |
| `EntityTrackingEvents` START / STOP_TRACKING | ❌ | | |
| `PlayerLookup.tracking(...)` (players watching a chunk, block entity or entity) | ❌ | only all players or players in a world | ✅ `count_update` |

### fabric-registry-sync-v0
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| Static registry sync (`fabric:registry/sync`) | ✅ | `pumpkin-fabric`, byte identical | ✅ |
| Modded blocks, items, block entity types from the dump | ✅ | `pumpkin-registry-ext` | ✅ |
| `DynamicRegistries` (modded synced dynamic registries) | ❌ | not checked yet | |
| `RegistryEntryAddedCallback`, `RegistryIdRemapCallback` | ➖ | registries are fixed after load | |

### fabric-events-interaction-v0
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| `UseBlockCallback` | ✅ | `player-interact-event`; per block: modded block `use` hook | ✅ |
| `AttackBlockCallback` | ✅ | `player-interact-event`, `block-damage-event`; per block: `attack` hook | ✅ |
| `UseItemCallback` | ✅ | `player-interact-event`; per item: modded item `use` hook | ✅ |
| `UseEntityCallback`, `AttackEntityCallback` | ✅ | `player-interact-entity-event`, `entity-damage-by-entity-event` | |
| `PlayerBlockBreakEvents` BEFORE / AFTER / CANCELED | 🟡 | `block-break-event` (before, cancellable); no after/canceled | |
| `PlayerPickItemEvents` (middle click block/entity) | ❌ | | |
| `FakePlayer` | ❌ | | |

### fabric-lifecycle-events-v1
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| `ServerTickEvents` START / END_SERVER_TICK | ✅ | `server-tick-start-event`, `server-tick-end-event` | |
| `ServerTickEvents` per level | ❌ | | |
| `ServerLifecycleEvents` SERVER_STARTED | ✅ | `server-load-event`; plugin `on-load` | |
| `ServerLifecycleEvents` STOPPING / STOPPED, datapack reload | 🟡 | plugin `on-unload`; no reload event | |
| `ServerLevelEvents` LOAD / UNLOAD | ✅ | `world-load-event`, `world-unload-event` | |
| `ServerChunkEvents` LOAD / UNLOAD | ✅ | `chunk-load-event`, `chunk-unload-event` | |
| `ServerEntityEvents` LOAD / UNLOAD | ✅ | `entity-spawn-event`, `entities-load-event`, `entity-remove-event` | |
| `ServerBlockEntityEvents` LOAD / UNLOAD | ❌ | | |
| `CommonLifecycleEvents` TAGS_LOADED | ➖ | tags are fixed after load | |

### fabric-entity-events-v1
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| `ServerLivingEntityEvents` ALLOW_DAMAGE / AFTER_DAMAGE / ALLOW_DEATH / AFTER_DEATH | ✅ | `entity-damage-event`, `entity-death-event`, `entity-resurrect-event` | |
| `ServerPlayerEvents` JOIN / LEAVE / AFTER_RESPAWN / COPY_FROM | 🟡 | join, leave, respawn events; no COPY_FROM (data kept across respawn) | |
| `ServerEntityCombatEvents` AFTER_KILLED_OTHER_ENTITY | 🟡 | `entity-death-event` has the killer | |
| `ServerEntityLevelChangeEvents` | ✅ | `player-change-world-event`, `entity-portal-event` | |
| `EntitySleepEvents` | 🟡 | bed enter/leave; no ALLOW_SLEEP_TIME, bed direction or respawn-anchor hooks | |
| `EntityElytraEvents` ALLOW / CUSTOM | 🟡 | `entity-toggle-glide-event`; no custom elytra | |
| `ServerMobEffectEvents` | ✅ | `entity-potion-effect-event` | |

### fabric-item-api-v1
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| Item `useOn` / `use` for modded items | ✅ | modded item hooks | ✅ |
| Read / write item components by id | ✅ | `modded.get/set/remove-item-component` | ✅ |
| Keep unknown modded components (e.g. `storagedrawers:drawer_count`) | ❌ | `ItemStack` drops them; needs a raw-NBT fallback | ✅ GUI, drops |
| `DefaultItemComponentEvents` | ➖ | default components come from the dump | |
| `FabricItem` (recipe remainder, attribute modifiers, reequip animation) | ❌ | | |
| `CustomDamageHandler` | ❌ | | |
| `EquipmentSlotProvider` | ❌ | | |
| `EnchantmentEvents` ALLOW_ENCHANTING / MODIFY | 🟡 | `prepare-item-enchant-event`, `enchant-item-event`; no per-item allow | |
| `ItemClickBehaviorCallback` (click one stack onto another in a menu) | ❌ | `inventory-click-event` only | |
| `BlockTransformerEvents` (strip, till, flatten...) | ❌ | | |
| `ItemComponentTooltipProviderRegistry` | ➖ | tooltips are drawn by the client | |

### fabric-menu-api-v1
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| Vanilla menu types | ✅ | `gui` resource, `player.open-gui` | |
| `ExtendedMenuType` / `ExtendedMenuProvider` (modded menu, `fabric-menu-api-v1:open_screen` with extra data) | ❌ | | ✅ drawer GUI |
| Modded menu slot layout and quick-move rules | ❌ | | ✅ drawer GUI |

### fabric-object-builder-api-v1
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| `FabricBlockEntityTypeBuilder` | ✅ | types from the dump; `modded.set-block-entity-data` | ✅ |
| `WoodTypeBuilder`, `BlockSetTypeBuilder` | ➖ | resolved in the dump | |
| `FabricEntityType`, modded entities | ❌ | | detached drawers? |
| `FabricDefaultAttributeRegistry` | ❌ | | |
| `FabricEntityDataRegistry` (custom synced entity data) | ❌ | | |
| `PoiHelper` | ❌ | | |
| `MinecartComparatorLogicRegistry` | ❌ | | |

### fabric-data-attachment-api-v1
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| Persistent data on entities and block entities | 🟡 | `set-custom-data` on entities and block entities; not Fabric's NBT layout | |
| Data on chunks and levels | ❌ | | |
| `AttachmentSyncPredicate` (sync to clients, `fabric:attachment_sync`) | ❌ | | |

### fabric-lookup-api-v1 and fabric-transfer-api-v1
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| `BlockApiLookup` / `ItemApiLookup` / `EntityApiLookup` | ❌ | | ✅ controller, hoppers |
| `ItemStorage.SIDED`, `Storage<ItemVariant>`, transactions | ❌ | | ✅ hoppers into drawers |
| `FluidStorage` | ❌ | | |

Vanilla hoppers moving items into a plugin block also need the host to ask the plugin for its
inventory.

### fabric-loot-api-v3
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| Mod loot tables | ✅ | imported from the dump | ✅ |
| Custom drops from code (`getDrops`) | ✅ | modded block `drops` hook | ✅ |
| `LootTableEvents` REPLACE / MODIFY / MODIFY_DROPS | 🟡 | `loot-generate-event`; can't change tables | |

### fabric-recipe-api-v1
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| Mod recipes | 🟡 | imported from the dump; 5 recipes of custom serializer types are skipped | ✅ |
| Register shaped / shapeless / cooking recipes from code | ✅ | `recipe.register-*` | |
| `CustomIngredient` (`fabric:all_of`, `any_of`, `components`, `difference`) | ❌ | | |
| Look up recipes from code (`RecipeManager`) | ❌ | | ✅ compacting drawers |
| `RecipeSynchronization` (send recipes to the client) | ❌ | | |

### fabric-command-api-v2
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| `CommandRegistrationCallback` | ✅ | `register-command` | |
| `ArgumentTypeRegistry` (modded argument types) | ❌ | | |
| `EntitySelectorOptionRegistry` | ❌ | | |

### fabric-content-registries-v0
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| Flammable, oxidizable, strippable, compostable, fuel, villager interactions, path types, vibration frequencies | ❌ | vanilla values are hardcoded | |
| `FluidBehavior`, `EntityFluidInteractionRegistry` | ❌ | | |

### fabric-game-rule-api-v1
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| Read and set vanilla game rules | ✅ | `world.get-game-rule`, `set-game-rule` | |
| `GameRuleBuilder` (modded game rules), `GameRuleEvents` | ❌ | | |

### fabric-permission-api-v1
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| Permission nodes and checks | ✅ | `register-permission`, `player.has-permission` | |
| `PermissionEvents` | ✅ | `player-permission-check-event` | |

### fabric-message-api-v1
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| `ServerMessageEvents` ALLOW / CHAT / COMMAND / GAME | ✅ | `player-chat-event`, `player-command-preprocess-event`, `server-broadcast-event` | |
| `ServerMessageDecoratorEvent` | ❌ | | |

### fabric-advancement-api-v1
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| Mod advancements | 🟡 | loaded with the datapack if dumped; not checked | |
| Grant / revoke advancements, `AdvancementEvents` | ✅ | `player.award-advancement`, `revoke-advancement`, `player-advancement-done-event` | |

### fabric-biome-api-v1 and fabric-dimensions-v1
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| `BiomeModifications` (add features and spawns) | ❌ | | |
| `NetherBiomes`, `TheEndBiomes` | ❌ | | |
| `DimensionEvents`, teleport across dimensions | ✅ | `teleport-world`, `entity-portal-event` | |

### fabric-particles-v1
| Feature | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| Spawn vanilla particles | ✅ | `player.spawn-particles` | |
| Spawn modded particle types | ❌ | `particle` is a vanilla-only enum | |

### Data-only and client-only modules
| Module | Status | Why |
|:--|:--|:--|
| `fabric-convention-tags-v2`, `fabric-tag-api-v1` | ✅ | every non-`minecraft` tag is in the dump |
| `fabric-resource-conditions-api-v1` | ➖ | conditions are resolved when the dump runs |
| `fabric-resource-loader-v1` | 🟡 | mod data comes from the dump; no reload listeners |
| `fabric-creative-tab-api-v1` | ➖ | client side; the registry is synced |
| `fabric-block-api-v1` | ➖ | block appearance is client side |
| `fabric-block-getter-api-v2` | ✅ | render data is block entity data, sent by `set-block-entity-data` |
| `fabric-serialization-api-v1` | ➖ | Java codec helpers |
| `fabric-data-generation-api-v1`, `fabric-gametest-api-v1`, `fabric-debug-api-v1` | ➖ | dev time |
| rendering, model loading, screen, key mapping, sound, client gametest | ➖ | client only |

## Vanilla overrides mods rely on

Most mod server logic overrides vanilla `Block` and `Item` methods rather than using Fabric API.
These are the `modded.wit` hooks.

| Vanilla method | Status | Pumpkin | SD |
|:--|:--|:--|:--|
| `Block.getStateForPlacement` | ✅ | `placement-state` | ✅ |
| `Block.setPlacedBy` | ✅ | `placed` | ✅ |
| `Block.useWithoutItem` / `useItemOn` | ✅ | `use` | ✅ |
| `Block.attack` | ✅ | `attack` | ✅ |
| `Block.getDrops` | ✅ | `drops` | ✅ |
| `Block.affectNeighborsAfterRemoval` / `onRemove` | ✅ | `removed` | ✅ |
| `Block.tick` (scheduled) | ✅ | `scheduled-tick`, `schedule-block-tick` | ✅ |
| `Block.getSignal` / `getDirectSignal` / `isSignalSource` | ❌ | | ✅ redstone upgrade |
| `Block.getAnalogOutputSignal` (comparator) | ❌ | | ✅ |
| `Block.neighborChanged`, `updateShape` | ❌ | | |
| `Block.randomTick` | ❌ | | |
| `Block.entityInside`, `stepOn` | ❌ | | |
| `BlockEntity` ticker | ❌ | only scheduled ticks | ✅ hopper/magnet upgrades |
| `Item.useOn` / `use` | ✅ | item hooks | ✅ |
| `Item.inventoryTick` | ❌ | | |
| Find entities in an area | 🟡 | `entity.get-nearby-entities` (around an entity); no box query on a world | ✅ magnet |
