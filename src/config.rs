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
    let st: Config = toml::from_str( &s )
        .map_err( |e| io::Error::new( io::ErrorKind::InvalidData, e ) )?;

    Ok( st )
}

fn write_store(st: &Config) -> Result<(), io::Error>
{
    fs::create_dir_all( appdata_base() )?;
    let s = toml::to_string_pretty( st )
        .map_err( |e| io::Error::new( io::ErrorKind::Other, e ) )?;

    let p = config_path();
    let tmp = p.with_extension( "toml.tmp" );

    fs::write( &tmp, s.as_bytes() )?;
    fs::rename( &tmp, &p )?;
    
    Ok(())
}
// Last ditch effort to get the path, assuming user installed the app alongside default_plugins.txt
pub fn failover() -> Option<PathBuf>
{   // Get the full path to the running exe
    let exe_path = env::current_exe().ok()?;
    // Get its parent directory
    let exe_dir = exe_path.parent()?;
    // Build the failover path: exe_dir + FILENAME_PLUGINS
    let failover_path = exe_dir.join( crate::plugin::FILENAME_PLUGINS );
    //println!("Using failover path: '{}'", failover_path.display());
    if failover_path.is_file()
    {
        Some( failover_path )
    }
    else
    {
        None
    }
}
// Initialise: check TOML, otherwise discover svencoop.exe and seed plugin files.
// Returns the svencoop folder path.
pub fn init() -> io::Result<PathBuf>
{
    if let Ok( store ) = read_store()
    {
        if let Some( dir ) = store.svencoopdir
        {
            return Ok( PathBuf::from( dir ) );
        }
    }
    // Case 2: discover svencoop.exe
    let splash = crate::gui::window::show_wait_splash();
    let exe_path = crate::utils::search_drives( "svencoop.exe" );
    splash.close();
    let svencoop_dir =
    if exe_path.exists()
    {
        exe_path.with_file_name( "svencoop" )
    }
    else
    {   // Last resort: look for default plugins file next to the application
        failover().ok_or_else( || io::Error::new( io::ErrorKind::NotFound, "svencoop.exe not found" ) )?
    };
    // Ensure plugin files exist
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
    let mut store = Config::default();
    store.svencoopdir = Some( svencoop_dir.to_string_lossy().into_owned() );
    let _ = write_store( &store );

    Ok( svencoop_dir )
}
