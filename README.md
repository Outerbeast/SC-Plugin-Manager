# Sven Co-op Plugin Manager
![alt text](https://github.com/Outerbeast/SC-Plugin-Manager/blob/main/preview.png?raw=true)

Sven Co-op Plugin Manager — a small Windows GUI utility to view and manage Sven Co-op server plugin entries (enabled/disabled) stored in the game's plugin files.

**Purpose**
- Make it easy to inspect, enable/disable, and save plugin entries for Sven Co-op.
- Maintain two files in the game install: `default_plugins.txt` (enabled plugins) and `disabled_plugins.txt` (disabled plugins).

## Installation
- Download the application from the [Releases](https://github.com/Outerbeast/SC-Plugin-Manager/releases/) section
- Launch the executable for initial setup, this will search for your Sven Co-op game install.

## Usage
🔍 Plugin List (Left Panel)
- Browse available plugins: Each plugin is listed with a checkbox.
- Enable/Disable plugins: Toggle the checkbox to activate or deactivate a plugin.

⚙ Plugin Configuration (Right Panel)
When a plugin is selected, you can configure its behavior:
- Name: Display name of the plugin.
- Script: Path or identifier for the plugin script.
- Command NS: Namespace prefix for plugin commands.
- Admin Level: Choose required access level from the dropdown.
- Included Maps: List of maps where the plugin is active.
- Excluded Maps: List of maps where the plugin is disabled.

✅ Plugin Controls (Bottom Section)
- Enabled: Checkbox to toggle plugin activation.
- Add: Create a new plugin entry.
- Remove: Delete the selected plugin.
- Apply: Save changes to the current plugin.
- Save: Persist all plugin configurations to disk.

💡 Tips
- Use the Apply button before switching plugins to avoid losing changes.
- The Save button writes all plugin states to the Sven Co-op plugin file.
- If a plugin doesn’t behave as expected, check the `Included Maps` and `Excluded Maps` fields

For more detailed information on how to configure plugins, please refer to to the [official documentation](https://wiki.svencoop.com/Running_Scripts#Plugins)

### Quick Add

# Building from source

## Prerequisites

1️⃣ Install Rust
- Visit [https://rustup.rs](https://rustup.rs) and download the Windows installer.
- Run it and accept the defaults (this installs `cargo`, `rustc`, and `rustup`).
- Close and reopen any terminal/PowerShell windows after installation.

---

2️⃣ Install Windows Build Tools
The GUI uses the Windows API, so you need the C++ build toolchain:

**Option A (Recommended)**  
- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).  
- During installation, select **"Desktop development with C++"**.

**Option B**  
- Add the MSVC target via:
```powershell
rustup target add x86_64-pc-windows-msvc
```

## Build instructions
1. Download or clone the repository:

```cmd
git clone https://github.com/Outerbeast/SC-Plugin-Manager
cd yourproject
```

2. Run the build script:
- Double-click build.cmd or run it manually:
```cmd
build.cmd
```

The executable will be generated in the current directory.

# Feedback & Issues
If you have feedback or encounter issues, please open an issue on [GitHub Issues](https://github.com/Outerbeast/SC-Plugin-Manager/issues).


### Credits
- Outerbeast - Author
