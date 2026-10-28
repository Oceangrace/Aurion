# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Healing and HPS no longer count mana and other resource restoration as healing.
- Journal analysis tabs open whenever their recorded data is available.
- Known world boss timers survive instance travel, capture reconnects and Aurion restarts
  during the same game session. Opening the map is still required to refresh boss data.
- Fixed an old target being selected after changing maps.
- A profile that cannot be loaded no longer blocks other profiles. Its files are preserved,
  and the library explains the problem, including when a newer Aurion may be needed.
- A failing HUD no longer interrupts other interfaces or the DPS/HPS meters.
- Restored missing skills, effects and enemy variants in the HUD editor, including
  hidden sources of boss mechanics. Catalog search supports IDs and loading more results.

### Changed

- Updated the game catalog for AION 2 client build 32.
- Updated the Cleric HUD import with a stamina bar below mana and player and target
  health bars aligned with mana.
- The journal initially shows the current fight; the full run remains available as Σ.
- World boss labels distinguish a recent sighting, an older sighting and an expected respawn.

### Added

- Widget templates for skill cooldowns, effects, resources, cast timers and enemy skill warnings
  in the HUD editor.
- Player stamina for HUD bars, text and conditions, with current, maximum and percentage values.
- World boss rows expose `row.state` for profiles: `waiting`, `countdown`, `observed-alive`,
  `last-seen-alive`, `expected` and `refresh`.

## [0.1.4] - 2026-10-24

### Fixed

- HUD profiles made with an earlier game catalog lost their skill and effect icons and could
  not be saved after the catalog update in 0.1.3. Catalog entries are now found by their
  keys, so existing and imported profiles keep working without picking skills again.
  An entry that is really gone from the catalog shows its unknown state and no longer blocks
  saving, import or export.
- A failed save no longer traps the editor and the overlay. Closing asks whether to stay
  or close the editor with the edits kept; quitting asks whether to stay or quit without saving.
- The editor can be closed while profiles are read-only after a load error.

## [0.1.3] - 2026-10-21

### Changed

- Updated the game catalog for AION 2 client build 31.
- Updated levels for 61 NPC definitions across 58 enemy selections in the HUD editor.
- Corrected 28 Russian skill and related effect labels for Summon Resurrection
  and Command: Proxy to match the current client localization.
- Added definitions for six new passive effects, preserving their hidden display settings.

## [0.1.2] - 2026-10-21

### Changed

- Renamed the GitHub repository from Auiron to Aurion and updated download links.

### Fixed

- Release checks now use the permanent GitHub repository ID, so repository renames
  do not break update notifications.

## [0.1.1] - 2026-10-21

### Added

- Release checks at startup and every four hours, with update notices in the editor,
  a toolbar indicator, and installed version and manual checks in settings.
- A release-page button for downloading updates manually.

### Fixed

- Class-restricted HUDs remaining hidden after starting Aurion inside an existing game
  session: observed own class-specific skills can now restore a missing character class.
- The editor now explains when the HUD is waiting for character data after Aurion
  starts inside an existing game session. The first-launch instructions cover this case.

## [0.1.0] - 2026-10-21

Initial release of Aurion for AION 2.

Requires Windows 10/11, Microsoft Edge WebView2, and Npcap for live capture.

### Added

- Transparent game overlay with a visual HUD editor for icons, bars, text, layers,
  and nested groups. Supports positioning, resizing, alignment, and screen scaling.
- Configurable triggers, conditions, formulas, animations, sounds, and spoken alerts.
- Player, target, and party frames with health, mana, buffs, debuffs, stacks,
  and effect timers where available from observed game data.
- Skill cooldowns, chain steps, cast bars, charge preparation, and recent skill history.
- Periodic healing forecasts with separate displays for useful healing and overhealing.
- World boss timers and configurable alerts before a scheduled spawn.
- Animated combat simulation in the editor with adjustable state and party size,
  kept separate from recorded combat history.
- Configurable DPS/HPS windows and a combat journal with fight and run history,
  charts, skill breakdowns, rotation timelines, effect uptime, deaths, comparisons,
  and CSV export.
- Profile import, export, updates, and author-provided translations. Imported author
  and version fields are read-only in the editor.
- English and Russian localization, a compact toolbar, system tray controls,
  and saved panel positions and settings.
- Passive incoming traffic capture and PCAP replay for offline analysis.
- A ready-to-import Cleric HUD with an accompanying in-game interface preset,
  gameplay GIFs, and editor screenshots.
- Installation, usage, development, and protocol documentation, with issue reporting
  guidelines and bug report and feature request forms in English and Russian.
- GNU GPL-3.0-or-later license.

### Known issues

- A boss reset without a death event, or a wipe with unidentified party members,
  can leave the fight open and merge the next pull into the same record.
- Party support is experimental; members or their statistics may be missing after an instance change.
- Cancelled charges can keep their bar visible until the catalog duration expires.
- A game panel opened while holding Alt may not hide the HUD.

See the [user guide](docs/USAGE.md#known-limitations) for details.
