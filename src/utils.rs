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
use std::path::PathBuf;

#[macro_export]
macro_rules! alloc_shared
{
    ( $value:expr ) =>
    {
        std::rc::Rc::new( std::cell::RefCell::new( $value ) )
    };
}
#[macro_export]
macro_rules! alloc_locked
{
    ($value:expr) =>
    {
        std::sync::Arc::new( std::sync::Mutex::new( $value ) )
    };
}

#[cfg(target_os = "windows")]
unsafe extern "system"
{
    fn AllocConsole() -> i32;
    fn FreeConsole() -> i32;
}

#[cfg(target_os = "windows")]
pub fn open_terminal()
{
    unsafe
    {
        AllocConsole();
    }
}

#[cfg(target_os = "windows")]
pub fn close_terminal()
{
    unsafe
    {
        FreeConsole();
    }
}
// Searches all drives for a specific filename, returns the path to that file
pub fn search_drives(file_name: &str) -> Option<PathBuf>
{
    if file_name.trim().is_empty()
    {
        return None;
    }

    #[cfg(target_os = "windows")]
    {
        for d in 'A'..='Z'
        {
            let drive = format!("{}:/", d);
            let root = std::path::Path::new( &drive );

            if !root.exists() || !root.is_dir()
            {
                continue;
            }

            let mut walker = walkdir::WalkDir::new( root )
                .max_depth( 12 )
                .into_iter()
                .filter_entry( |e|
                {
                    let name = e.file_name().to_string_lossy();
                    !name.eq_ignore_ascii_case( "$Recycle.Bin" )
                })
                .filter_map(Result::ok)
                .filter( |e|
                {
                    e.file_name().to_string_lossy().eq_ignore_ascii_case( file_name )
                });

            if let Some( entry ) = walker.next()
            {
                return Some( entry.path().to_path_buf() );
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let search_paths: Vec<PathBuf> = vec!
        [
            PathBuf::from( "/" ),
            dirs::home_dir()
                .map( |p| p.join( ".steam" ).join( "steam" ) )
                .unwrap_or_default(),
            PathBuf::from( "/mnt" ),
            PathBuf::from( "/opt" ),
            PathBuf::from( "/usr" ).join( "games" ),
        ];

        for root in search_paths
        {
            if !root.exists() || !root.is_dir()
            {
                continue;
            }

            let mut walker = walkdir::WalkDir::new( &root )
                .max_depth( 12 )
                .into_iter()
                .filter_entry( |e|
                {
                    let name = e.file_name().to_string_lossy();
                    !name.starts_with('.')
                })
                .filter_map( Result::ok )
                .filter( |e| e.file_name().to_string_lossy() == file_name );

            if let Some(entry) = walker.next()
            {
                return Some( entry.path().to_path_buf() );
            }
        }
    }

    None
}
