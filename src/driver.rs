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
    collections,
    env,
    fs,
    io,
    path::Path
};

use crate::
{
    APPNAME,
    app::
    {
        launch_gui,
        popup,
        PopupButtons
    },
    config,
    plugin::
    {
        load_plugins,
        save_plugins,
        PluginContext,
        PluginEntry,
        PluginState,
        FILENAME_DISABLED_PLUGINS,
        FILENAME_PLUGINS,
    }
};

pub fn run() -> Result<(), io::Error>
{
    let svencoop_dir =
    match config::init()
    {
        Ok( dir ) => dir,
        Err( e ) =>
        {
            popup(
                "Sven Co-op install Not Found",
                &format!(
                    "Could not find a valid Sven Co-op installation.\nReason:\n{}\n\n\
                    Try installing {} directly to 'Sven Co-op\\svencoop' and try again.",
                    e, APPNAME
                ),
                "❌",
                PopupButtons::None
            );

            return Err( e );
        }
    };

    let files =
    [
        ( svencoop_dir.join( FILENAME_PLUGINS ), PluginState::Enabled ),
        ( svencoop_dir.join( FILENAME_DISABLED_PLUGINS ), PluginState::Disabled )
    ];

    let mut plugins = collections::HashMap::new();

    for ( path, state ) in files
    {
        match fs::read_to_string( path.to_str().unwrap_or_default() )
        {
            Ok( file ) => plugins.extend( load_plugins( &file, state ) ),

            Err( e ) =>
            {
                if path.ends_with( FILENAME_DISABLED_PLUGINS )
                {
                    break;
                }

                popup(
                    "Error reading plugin file",
                    &format!(
                        "The plugin file in '{}' could not be opened.\nReason:\n{}",
                        path.display(),
                        e
                    ),
                    "❌",
                    PopupButtons::Ok );
            }
        }
    }

    let args: Vec<_> = env::args().collect();

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
                    popup("Invalid File",
                          &format!( "'{}' is not a valid plugin script file.\n\
                        Plugin script files end with the '.as' file extension.", file ),
                          "⚠️", PopupButtons::Ok );

                    continue;
                }

                let name = Path::new( file )
                    .file_stem()
                    .and_then( |s| s.to_str() )
                .unwrap_or(".");

                if name.is_empty()
                {
                    popup(
                        "Error",
                        &format!( "Failed to extract plugin name from file '{}'", file ),
                        "❌",
                        PopupButtons::Ok );

                    continue;
                }

                if plugins.clone().contains_key( name )
                {
                    popup("Info",
                          &format!("The plugin script '{}' is already installed.\n\n\
                        To disable or remove this plugin, launch {} and do this manually.", file, APPNAME),
                          "ℹ️", PopupButtons::Ok );

                    continue;
                }

                let new_plugin_entry = PluginEntry::add_plugin( name, file );
                plugins.insert( new_plugin_entry.0, new_plugin_entry.1 );

                match PluginEntry::install_plugin( file, &svencoop_dir )
                {
                    Ok( () ) => {}
                    Err( e ) =>
                    {
                        popup("Installation Failed",
                              &format!("Failed to install plugin '{}' from script file '{}'.\
                            \nError code {}", name, file, e),
                              "❌", PopupButtons::Ok );

                        continue;
                    }
                }

                match save_plugins( &PluginContext::from_hashmap( plugins.clone() ) )
                {
                    Ok(()) =>
                    {
                        popup(
                            "Plugin Installed",
                            &format!("Plugin '{}' installed from script file '{}'.", name, file),
                            "ℹ️",
                            PopupButtons::Ok );
                    }

                    Err( e ) =>
                    {
                        popup("Installation Failed",
                              &format!( "Failed to install plugin '{}' from script file '{}'.\
                            \nError code {}", name, file, e ),
                              "❌", PopupButtons::Ok );
                    }
                }
            }
        }

        _ =>// Nothing was dragged, launch gui
        {
            if let Err( e ) = launch_gui( PluginContext::from_hashmap( plugins ) )
            {
                popup(
                    "Error",
                    &format!( "Failed to launch window.\nReason: {e}" ),
                    "❌",
                    PopupButtons::None );

                return Err( io::Error::other( e ) );
            }
        }
    }

    Ok(())
}
