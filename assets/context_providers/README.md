# Context providers (1.21.1 branch)

Vanilla 1.21.1 hardcodes these values in Java (furnace fuel times in
`AbstractFurnaceBlockEntity`, halved by blast furnaces and smokers, `BrewingStandBlockEntity.FUEL_USES`,
the composter chance tiers in `ComposterBlock`). The shared Pumpkin code reads them as 26.x
context providers, so this branch keeps them here in that format, with 1.21.1's values. The
vanilla datapack itself is downloaded from the 1.21.1 server jar at build time and has none of them.
