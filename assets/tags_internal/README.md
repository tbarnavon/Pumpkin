Tags of later versions that Pumpkin's game code checks where 1.21.1 hard-codes the same rule (which
blocks support a crop, which mobs burn in daylight, ...), with the tags they reference that 1.21.1
doesn't have. Codegen resolves them against 1.21.1's ids and keeps them server-side: they are never
sent to clients, and a vanilla 1.21.1 tag of the same name wins.
