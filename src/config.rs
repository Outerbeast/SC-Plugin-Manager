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
    path::PathBuf,
    sync::OnceLock
};

use crate::
{
    APPNAME, plugin::
    {
        FILENAME_DISABLED_PLUGINS,
        FILENAME_PLUGINS
    },
    utils
};

pub static SVENCOOP_PATH: OnceLock<PathBuf> = OnceLock::new();
// struct only for housing serialised data
#[derive( Debug, Default, serde::Serialize, serde::Deserialize )]
pub struct Config
{
    pub svencoopdir: Option<String>
}

fn appdata_base() -> PathBuf 
{
    #[cfg(target_os = "windows")]
    {
        if let Ok( local ) = env::var( "LOCALAPPDATA" )
        {
            return PathBuf::from( local ).join( APPNAME );
        }

        if let Ok( appdata ) = env::var( "APPDATA" )
        {
            return PathBuf::from( appdata ).join( APPNAME );
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some( config ) = env::var( "XDG_CONFIG_HOME" )
            .ok()
            .map( PathBuf::from )
            .filter( |p| p.exists() )
        {
            return config.join( APPNAME );
        }

        if let Some( home ) = dirs::home_dir()
        {
            return home.join( ".config" ).join( APPNAME );
        }
    }

    env::current_dir().unwrap_or( PathBuf::from( "." ) ).join( APPNAME )
}

fn config_path() -> PathBuf
{
    appdata_base().join( format!( "{}.toml", APPNAME ) )
}

pub fn read_store() -> Result<Config, io::Error>
{
    match fs::read_to_string( config_path() )
    {
        Ok( s ) => toml::from_str( &s ).map_err( io::Error::other ),
        // Forgot why I returned Config::default() here.
        Err( e ) if e.kind() == io::ErrorKind::NotFound => Ok( Config::default() ),
        Err( e ) => Err( e )
    }
}

fn write_store(st: &Config) -> Result<(), io::Error>
{
    fs::create_dir_all( appdata_base() )?;

    let p = config_path();
    let buf = p.with_extension( "toml.tmp" );
    let s = toml::to_string_pretty( st ).map_err( io::Error::other )?;

    fs::write( &buf, s.as_bytes() )?;
    fs::rename( &buf, &p )?;
    
    Ok(())
}
// Returns the path to the plugins file
pub fn init() -> Result<PathBuf, io::Error>
{   // Load config first if its exists
    if let Ok( store ) = read_store() && let Some( dir ) = store.svencoopdir
    {
        return Ok( PathBuf::from( dir ) );
    }
    // Initial setup
    #[cfg( target_os = "windows" )] crate::utils::open_terminal();
    println!( "Initial setup, please wait..." );

    let exe_path = env::current_dir().unwrap_or( PathBuf::from( "." ) );// If the plugin file exists in the current dir, just use that.
    let svencoop_dir =
    match exe_path.join( FILENAME_PLUGINS ).exists()
    {
        true => exe_path,
        false =>// Doesn't exist, look for it
        {
            utils::search_drives( "sven-coop.fgd" )
                .unwrap_or( PathBuf::from( "." ) )
                .with_extension( "" )// drops the extension
        }
    };

    if !svencoop_dir.exists()
    {
        let s_err = "No directory to svencoop exists.";
        eprintln!( "{}", s_err );
        #[cfg( target_os = "windows" )] crate::utils::close_terminal();

        return Err( io::Error::new( io::ErrorKind::NotFound, s_err ) );
    }
    // Ensure plugin files exist - redundant for default_plugins.txt if the app was directly installed
    let enabled_file = svencoop_dir.join( FILENAME_PLUGINS );
    let disabled_file = svencoop_dir.join( FILENAME_DISABLED_PLUGINS );
    
    if !enabled_file.exists()
    {
        fs::write( &enabled_file, b"" )?;
    }

    if !disabled_file.exists()
    {
        fs::write( &disabled_file, b"" )?;
    }
    // Save folder path into TOML
    write_store( &Config { svencoopdir: Some( svencoop_dir.to_string_lossy().into_owned() ) } )?;
    println!( "Sven Co-op path found: {}", svencoop_dir.to_string_lossy() );
    #[cfg( target_os = "windows" )] crate::utils::close_terminal();

    Ok( svencoop_dir )
}
