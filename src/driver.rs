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
    env,
    io,
    path::Path
};

use native_windows_gui::
{
    MessageButtons,
    MessageIcons
};

use crate::
{
    APPNAME,
    config,
    gui,
    plugin::{ self, * }
};

pub fn run() -> Result<(), io::Error>
{
    let svencoop_dir =
    match config::init()
    {
        Ok( dir ) =>
        {
            dir
        }

        Err( e ) =>
        {
            gui::window::message_box( "Sven Co-op install Not Found",
                format!( "Could not find a valid Sven Co-op installation.
                    \nReason:\n{}\n\nTry installing {} directly to 'Sven Co-op\\svencoop' and try again.", e, APPNAME ).as_str(), 
                MessageButtons::Ok,
                MessageIcons::Error );

            return Err( e );
        }
    };

    let files =
    [
        (
            svencoop_dir.join( plugin::FILENAME_PLUGINS ),
            PluginState::Enabled,
        ),
        (
            svencoop_dir.join( plugin::FILENAME_DISABLED_PLUGINS ),
            PluginState::Disabled,
        ),
    ];

    let mut plugins = std::collections::HashMap::new();

    for ( path, state ) in files
    {
        match std::fs::read_to_string( path.to_str().unwrap_or_default() )
        {
            Ok( file ) =>
            {
                plugins.extend( load_plugins( &file, state ) );
            }

            Err( e ) =>
            {   // It doesn't exist since we haven't created it yet.
                if path.ends_with( FILENAME_DISABLED_PLUGINS )
                {
                    break;
                }

                gui::window::message_box(
                    "Error reading plugin file",
                    &format!( "The plugin file in '{}' could not be opened.\nReason:\n{}", path.display(), e ),
                    MessageButtons::Ok,
                    MessageIcons::Error );
            }
        }
    }
    // Quick install
    let args: Vec<String> = env::args().collect();

    match args.len()
    {
        n if n > 1 =>
        {
            for file in &args[1..]
            {
                if file.is_empty()
                {
                    continue;
                }

                if !file.ends_with( ".as" )
                {
                    gui::window::message_box( "Invalid File", 
                        format!( "'{}' is not a valid plugin script file.\nPlugin script files end with the '.as' file extension.", file ).as_str(), 
                        MessageButtons::Ok, 
                        MessageIcons::Warning );

                    continue;
                }
                
                let name = Path::new( file ).file_stem().and_then( |s| s.to_str() ).unwrap_or( "" );
                
                if name.is_empty()
                {
                    gui::window::message_box( "Error",
                        format!( "Failed to extract plugin name from file '{}'", file ).as_str(), 
                        MessageButtons::Ok, 
                        MessageIcons::Error );

                    continue;
                }
                // Plugin already exists
                if plugins.contains_key( name )
                {
                    gui::window::message_box( "Info",
                        format!( "The plugin script '{}' is already installed.\n\n
                        To disable or remove this plugin, launch {} and do this manually.", file, APPNAME ).as_str(), 
                        MessageButtons::Ok, 
                        MessageIcons::Info );

                    continue;
                }
                
                let new_plugin_entry = PluginEntry::add_plugin( name, file );
                plugins.insert( new_plugin_entry.0, new_plugin_entry.1 );
                // Install the script file to the game first
                match PluginEntry::install_plugin( file, &svencoop_dir )
                {
                    Ok( _ ) => { }// Redundant, for now

                    Err( e ) =>// Something bad happened, skip this install
                    {
                        gui::window::message_box( "Installation Failed",
                            format!( "Failed to install plugin '{}' from script file '{}'.\nError code {}", name, file, e ).as_str(), 
                            MessageButtons::Ok, 
                            MessageIcons::Error );

                        continue;
                    }
                }
                // Update plugin config file
                match save_plugins( &plugins )
                {
                    Ok( _ ) =>
                    {
                        gui::window::message_box( "Plugin Installed",
                            format!( "Plugin '{}' installed from script file '{}'.", name, file ).as_str(), 
                            MessageButtons::Ok, 
                            MessageIcons::Info );
                    }

                    Err( e ) =>
                    {
                        gui::window::message_box( "Installation Failed",
                            format!( "Failed to install plugin '{}' from script file '{}'.\nError code {}", name, file, e ).as_str(), 
                            MessageButtons::Ok, 
                            MessageIcons::Error );
                    }
                }
            }
        }

        _ =>// Nothing was dragged, launch menu
        {
            gui::events::GUI( plugins );
        }
    }

    Ok( () )
}
