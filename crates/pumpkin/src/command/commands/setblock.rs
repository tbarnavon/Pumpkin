use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockFlags;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::block::BlockArgumentType;
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Place a block.";
const PERMISSION: &str = "minecraft:command.setblock";

const ERROR_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SETBLOCK_FAILED,
    translation::java::COMMANDS_SETBLOCK_FAILED,
);

#[derive(Clone, Copy)]
enum Mode {
    /// with particles + item drops
    Destroy,

    /// only replaces air
    Keep,

    /// default; without particles
    Replace,

    /// places block without triggering updates around it
    Strict,
}

struct SetBlockExecutor(Mode);

/// `BlockInput.place` with a tag: the block entity now at `pos` loads `nbt` over its own data.
pub(super) fn load_block_entity_nbt(
    world: &crate::world::World,
    pos: &pumpkin_util::math::position::BlockPos,
    nbt: &pumpkin_nbt::compound::NbtCompound,
) {
    let Some(existing) = world.get_block_entity(pos) else {
        return;
    };
    let mut data = pumpkin_nbt::compound::NbtCompound::new();
    existing.write_nbt(&mut data);
    data.merge(nbt);
    data.put_string("id", existing.resource_location().to_string());
    data.put_int("x", pos.0.x);
    data.put_int("y", pos.0.y);
    data.put_int("z", pos.0.z);
    if let Some(block_entity) = crate::block::entities::block_entity_from_nbt(&data) {
        world.add_block_entity(block_entity);
    }
}

impl CommandExecutor for SetBlockExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let input = BlockArgumentType::get_state(context, "block")?;
        let block_state_id = input.state;
        let mode = self.0;
        let world = context.source.world();
        let pos = BlockPosArgumentType::get_loaded_block_pos(context, "pos")?;

        let success = match mode {
            Mode::Destroy => {
                world.break_block(
                    &pos,
                    None,
                    BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL | BlockFlags::FORCE_STATE,
                );
                world.set_block_state(
                    &pos,
                    block_state_id,
                    BlockFlags::NOTIFY_ALL | BlockFlags::FORCE_STATE,
                );
                true
            }
            Mode::Replace => {
                world.set_block_state(
                    &pos,
                    block_state_id,
                    BlockFlags::NOTIFY_ALL | BlockFlags::FORCE_STATE,
                );
                true
            }
            Mode::Keep => {
                let old_state = world.get_block_state(&pos);
                if old_state.is_air() {
                    world.set_block_state(
                        &pos,
                        block_state_id,
                        BlockFlags::NOTIFY_ALL | BlockFlags::FORCE_STATE,
                    );
                    true
                } else {
                    false
                }
            }
            Mode::Strict => {
                world.set_block_state(
                    &pos,
                    block_state_id,
                    BlockFlags::NOTIFY_LISTENERS
                        | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK
                        | BlockFlags::FORCE_STATE,
                );
                true
            }
        };

        if success {
            if let Some(nbt) = &input.nbt {
                load_block_entity_nbt(world, &pos, nbt);
            }
            world.flush_block_updates();
            context.source.send_feedback(
                TextComponent::translate_cross(
                    pumpkin_data::translation::java::COMMANDS_SETBLOCK_SUCCESS,
                    pumpkin_data::translation::bedrock::COMMANDS_SETBLOCK_SUCCESS,
                    [
                        TextComponent::text(pos.0.x.to_string()),
                        TextComponent::text(pos.0.y.to_string()),
                        TextComponent::text(pos.0.z.to_string()),
                    ],
                ),
                true,
            );
            Ok(1)
        } else {
            Err(ERROR_FAILED.create_without_context())
        }
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("setblock", DESCRIPTION).requires(PERMISSION).then(
            argument("pos", BlockPosArgumentType).then(
                argument("block", BlockArgumentType)
                    .executes(SetBlockExecutor(Mode::Replace))
                    .then(literal("destroy").executes(SetBlockExecutor(Mode::Destroy)))
                    .then(literal("keep").executes(SetBlockExecutor(Mode::Keep)))
                    .then(literal("replace").executes(SetBlockExecutor(Mode::Replace)))
                    .then(literal("strict").executes(SetBlockExecutor(Mode::Strict))),
            ),
        ),
    );
}
