/*
	Sven Co-op Plugin Manager Version 1.0

Copyright (C) 2025 Outerbeast
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
*/
use std::
{
    fs,
    io,
    collections::HashMap,
    path::
    {
        Path,
        PathBuf
    }
};

pub const FILENAME_PLUGINS: &str = "default_plugins.txt";
pub const FILENAME_DISABLED_PLUGINS: &str = "disabled_plugins.txt";
// AdminLevel_t "enum" (actual Rust enums made code unmanageable, hence the choice)
pub type AdminLevel = i8;
pub const ADMIN_INIT: AdminLevel  = -1;// (UNUSED) Level on connect, tells functions not to use cached level
pub const ADMIN_NO: AdminLevel    = 0;// Not an administrator
pub const ADMIN_YES: AdminLevel   = 1;// Server administrator
pub const ADMIN_OWNER: AdminLevel = 2;// Server owner (applies to a listenserver host player)

#[derive(Debug, Clone, PartialEq)]
pub enum PluginState
{
    Enabled,
    Disabled,
    Removed,
}

#[derive(Debug, Clone)]
pub struct PluginEntry
{
    pub name: String,
    pub script: String,
    pub state: PluginState,
    pub concommandns: String,// optional
    pub adminlevel: AdminLevel,// optional
    pub maps_included: String,// optional
    pub maps_excluded: String,// optional
    pub start: usize,
    pub end: usize,
}
// Plugin methods
impl PluginEntry
{   // Constructor
    pub fn new(name: &str, script: &str) -> Self
    {
        PluginEntry
        {
            name: name.to_string(),
            script: script.to_string(),
            state: PluginState::Enabled,// If we've just created it, then of course it's enabled
            concommandns: String::new(),
            adminlevel: ADMIN_NO,
            maps_included: String::new(),
            maps_excluded: String::new(),
            start: 0,
            end: 0,
        }
    }

    pub fn toggle_state(&mut self)
    {
        self.state = match self.state
        {
            PluginState::Enabled => PluginState::Disabled,
            PluginState::Disabled => PluginState::Enabled,
            PluginState::Removed => PluginState::Removed,
        };
    }
    // !-TODO-!: need to get the path to the svencoop install again to check if the given script is there or not
/*     pub fn validate_plugin_install(&self) -> bool
    {
        false
    }

    pub fn validate_script_install(script: &str) -> bool
    {
        false
    } */
    // New plugin entry, name and script are required minimum fields, returns (key, Plugin) tuple
    // Maybe this should be a Plugin constructor instead?
    pub fn add_plugin(name: &str, script: &str) -> (String, PluginEntry)
    {
        let name_trim = name.trim();
        let script_trim = script.trim();

        if name_trim.is_empty() || script_trim.is_empty()
        {
            println!( "Plugin name and script cannot be empty." );
            return ( String::new(), PluginEntry::new("", "") );
        }

        let key = name_trim.to_string();

        let plugin = PluginEntry
        {
            name: key.clone(),
            script: script_trim.to_string(),
            state: PluginState::Enabled,
            concommandns: String::new(),
            adminlevel: ADMIN_NO,
            maps_included: String::new(),
            maps_excluded: String::new(),
            start: 0,
            end: 0,
        };
        // To-do: actually install the script file to svencoop\scripts\plugins
        ( key, plugin )
    }
    // Copies the file to the game install
    pub fn install_plugin(script: &str, svencoop_dir: &Path) -> io::Result<()>
    {
        let src = PathBuf::from( script );
        // Replace "svencoop" with "svencoop_addon" in the base path
        let parent = svencoop_dir.parent().unwrap_or( svencoop_dir );
        let addon_dir = parent.join( "svencoop_addon" );
        // Destination: svencoop_addon/scripts/plugins/<filename>
        let dst = addon_dir
            .join( "scripts" )
            .join( "plugins" )
            .join( src.file_name().unwrap_or_default() );
        // Ensure the destination directory exists
        if let Some( parent ) = dst.parent()
        {
            fs::create_dir_all( parent )?;
        }
        // Copy the file
        fs::copy( &src, &dst )?;

        Ok(())
    }
    // Returns the plugin entry as a formatted string
    pub fn write_plugin(&self) -> String
    {
        if self.name.trim().is_empty() || self.script.trim().is_empty() || matches!( self.state, PluginState::Removed )
        {
            return String::new();
        }
        // The actual plugin entry format
        let plugin_format =
        r#"
        "plugin"
        {
            "name" "<NAME>"
            "script" "<SCRIPT>"
            "concommandns" "<CONCOMMANDNS>"
            "adminlevel" "<ADMINLEVEL>"
            "maps_included" "<MAPSINCLUDED>"
            "maps_excluded" "<MAPSEXCLUDED>"
        }
        "#;

        plugin_format
            .replace( "<NAME>", &self.name )
            .replace( "<SCRIPT>", &self.script )
            .replace( "<CONCOMMANDNS>", &self.concommandns )
            .replace( "<ADMINLEVEL>", &( self.adminlevel as i32 ).to_string() )
            .replace( "<MAPSINCLUDED>", &self.maps_included )
            .replace( "<MAPSEXCLUDED>", &self.maps_excluded )
    }
}

pub fn load_plugins(text: &str, state: PluginState) -> HashMap<String, PluginEntry>
{
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0usize;
    let mut plugins: HashMap<String, PluginEntry> = HashMap::new();
    let mut unnamed_counter = 0usize;

    while i < lines.len()
    {
        let line = lines[i].trim();

        if line.starts_with( "\"plugin\"" )
        {
            let mut name = String::new();// This field may not be necessary given this is being shoved into a hashmap where the plugin name is the key
            let mut script = String::new();
            let mut concommandns = String::new();
            let mut adminlevel = ADMIN_NO;
            let mut maps_included = String::new();
            let mut maps_excluded = String::new();

            let start = i;
            i += 1; // move past "plugin"

            while i < lines.len() && !lines[i].trim().starts_with( '}' )
            {
                let inner_line = lines[i].trim();

                if inner_line.starts_with( "\"name\"" ) 
                {
                    name = inner_line.split( '"' ).nth( 3 ).unwrap_or( "" ).to_string();
                } 
                else if inner_line.starts_with( "\"script\"" ) 
                {
                    script = inner_line.split( '"' ).nth( 3 ).unwrap_or( "" ).to_string();
                } 
                else if inner_line.starts_with( "\"concommandns\"" ) 
                {
                    concommandns = inner_line.split( '"' ).nth( 3 ).unwrap_or( "" ).to_string();
                }
                else if inner_line.starts_with( "\"adminlevel\"" ) 
                {
                    let level = inner_line.split( '"' ).nth( 3 ).unwrap_or( "0" );
                    adminlevel = level.parse::<i8>().unwrap_or( 0 );
                }
                else if inner_line.starts_with( "\"maps_included\"" )
                {
                    maps_included = inner_line.split( '"' ).nth( 3 ).unwrap_or( "" ).to_string();
                }
                else if inner_line.starts_with( "\"maps_excluded\"" )
                {
                    maps_excluded = inner_line.split( '"' ).nth( 3 ).unwrap_or( "" ).to_string();
                }

                i += 1;
            }

            let end = i;
            // ensure we have a key for the hashmap; if name is empty, generate a unique key
            let key =
            if name.is_empty() 
            {
                let k = format!( "__unnamed_{}", unnamed_counter );
                unnamed_counter += 1;

                k
            } 
            else 
            {
                name.clone()
            };

            let plugin = PluginEntry
            {
                name,
                script,
                state: state.clone(),
                concommandns,
                adminlevel,
                maps_included,
                maps_excluded,
                start,
                end,
            };
            // insert; if duplicate key exists, this will replace the previous entry
            plugins.insert( key, plugin );
        }

        i += 1;
    }

    plugins
}

pub fn save_plugins(plugins: &HashMap<String, PluginEntry>) -> Result<(), io::Error>
{
    let mut enabled_plugins = String::new();
    let mut disabled_plugins = String::new();

    for plugin in plugins.values()
    {
        match plugin.state
        {
            PluginState::Enabled => enabled_plugins.push_str( &plugin.write_plugin() ),
            PluginState::Disabled => disabled_plugins.push_str( &plugin.write_plugin() ),
            PluginState::Removed => (),// ignore removed plugins
        }
    }

    let store = crate::config::read_store()?;
    let path = 
    match store.svencoopdir
    {
        Some( dir ) => PathBuf::from( dir ),
        None => std::env::current_dir().unwrap_or_default(),// ??? Theoretically should never happen.
    };

    fs::write( path.join( FILENAME_PLUGINS ), format!( "\"plugins\"\n{{\n{}}}\n", enabled_plugins), )?;
    fs::write( path.join( FILENAME_DISABLED_PLUGINS ),format!( "\"disabled_plugins\"\n{{\n{}}}\n", disabled_plugins), )?;

    Ok( () )
}
