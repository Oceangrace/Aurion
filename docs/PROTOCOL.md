[← Aurion](../README.md) · **English** / [Русский](ru/PROTOCOL.md)

# AION 2 protocol

[User guide](USAGE.md) · [Development](DEVELOPMENT.md) · **Protocol**

Aurion reads incoming game traffic through Npcap or replays a PCAP recording.
The current parser uses client build 32 data.

## Game data

<table width="100%">
  <thead>
    <tr>
      <th align="left" width="28%">Group</th>
      <th align="left" width="72%">Data used by Aurion</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>Character and target</strong></td>
      <td>Name, class, HP, MP, target selection</td>
    </tr>
    <tr>
      <td><strong>Skills</strong></td>
      <td>Casts, cooldowns, chain steps, learned skills</td>
    </tr>
    <tr>
      <td><strong>Effects</strong></td>
      <td>Buffs, debuffs, stacks, remaining duration</td>
    </tr>
    <tr>
      <td><strong>Combat</strong></td>
      <td>Damage, healing, hits, deaths</td>
    </tr>
    <tr>
      <td><strong>Party</strong></td>
      <td>Members and their characters</td>
    </tr>
    <tr>
      <td><strong>Zone</strong></td>
      <td>Map changes and zone timers</td>
    </tr>
  </tbody>
</table>

Per-message support and limitations are listed in
[opcodes.json](../data/protocol/v32/opcodes.json).

## Limitations

- Missing values remain unknown; they are not treated as zero.
- Capture gaps and late connections can leave fight data incomplete.
- Target of target is inferred from casts and skill uses; actual aggro is unavailable.
- An expired cooldown does not confirm that a skill can be used: mana, range, and control effects also matter.
- Replay accepts classic little-endian PCAP. PCAPNG is not supported.

## Implementation

[Packet parsing](../src-tauri/src/integrations/aion2/protocol/)
converts messages into game events. [Game catalogs](../data/catalog/v32/)
provide names, icons, and skill properties.
