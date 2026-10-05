# Codegen extras (1.21.1 branch)

Vanilla 1.21.1 hardcodes these registries in Java (villager trades, trial spawner configs, cat,
frog and other mob variants, instruments, decorated pot patterns, block transformers, the day and
villager timelines, …). The shared Pumpkin code is generated from them in the later versions'
data format, so this branch keeps them here as codegen input (`tools/pumpkin-codegen/src/datapack.rs`).
They are not part of the runtime datapack, which is the 1.21.1 server jar's data extracted at
build time. Regenerating keeps today's generated files.
