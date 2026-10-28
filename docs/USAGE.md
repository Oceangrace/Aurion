[← Aurion](../README.md) · **English** / [Русский](ru/USAGE.md)

# User guide

**User guide** · [Development](DEVELOPMENT.md) · [Protocol](PROTOCOL.md)

## Launching

The app runs on Windows 10/11. It requires WebView2 and
[Npcap](https://npcap.com/) for live capture. Launch the built `aurion.exe` file.
See the [development guide](DEVELOPMENT.md) for building from source.

### First launch while already in the game

Starting Aurion before entering the world supplies the full initial character data.
When starting inside an existing session, Aurion can identify your class from your observed
class-specific skills and load matching HUDs without re-entering the world. Common skills
do not identify a class. Importing a profile only loads its layout and rules.

The editor indicates when character data is incomplete. Missing names, HP/MP values, and
the full skill list require the corresponding server messages; they are not inferred from
other characters. If these remain missing, keep Aurion open, return to character selection,
and enter the world again.
After a successful connection, Aurion saves a session snapshot for later restarts in the same game session.

### Updates

Aurion checks the latest public GitHub release at startup and every four hours.
A newer stable version appears in the editor and settings, with an indicator on the toolbar.
Open **Settings → Updates** to see your installed version, check manually, or open the release page.
Download and extract the new archive yourself; Aurion does not replace its executable automatically.
An unavailable network does not block the HUD or editor.

### Npcap and permissions

Install [Npcap](https://npcap.com/#download). Aurion loads Npcap directly, so
“Install Npcap in WinPcap API-compatible Mode” is not required.
If you enable the installer option that restricts capture to administrators,
run Aurion as administrator. See the [Npcap installation guide](https://npcap.com/guide/npcap-users-guide.html)
for details on these options.

If AION 2 runs as administrator, launch Aurion at the same privilege level so
Alt-based HUD visibility works while the game has focus.

## Editor and toolbar

<table width="100%">
  <thead>
    <tr>
      <th align="left" width="27%">Setting</th>
      <th align="left" width="31%">Where to find it</th>
      <th align="left" width="42%">What it changes</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>Compact toolbar</strong></td>
      <td>Settings → Toolbar</td>
      <td>Hover to expand; click the arrow to hide</td>
    </tr>
    <tr>
      <td><strong>HUD and DPS/HPS</strong></td>
      <td>Settings → Modules</td>
      <td>Enable or disable modules</td>
    </tr>
    <tr>
      <td><strong>HUD scale</strong></td>
      <td>Information → HUD scale</td>
      <td>Size of elements over the game</td>
    </tr>
    <tr>
      <td><strong>Editor scale</strong></td>
      <td>Application settings</td>
      <td>Size of panels and utility windows</td>
    </tr>
  </tbody>
</table>

### Layers

Layers are docked on the right by default. Open **⋯** in the Layers header to choose
**Right**, **Left**, or **Separate window**. Drag the divider to adjust the docked width;
the separate panel can be moved by its header and resized from its edges.
Placement, docked width, and the separate panel's position and size are remembered locally.
The Layers button in the editor rail hides or reopens the panel.

### Keyboard shortcuts

<table width="100%">
  <thead>
    <tr>
      <th align="left" width="40%">Action</th>
      <th align="left" width="60%">Keyboard and mouse</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Save</td>
      <td><kbd>Ctrl</kbd> + <kbd>S</kbd></td>
    </tr>
    <tr>
      <td>Undo</td>
      <td><kbd>Ctrl</kbd> + <kbd>Z</kbd></td>
    </tr>
    <tr>
      <td>Redo</td>
      <td><kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>Z</kbd> or <kbd>Ctrl</kbd> + <kbd>Y</kbd></td>
    </tr>
    <tr>
      <td>Duplicate element</td>
      <td><kbd>Ctrl</kbd> + <kbd>D</kbd></td>
    </tr>
    <tr>
      <td>Delete element</td>
      <td><kbd>Delete</kbd></td>
    </tr>
    <tr>
      <td>Clear selection</td>
      <td><kbd>Esc</kbd></td>
    </tr>
    <tr>
      <td>Move by 1 px</td>
      <td><kbd>←</kbd> <kbd>↑</kbd> <kbd>↓</kbd> <kbd>→</kbd></td>
    </tr>
    <tr>
      <td>Move by 10 px</td>
      <td><kbd>Shift</kbd> + arrow keys</td>
    </tr>
    <tr>
      <td>Keep proportions</td>
      <td><kbd>Shift</kbd> + resize</td>
    </tr>
    <tr>
      <td>Move without snapping</td>
      <td><kbd>Alt</kbd> + drag</td>
    </tr>
    <tr>
      <td>Move the selected group</td>
      <td><kbd>Ctrl</kbd> + drag a child element</td>
    </tr>
    <tr>
      <td>Add to / remove from selection</td>
      <td><kbd>Shift</kbd> + click in the layer list</td>
    </tr>
  </tbody>
</table>

## Game data and visibility

Aurion reads incoming game traffic through Npcap. Skill and effect icons are downloaded
from `assets.playnccdn.com` and cached locally. Without a connection, cached icons remain available.

In normal play, the HUD and DPS/HPS windows follow game focus and hide over detected game panels.
Holding Alt to show the cursor does not by itself hide the HUD. The editor has separate visibility.

After restarting Aurion while the game is running, saved session data can restore the character
and resources. Restored values remain approximate until fresh game data arrives; some information
may be unavailable until the game sends it again.

## Known limitations

- A boss reset without a death event can leave the fight open and merge the next pull into it.
  This can also happen after a wipe if a party member or their health was unknown.
  Combat-dependent HUD elements can remain active until a confirmed end or map change.
- Party support is experimental. Members or their statistics may be missing after an instance change.
- Charge cancellation is not decoded; the bar can continue until its catalog duration expires.
- Game panels are detected from cursor state and Alt. Opening a panel while holding Alt can
  leave the HUD visible over it.

## User data

Data directory: `%APPDATA%/aurion` (or `AURION_DATA_DIRECTORY`).

```text
preferences.json          application settings
combat-journal.json       journal windows and appearance
combat-reports.json       fight history (50 by default, up to 100)
game-session.json         saved character, resources, skill book and NPC data
world-boss-cache.json     world boss observations for the current game process, grouped by server
icon-cache/               icons downloaded from assets.playnccdn.com
aurion.log                icon download errors
profiles/
  workspace.json          format version, revision, profile order
  <Profile>/
    interface.json        elements, formulas, metadata, HUD scale
    locales/<language>.json  author translations: source text → translation
```
