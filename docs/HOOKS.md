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
| `entity-inside` (opt-in, batched) | `Block.entityInside` | `044588b3a`, batched in `TICKBATCH` |
| `step-on` (opt-in, batched) | `Block.stepOn` | `044588b3a`, batched in `TICKBATCH` |
| `ticker` (opt-in, batched) | `EntityBlock.getTicker` | `dfe6c259c`, batched in `TICKBATCH` |

## Item hooks (`modded.item-hooks`, `context.register-item-hooks`)

Called through the plugin export `handle-item-hook`.

| Hook | Java | Commit |
|:--|:--|:--|
| `use-on-block` | `Item.useOn` | `0084f1557` |
| `use` | `Item.use` | `0084f1557` |
| `inventory-tick` (opt-in, batched) | `Item.inventoryTick` | `c3b1024a1`, batched in `TICKBATCH` |
| `stacked-on-me` | `Item.overrideOtherStackedOnMe` | `13b9d100c` |
| `stacked-on-other` | `Item.overrideStackedOnOther` | `13b9d100c` |
| `destroyed` | `Item.onDestroyed` | `13b9d100c` |
| `not-in-containers` (no call) | `Item.canFitInsideContainerItems` | `13b9d100c` |

Hooks marked *batched* run every tick. The host queues them while the world ticks and sends
each plugin one `handle-tick-batch` call per world after block entities tick (see below).

## Other plugin exports and registrations

| API | What it does | Commit |
|:--|:--|:--|
| export `handle-tick-batch` (`modded.tick-batch`) | One call per plugin, world and tick with all its batched per-tick hooks, in vanilla order: inventories, entity contacts, block entities | `TICKBATCH` |
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

## Known gaps

From the old Fabric API parity table (removed 2026-10-03). These are inputs for the hook
catalog (`ROADMAP.md`, item 6), not commitments.

- **Entities:** modded entity types, default attributes, custom synced entity data, fake
  players, custom damage handlers, equipment slots, POI types, minecart comparator logic.
- **Blocks and items:** block transformers (strip, till, flatten), oxidizable, strippable and
  path-type registries, fluid behaviour, `FluidStorage`, block / item / entity API lookups,
  `FabricItem` (recipe remainder, attribute modifiers), per-item enchanting rules.
- **Item storage:** no sides and no transactions on `set-item-storage`.
- **World and server:** per-world tick events, a datapack reload event, modded game rules,
  biome modifications, modded particle types, modded command argument types and entity
  selector options, synced dynamic registries.
- **Data:** loot tables can't be changed (only drops replaced), recipes aren't synced to the
  client, attachment sync to clients, no resource reload listeners.
- **Partial:** configuration tasks that hold the phase for a reply; sleep (no bed direction or
  wake-up position); elytra (no custom elytra); kill events (only `entity-death-event`).
