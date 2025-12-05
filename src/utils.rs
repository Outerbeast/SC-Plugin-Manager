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
extern crate native_windows_gui as nwg;
use std::
{
    fs, 
    path::
    {
        Path,
        PathBuf
    }
};
// Is this function necessary?
pub fn open_file(path: &str) -> Result<String, std::io::Error>
{
    fs::read_to_string( path )
}

pub fn search_drives(file_name: &str) -> PathBuf
{
    if file_name.trim().is_empty()
    {
        return PathBuf::new();
    }

    let mut results: Vec<PathBuf> = Vec::new();

    for drive in ["A:/", "B:/", "C:/", "D:/", "E:/", "F:/",]
    {
        let root = Path::new( drive );

        if root.exists() && root.is_dir()
        {
            let walker = 
            walkdir::WalkDir::new( root )
                .max_depth( 10 )
                .into_iter()
                .filter_entry( |e|
                {
                    let name = e.file_name().to_string_lossy();
                    !name.eq_ignore_ascii_case( "$Recycle.Bin" )
                })
                .filter_map( Result::ok )
                .filter( |e| e.file_name().to_string_lossy().eq_ignore_ascii_case( file_name ) );

            for entry in walker
            {
                results.push( entry.path().to_path_buf() );
            }
        }
    }

    match results.is_empty()
    {
        true => PathBuf::new(),
        false => results[0].clone(),
    }
}
/// Show a one-off Open file dialog parented to `parent`.
/// Returns Some(path) when the user selects a file, None when cancelled.
pub fn open_file_dialog(parent: &nwg::Window) -> Option<PathBuf>
{
    let mut dlg = nwg::FileDialog::default();
    nwg::FileDialog::builder()
        .title( "Select plugin file" )
        .action( nwg::FileDialogAction::Open )
        .multiselect( false )
        .filters( "Plugins (*.as)|All files (*.*)" )
        .build( &mut dlg )
    .unwrap_or_default();
    // run(parent) blocks until the dialog closes; returns bool
    if dlg.run( Some( parent ) )
    {
        match dlg.get_selected_item()
        {
            Ok( os ) => Some( PathBuf::from( os ) ),
            Err( _ ) => None,
        }
    }
    else 
    {
        None
    }
}
