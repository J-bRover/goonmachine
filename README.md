# RICO-32

<p align="center">
  <img src="https://github.com/RICO-32.png" alt="RICO-32 profile picture" width="200"/>
</p>

<div align="center">

**The Rust Imaginary Console Offline**

A fantasy console inspired by PICO-8, built with Rust for performance and Lua for scripting.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

</div>

---

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Screenshots](#screenshots)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Sprite Engine](#sprite-engine)
- [IDE Engine](#ide-engine)
- [Terminal Engine](#terminal-engine)
- [Cartridge & Lua Files](#cartridge--lua-files)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)
- [TODO](#todo)

## Introduction

RICO-32 is a fantasy console that combines the nostalgic charm of retro game development with modern Rust performance. It features a 128×128 pixel game screen with a 16-color palette, Lua scripting support, and a built-in console for debugging and logging.

The console is designed to be simple yet powerful, allowing developers to create games quickly using Lua while benefiting from Rust's performance and safety guarantees. Whether you're prototyping a game idea or creating a full retro-style game, RICO-32 provides the tools you need.

## Features

- **128×128 Pixel Display**: Classic retro resolution with 4× pixel scaling for modern displays
- **16-Color Palette**: Predefined color palette for consistent retro aesthetics
- **Lua Scripting**: Full Lua 5.4 support with custom module system. Only safe Lua features are enabled, allowing for security when running external cartridges.
- **Built-in Console**: Integrated console engine for logging and debugging
- **Sprite Support**: Create custom 16x16 sprites within the console and use and load them in the game
- **Built-in IDE**: Edit your Lua scripts with syntax highlighting without leaving the console.
- **Input Handling**: Mouse and keyboard input with frame-accurate state tracking
- **Frame Rate Control**: Configurable frame rate with delta time support
- **Modular Architecture**: Clean separation between game engine, console engine, and scripting
- **In Memory Reload Support**: Restart functionality for rapid iteration and smooth development.
- **Full cartridge support**: .r32 files are cartridges that can be shared with other users. RICO-32 will make a main.r32 if one doesn't already exist on startup but any cartridge can be loaded and saved to from the inbuilt terminal.
- **Full inbuilt terminal**: Contains commands to load cartridges, save cartridges to different formats, and export standalone executables.

## Screenshots

### GIF of Tetris Example
![TETRISRICO](https://github.com/user-attachments/assets/bf24e719-d5b9-487a-964a-db928732d384)

### GIF of Platformer Example
![ezgif-6b0bad010cd3138b](https://github.com/user-attachments/assets/1e577a1e-ec80-4495-8166-a6d24eb36ce6)


**more examples eventually...**


## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70 or later)
- Cargo (comes with Rust)
- A C compiler (for building native dependencies) (i think)

### Building from Source

1. **Clone the repository**:
   ```bash
   git clone https://github.com/RICO-32/RICO-32.git
   cd RICO-32
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Run the project**:
   ```bash
   cargo run
   ```

   Or run the release binary directly:
   ```bash
   cargo run --release
   ```

## Quick Start

1. **Run RICO-32**: Execute `cargo run --release` and your RICO-32 will start!

2. **Create your game scripts**: Edit `r32/main.lua` to start building your game. You can use the built-in IDE or your favorite external editor.
   
3. **Build the game**: Use the sprite engine to create and edit sprites and the console to debug your game.

4. Building a final standalone executable for your game can be done by running the following in the inbuilt terminal.
```bash
export <filename>
```

### Example Game

```lua
function start()
    rico:log("Welcome to RICO-32!")
    rico:set_frame_rate(60)
end

function update(dt)
    rico:clear("BLACK")
    rico:print_scr(10, 10, "WHITE", "Hello, World!")
    
    local mouse = rico:mouse()
    if mouse.pressed then
        rico:circle(mouse.x, mouse.y, 5, "RED")
    end
end
```

## Sprite Engine

RICO-32 includes a built-in sprite editor for creating and managing 32×32 pixel sprites. Access the sprite editor from the main interface to design sprites that can be used in your games.

### Features

- **32×32 Sprite Canvas**: Edit sprites pixel-by-pixel with a 3× zoomed preview
- **60-Sprite Sheet**: Store up to 60 sprites (expandable) in a persistent sprite sheet
     - RICO-32 features an inbuilt sprite file spec to store and read the sprite sheet for the engine
- **Drawing Tools**:
  - **Pencil**: Draw individual pixels with the selected color
  - **Fill**: Flood-fill connected areas with the selected color
  - **Eraser**: Remove pixels (set to transparent)
  - **Select**: Create rectangular selections for advanced editing
- **Selection Tools**:
  - Click and drag to create rectangular selections
  - Move selections by clicking inside and dragging
  - Copy (Ctrl+C) and paste (Ctrl+V) selections
  - Flip horizontal/vertical within selections or entire sprite
  - Clear selected areas or entire sprite
- **Undo/Redo**: Full undo/redo support with keyboard shortcuts
- **16-Color Palette**: Quick access to all RICO-32 colors
- **Auto-Save Indicator**: Changes are marked with an asterisk (*) until saved

### Using the Sprite Editor

1. **Select a sprite**: Click on any sprite in the sprite sheet panel (bottom of screen). Scroll to view more sprites.
2. **Choose a tool**: Click one of the tool buttons (Pencil, Eraser, Fill, Select)
3. **Pick a color**: Click a color from the palette at the top
4. **Draw**: Click and drag on the canvas to draw or use tools
5. **Save**: Click the save button to persist changes to disk

### Adding More Sprites

Click the **+** button at the bottom of the sprite sheet panel to add 6 more sprite slots. The sprite sheet automatically expands to accommodate your needs.

### In-Game Usage

Load and draw sprites in your Lua scripts using the `draw()` function:

```lua
function start()
    rico:set_frame_rate(60)
end

function update(dt)
    rico:clear("BLACK")
    
    -- Draw sprite 0 at position (48, 48)
    rico:draw(48, 48, 0)
    
    -- Draw sprite 5 at position (80, 48)
    rico:draw(80, 48, 5)
end
```

## IDE Engine

RICO-32 features a built-in code editor for Lua scripts, accessible from the "IDE" tab.

### Features

- **Syntax Highlighting**: Code is colored to distinguish keywords, strings, numbers, and comments.
- **File Browser**: A list of all `.lua` files in the `r32/` directory is shown at the bottom. Click a file to open it.
- **Create New Files**: Click the `+` button to create a new Lua file.
- **Live External Reload**: The IDE automatically detects and reloads files if they are changed by an external editor.
- **Status Bar**: Displays the current file name, line/column number, and an asterisk (`*`) for unsaved changes.
- **Standard Editing Controls**:
  - Full mouse support for cursor placement and selection.
  - **Ctrl+S**: Save the current file.
  - **Ctrl+C**: Copy selected text.
  - **Ctrl+X**: Cut selected text.
  - **Ctrl+V**: Paste text from the clipboard.
  - **Ctrl+Z**: Undo the last action.
  - **Ctrl+R**: Redo the last undone action.
  - **Ctrl+A**: Select all text in the file.
  - **Arrow Keys**: Move the cursor.
  - **Shift + Arrow Keys**: Create or modify a selection.
  - **Tab**: Insert 4 spaces.
  - **Enter**: Insert a new line.
  - **Backspace**: Delete characters or the current selection.

## Terminal Engine

RICO-32 features an inbuilt terminal to perform essential RICO-related commands.

### Commands 
- `load <filepath>`: Loads a cartridge into memory, can be a normal `.r32` cartridge or a base 64 version stored as `.r32.txt`.
- `save <filepath>`: Saves the current loaded cartridge to a file on disk, can be a normal `.r32` cartridge or a base 64 version stored as `.r32.txt`.
- `export <filepath>`: Exports the current loaded cartridge as a standalone executable for the current operating system. There is currently only support for linux and windows.
- `mv <source> <destination>`: Moves or renames a file within the `r32/` directory. Parent directories for the destination will be created if they don't exist.
- `rm <filename>`: Removes a Lua file from the `r32/` directory.
- `help [cmd]`: prints out a list of commands what they do. If a command is specified, gives details on how to use that command and a bit more info about it.

## Cartridge & Lua Files

- Lua files are **extracted to `r32/`** next to the executable and cartridge when a game is loaded.
- Users can **edit files externally** with their favorite editor.
- RICO-32 automatically watches the `r32/` folder for any changes made through any IDE and auto recompiles the cartridge to be instantly loaded whenever the game is restarted through restarting the whole engine or the inbuilt game restart.
- Sprites are **never written to r32/**, remaining fully in memory and the cartridge, and must be saved to the cartridge using the checkmark within the sprite editor.
- Cartridges fully contain all information related to all RICO-32 games, and thus RICO-32 games can be shared easily by sharing .r32 files or using their Base64 version.
- To convert the formats of your cartridges from `.r32` to `.r32.txt` (the base64 version), simply load whichever format from the inbuilt terminal and then save it back to whichever format.
```bash
load cart.r32
save b64.r32.txt
```

### Note: all changes made to the r32/ directory while RICO-32 is not running will be discarded and overwritten with the cartridge upon initial RICO-32 startup.

## API Reference

All RICO API functions must be prefixed with `rico:` to be called in Lua. This does not include the core functions listed below.

### Core Functions

#### `start()`
Called once at the start of the program. Use this for initialization.

#### `update(dt)`
Called every frame. `dt` is the delta time in milliseconds since the last frame.

### Graphics Functions

#### `set_pix(x, y, COLOR)`
Sets the pixel at coordinates `(x, y)` to the specified color.

**Parameters:**
- `x` (number): X coordinate (0-127)
- `y` (number): Y coordinate (0-127)
- `COLOR` (string): One of the 16 color names

#### `get_pix(x, y) -> COLOR`
Gets the color of the pixel at coordinates `(x, y)`.

**Returns:** Color name as a string

#### `clear(COLOR)`
Fills the entire screen with the specified color.

#### `rectfill(x, y, w, h, COLOR)`
Fills a rectangle with the specified color.

**Parameters:**
- `x, y` (number): Top-left corner coordinates
- `w, h` (number): Width and height
- `COLOR` (string): Fill color

#### `rect(x, y, w, h, COLOR)`
Draws a rectangle outline with the specified color.

#### `circle(x, y, r, COLOR)`
Draws a filled circle.

**Parameters:**
- `x, y` (number): Center coordinates
- `r` (number): Radius
- `COLOR` (string): Fill color

#### `draw(x, y, idx)`
Draws a sprite created in the sprite engine.

**Parameters:**
- `x, y` (number): Top-left corner coordinates
- `idx` (number): Index of the sprite (check by clicking on it in the sprite tab, should display which sprite is being edited)

### Text Functions

#### `print_scr(x, y, COLOR, text)`
Prints text to the screen. Each character is 8×8 pixels.

#### `print_scr_mid(x, y, COLOR, text)`
Prints text with medium-sized characters (4×6 pixels).

#### `print_scr_mini(x, y, COLOR, text)`
Prints text with mini-sized characters (4×4 pixels).

### Input Functions

#### `mouse() -> mouse_object`
Returns a mouse object with the following properties:
- `.just_pressed` (boolean): Whether the left button was just pressed (lasts 1 frame)
- `.pressed` (boolean): Whether the left button is currently pressed
- `.x` (number): X coordinate in pixels (-1 if outside window)
- `.y` (number): Y coordinate in pixels (-1 if outside window)

#### `key_pressed(NAME) -> boolean`
Returns whether the specified key is currently pressed.

#### `key_just_pressed(NAME) -> boolean`
Returns whether the specified key was just pressed this frame.

**Supported Keys:**
- Numbers: `"1"` through `"0"`
- Letters: `"A"` through `"Z"`
- Arrows: `"Left"`, `"Up"`, `"Right"`, `"Down"`
- Special: `"Back"`, `"Enter"`, `"Space"`

### System Functions

#### `log(message)`
Prints a message to the console. Messages are displayed in the console panel below the game screen.

#### `set_frame_rate(rate)`
Sets the target frame rate. Set to 0 or negative for unlimited frame rate.

### Colors

The following 16 colors are available:

- `"BLACK", "WHITE","GRAY", "SILVER"`
- `"RED", "MAROON", "ORANGE", "YELLOW"`
- `"GOLD", "GREEN", "OLIVE", "BROWN"`
- `"BLUE", "TEAL", "PURPLE", "PINK"`

## Examples

The project includes several example games in the `examples/` directory:

- **Platformer**: A simple platformer game
- **Shooter**: A top-down shooter example
- **Tetris**: Classic Tetris implementation

To run an example, simply run the following in the inbuilt terminal.
```bash
load <filepath>
```

## Contributing

Contributions are welcome! Here's how you can help:

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes**: Follow Rust and Lua best practices
4. **Commit your changes**: `git commit -m 'Add amazing feature'`
5. **Push to the branch**: `git push origin feature/amazing-feature`
6. **Open a Pull Request**: Provide a clear description of your changes

### Development Guidelines

- Follow Rust naming conventions and style guidelines
- Add comments for complex logic
- Update documentation for new API functions
- Test your changes with example games
- Ensure the project builds with `cargo build`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

Copyright (c) 2025 Dhruv Goel

## TODO

### Future Enhancements

- [ ] Sound and music support
- [ ] Save/load game state functionality
- [ ] Export to web (WebAssembly)
- [ ] Additional graphics primitives (lines, polygons)
- [ ] Tilemap support
- [ ] Physics engine integration
- [ ] Networking capabilities for multiplayer games

---

<div align="center">

Made with <3

[Report Bug](https://github.com/RICO-32/RICO-32/issues) · [Request Feature](https://github.com/RICO-32/RICO-32/issues)

</div>
