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
    fs,
    io,
    path::PathBuf
};

use crate::gui::window::message_box;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Config
{
    pub svencoopdir: Option<String>,
}

fn appdata_base() -> PathBuf 
{
    if let Ok( local ) = env::var( "LOCALAPPDATA" ) 
    {
        PathBuf::from( local ).join( crate::APPNAME )
    } 
    else if let Ok( appdata ) = env::var( "APPDATA" ) 
    {
        PathBuf::from( appdata ).join( crate::APPNAME )
    } 
    else 
    {
        std::env::current_dir().unwrap().join( crate::APPNAME )
    }
}

fn config_path() -> PathBuf
{
    appdata_base().join( format!( "{}.toml", crate::APPNAME ) )
}

pub fn read_store() -> Result<Config, io::Error>
{
    let p = config_path();
    
    if !p.exists()
    { 
        return Ok( Config::default() );
    }

    let s = fs::read_to_string( p )?;
    let conf: Config = toml::from_str( &s ).map_err( io::Error::other )?;
    
    Ok( conf )
}

fn write_store(st: &Config) -> Result<(), io::Error>
{
    fs::create_dir_all( appdata_base() )?;

    let p = config_path();
    let tmp = p.with_extension( "toml.tmp" );
    let s = toml::to_string_pretty( st ).map_err( io::Error::other )?;

    fs::write( &tmp, s.as_bytes() )?;
    fs::rename( &tmp, &p )?;
    
    Ok(())
}
// Initialise: check TOML, otherwise discover svencoop.exe and seed plugin files.
// Returns the svencoop folder path.
fn search_install() -> PathBuf
{
    let exe_path = crate::utils::search_drives( "svencoop.exe" );

    if exe_path.exists()
    {
        exe_path.with_file_name( "svencoop" )
    }
    else
    {
        PathBuf::new()
    }
}

pub fn init() -> io::Result<PathBuf>
{   // Load config first if its exists
    if let Ok( store ) = read_store() && let Some( dir ) = store.svencoopdir
    {
        return Ok( PathBuf::from( dir ) );
    }
    // Initial setup
    let splash = crate::gui::window::show_wait_splash();
    let exe_path = env::current_dir().unwrap_or_default();// If the plugin file exists in the current dir, just use that.
    let svencoop_dir =
    match exe_path.join( crate::plugin::FILENAME_PLUGINS ).exists()
    {
        true => exe_path,
        false => search_install(),
    };

    if !svencoop_dir.exists()
    {
        splash.close();
        return Err( io::Error::new( io::ErrorKind::NotFound, "No directory exists." ) );
    }
    // Ensure plugin files exist - redundant for default_plugins.txt if the app was directly installed
    let enabled_file = svencoop_dir.join( crate::plugin::FILENAME_PLUGINS );
    let disabled_file = svencoop_dir.join( crate::plugin::FILENAME_DISABLED_PLUGINS );

    if !enabled_file.exists()
    { 
        fs::write( &enabled_file, b"" )?;
    }

    if !disabled_file.exists()
    {
        fs::write( &disabled_file, b"" )?;
    }
    // Save folder path into TOML
    if let Err( e ) = write_store( &Config { svencoopdir: Some( svencoop_dir.to_string_lossy().into_owned() ) } )
    {
        message_box( "Error",
            format!( "Failed to save config.\nReason: {}", e ).as_str(),
            native_windows_gui::MessageButtons::Ok,
            native_windows_gui::MessageIcons::Error );
    };
    
    splash.close();
    Ok( svencoop_dir )
}
