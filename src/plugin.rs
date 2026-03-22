/*
	Sven Co-op Plugin Manager Version 2.0

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
    env,
    fs,
    io,
    collections::HashMap,
    ops::Not,
    path::{ Path, PathBuf }
};

use num_enum::{ FromPrimitive, IntoPrimitive };

pub const FILENAME_PLUGINS: &str = "default_plugins.txt";
pub const FILENAME_DISABLED_PLUGINS: &str = "disabled_plugins.txt";

pub const CHECKED: &str = "✔";
pub const UNCHECKED: &str = "☐";

#[derive( Debug, Default, Clone, Copy, FromPrimitive, IntoPrimitive )]
#[repr(isize)]
pub enum AdminLevel// straight from AdminLevel_t: https://sven-coop.github.io/AdminLevel_t
{
    Init = -1,// (UNUSED) Level on connect, tells functions not to use cached level

    #[default]
    No,// Not an administrator
    Yes,// Not an administrator
    Owner// Server owner (applies to a listenserver host player)
}

#[derive( Debug, Clone, PartialEq )]
pub enum PluginState
{
    Enabled,
    Disabled,
    Removed
}

impl PluginState
{
    pub fn marker(&self) -> &'static str
    {
        match self
        {
            PluginState::Enabled => CHECKED,
            PluginState::Disabled | PluginState::Removed => UNCHECKED
        }
    }

    pub fn toggle(&self) -> Self
    {
        match self
        {
            PluginState::Disabled => PluginState::Enabled,
            PluginState::Enabled => PluginState::Disabled,
            PluginState::Removed => PluginState::Removed// stays removed.
        }
    }
}

impl Not for PluginState
{
    type Output = PluginState;

    fn not(self) -> Self
    {
        match self
        {
            PluginState::Disabled => PluginState::Enabled,
            PluginState::Enabled => PluginState::Disabled,
            _ => self,
        }
    }
}

#[derive( Default )]
pub struct PluginContext
{
    pub plugins: HashMap<String, PluginEntry>,
    pub selected_plugin_name: Option<String>
}

impl PluginContext
{
    pub fn from_hashmap(plugins: HashMap<String, PluginEntry>) -> Self
    {
        Self
        {
            plugins,
            selected_plugin_name: None,
        }
    }

    pub fn has_plugin(&self, name: &str) -> bool
    {
        self.plugins.contains_key( name )
    }
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
        Self
        {
            name: name.to_string(),
            script: script.to_string(),
            state: PluginState::Enabled,// If we've just created it, then of course it's enabled
            concommandns: String::new(),
            adminlevel: AdminLevel::No,
            maps_included: String::new(),
            maps_excluded: String::new(),
            start: 0,
            end: 0,
        }
    }

    pub fn toggle_state(&mut self)
    {
        self.state = self.state.toggle();
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
    pub fn add_plugin(name: &str, script: &str) -> (String, Self)
    {
        let name_trim = name.trim();
        let script_trim = script.trim();

        if name_trim.is_empty() || script_trim.is_empty()
        {
            println!( "Plugin name and script cannot be empty." );
            return ( String::new(), PluginEntry::new( "", "" ) );
        }

        let key = name_trim.to_string();

        let plugin = PluginEntry
        {
            name: key.clone(),
            script: script_trim.to_string(),
            state: PluginState::Enabled,
            concommandns: String::new(),
            adminlevel: AdminLevel::No,
            maps_included: String::new(),
            maps_excluded: String::new(),
            start: 0,
            end: 0,
        };

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

        Ok( () )
    }
    // Returns the plugin entry as a formatted string
    pub fn write_plugin(&self) -> String
    {
        if self.name.trim().is_empty()
        || self.script.trim().is_empty()
        || matches!( self.state, PluginState::Removed )
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
            "adminlevel" "<ADMINLEVEL>"
            "concommandns" "<CONCOMMANDNS>"
            "maps_included" "<MAPSINCLUDED>"
            "maps_excluded" "<MAPSEXCLUDED>"
        }"#;

        let mut plugin_entry = plugin_format
            .replace( "<NAME>", &self.name )
            .replace( "<SCRIPT>", &self.script )
            .replace( "<ADMINLEVEL>", &( self.adminlevel as i8 )
        .to_string() );
        // Only include optional fields if they are not empty
        for ( placeholder, value, full_line ) in
        [
            ( "<CONCOMMANDNS>", &self.concommandns, r#""concommandns" "<CONCOMMANDNS>""# ),
            ( "<MAPSINCLUDED>", &self.maps_included, r#""maps_included" "<MAPSINCLUDED>""# ),
            ( "<MAPSEXCLUDED>", &self.maps_excluded, r#""maps_excluded" "<MAPSEXCLUDED>""# )
        ]
        {
            plugin_entry =
            match value.trim().is_empty()
            {
                true => plugin_entry.replace( full_line, "" ),// Unfortunately leaves whitespace, but oh well
                false => plugin_entry.replace( placeholder, value )
            };
        }

        plugin_entry
    }
}

pub fn load_plugins(text: &str, state: PluginState) -> HashMap<String, PluginEntry>
{
    let lines: Vec<_> = text.lines().collect();
    let mut i = 0;
    let mut plugins: HashMap<String, PluginEntry> = HashMap::new();
    let mut unnamed_counter: usize = 0;

    while i < lines.len()
    {
        let line = lines[i].trim();

        if line.starts_with( "\"plugin\"" )
        {   // This field may not be necessary given this is being shoved into a hashmap where the plugin name is the key
            let mut name = String::new();
            let mut script = String::new();
            let mut adminlevel = AdminLevel::No;
            let mut concommandns = String::new();
            let mut maps_included = String::new();
            let mut maps_excluded = String::new();

            let start = i;
            i += 1; // move past "plugin"

            while i < lines.len() && let inner_line = lines[i].trim() && !inner_line.trim().starts_with( '}' )
            {
                if inner_line.starts_with( "\"name\"" ) 
                {
                    name = inner_line.split( '"' ).nth( 3 ).unwrap_or( "" ).to_string();
                } 
                else if inner_line.starts_with( "\"script\"" ) 
                {
                    script = inner_line.split( '"' ).nth( 3 ).unwrap_or( "" ).to_string();
                } 
                else if inner_line.starts_with( "\"adminlevel\"" ) 
                {
                    let level = inner_line.split( '"' ).nth( 3 ).unwrap_or( "0" );
                    adminlevel = AdminLevel::from( level.parse::<isize>().unwrap_or( 0 ) );
                }
                else if inner_line.starts_with( "\"concommandns\"" ) 
                {
                    concommandns = inner_line.split( '"' ).nth( 3 ).unwrap_or( "" ).to_string();
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

            let key = // Ensure we have a key for the hashmap; if name is empty, generate a unique key
            match name.is_empty()
            {
                true =>
                {
                    let k = format!( "__unnamed_{}", unnamed_counter );
                    unnamed_counter += 1;

                    k
                }

                false => name.clone()
            };

            let end = i;
            // insert; if duplicate key exists, this will replace the previous entry
            plugins.insert( key, PluginEntry
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
            });
        }

        i += 1;
    }

    plugins
}

pub fn save_plugins(ctx: &PluginContext) -> Result<(), io::Error>
{
    let mut enabled_plugins = String::new();
    let mut disabled_plugins = String::new();

    for plugin in ctx.plugins.values()
    {
        match plugin.state
        {
            PluginState::Enabled => enabled_plugins.push_str( &plugin.write_plugin() ),
            PluginState::Disabled => disabled_plugins.push_str( &plugin.write_plugin() ),
            PluginState::Removed => { }// ignore removed plugins
        }
    }

    let store = crate::config::read_store()?;
    let path = 
    match store.svencoopdir
    {
        Some( dir ) => PathBuf::from( dir ),
        None => env::current_dir().unwrap_or( PathBuf::from( "." ) )
    };

    fs::write( path.join( FILENAME_PLUGINS ), format!( "\"plugins\"\n{{{}}}", enabled_plugins ) )?;
    fs::write( path.join( FILENAME_DISABLED_PLUGINS ),format!( "\"disabled_plugins\"\n{{{}}}", disabled_plugins ) )?;

    Ok( () )
}
