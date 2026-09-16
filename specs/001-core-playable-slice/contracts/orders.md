# Contract: Orders

An `Order` is the only thing that enters the simulation and the only thing a recording stores
(FR-002). Everything the player, an AI, a script, or a remote peer can ever do is expressible here.

## Shape

```text
Order {
  tick:   u32            // tick at which it takes effect
  issuer: Player
  target: Units([Handle]) | Group(GroupId)
  action: Move | Attack | Build | Reclaim | Load | Unload | Stop
                | DefineGroup | SetStanding
  params: position | handle | class | predicate      // action-dependent
}
```

## Rules

- **Addressing.** An order targets either an explicit set of units or a group. Targeting a group is
  not sugar for expanding it — the order attaches to the group, so units joining later inherit it
  (FR-033).
- **Validation.** `Sim::validate` rejects before the order is recorded. Grounds: impassable
  destination for the target's domain, target not owned by the issuer, unknown or stale handle,
  action not supported by the target's class. A rejected order is never recorded and never replayed.
- **Precedence.** A personal order supersedes a group standing order until the unit's queue empties
  (FR-034).
- **Same-tick conflict.** Orders are applied in list order; the later one wins. Because the list is
  what gets recorded, replay resolves identically.
- **Group rules are orders too.** `DefineGroup` and `SetStanding` are ordinary orders, which is why
  group configuration survives replay without a separate mechanism.

## Extension rule

Adding an action MUST NOT change this shape. An action that cannot be expressed as
`(tick, issuer, target, action, params)` is a signal that the design has grown a side channel around
the simulation boundary, and MUST be reconsidered rather than accommodated.
