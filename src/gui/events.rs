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

use nwg::*;
use super::window::*;
use std::
{
    collections::HashMap,
    rc::Rc
};

use crate::
{
    utils::open_file_dialog,
    APPNAME, plugin::
    {
        self,
        AdminLevel,
        PluginEntry,
        save_plugins
    }
};

pub fn GUI(plugins: HashMap<String, PluginEntry>)
{   // Initialise the window
    if let Err( e ) = init()
    {
        message_box( "FATAL ERROR",
        format!( "Failed to initialise window.\nError code: {}", e ).as_str(),
        MessageButtons::Ok,
        MessageIcons::Error );

        panic!( "{} panicked: {}", APPNAME, e );
    }

    let gui = super::window::build_main_window( plugins );
    let cloned_gui = Rc::clone( &gui );
    let window_handle = gui.borrow().window.handle;

    bind_event_handler( &window_handle, &window_handle, move |evt, _evt_data, handle|
    {
        let gui = cloned_gui.borrow();

        match evt
        {
            Event::OnListBoxSelect =>
            {   // BSP whitelist listbox → toggle checkmark
                if handle == gui.lst_plugins.handle
                {
                    let selected_name = gui.lst_plugins.selection_string().unwrap_or_default();
                    // remove checkmark prefix then trim leading whitespace
                    let selected_name = selected_name
                        .trim_start_matches( CHECKED )
                        .trim_start_matches( UNCHECKED )
                        .trim_start();

                    let plugins_ref = gui.plugins.borrow();
                    // lookup safely (avoid panic if key missing)
                    if let Some( selected_plugin ) = plugins_ref.get( selected_name )
                    {   // Update fields based on selected plugin
                        gui.txt_name.set_text( &selected_plugin.name );
                        gui.txt_script.set_text( &selected_plugin.script );
                        gui.txt_concommandns.set_text( &selected_plugin.concommandns );
                        gui.cb_adminlevel.set_selection(
                        match selected_plugin.adminlevel
                        {
                            AdminLevel::AdminNo => Some( 0 ),
                            AdminLevel::AdminYes => Some( 1 ),
                            AdminLevel::AdminOwner => Some( 2 ),
                            _ => Some( 0 ),
                        });

                        gui.txt_maps_included.set_text( &selected_plugin.maps_included.join( "\n" ) );
                        gui.txt_maps_excluded.set_text( &selected_plugin.maps_excluded.join( "\n" ) );
                        gui.chk_enabled.set_check_state(
                        match selected_plugin.state
                        {
                            plugin::PluginState::Enabled => CheckBoxState::Checked,
                            plugin::PluginState::Disabled => CheckBoxState::Unchecked,
                            _ => CheckBoxState::Unchecked,
                        });
                    }
                }
            }
            // Button clicks
            Event::OnButtonClick
            if handle == gui.chk_enabled.handle => 
            {   // Toggle enabled/disabled state
                // Get raw visible string, strip checkmark prefix to lookup the plugin
                let selected_name = gui.lst_plugins.selection_string().unwrap_or_default();
                let selected_name = selected_name
                    .trim_start_matches( CHECKED )
                    .trim_start_matches( UNCHECKED )
                    .trim_start();
                // Update plugin state in-place
                {
                    let mut plugins_ref = gui.plugins.borrow_mut();
                    if let Some( selected_plugin ) = plugins_ref.get_mut( selected_name )
                    {
                        selected_plugin.state =
                        match gui.chk_enabled.check_state()
                        {
                            CheckBoxState::Checked => plugin::PluginState::Enabled,
                            CheckBoxState::Unchecked => plugin::PluginState::Disabled,
                            _ => selected_plugin.state.clone(),
                        };
                    }
                }
                // Rebuild and replace the ListBox collection so the UI shows the new checkmark
                {
                    let plugins_ref = gui.plugins.borrow();
                    let mut rows: Vec<(String, bool)> = plugins_ref
                        .values()
                        .map( |p| ( p.name.clone(), p.state == plugin::PluginState::Enabled ) )
                    .collect();
                    // sort by name case-insensitively
                    rows.sort_unstable_by_key( |(name, _)| name.to_ascii_lowercase() );
                    // format with prefix
                    let new_collection: Vec<String> = rows
                        .into_iter()
                        .map( |(name, enabled)|
                        {
                            format!( "{} {}", if enabled { CHECKED } else { UNCHECKED }, name )
                        })
                    .collect();
                    // Replace the listbox collection
                    gui.lst_plugins.set_collection( new_collection );
                }
            }

            Event::OnButtonClick
            if handle == gui.cb_adminlevel.handle =>
            {   // Prevent the drop-down from opening when a plugin is not selected
                if gui.lst_plugins.selection_string().is_none()
                {   // !-BUG-!: Doesn't appear to work as intended. Box is still opening.
                    gui.cb_adminlevel.set_selection( None );
                }
            }
            // Add button clicked
            Event::OnButtonClick
            if handle == gui.btn_add.handle =>
            {   // Add new plugin
                let script_path = 
                match open_file_dialog( &gui.window )
                {
                    Some( p ) => p,
                    None => return,// Didn't get a file
                };

                let name = script_path
                    .file_stem()
                    .and_then( |s| s.to_str() )
                    .unwrap_or( "" )
                .to_string();

                if name.trim().is_empty()
                {
                    message_box( "Invalid Plugin File",
                    "Failed to extract plugin name from the selected file.",
                    MessageButtons::Ok,
                    MessageIcons::Error );

                    return;
                }

                let mut plugins_ref = gui.plugins.borrow_mut();

                if plugins_ref.contains_key( name.as_str() )
                {
                    message_box( "Plugin Already Exists", 
                    "A plugin with this name already exists. Please choose a different name.",
                    MessageButtons::Ok,
                    MessageIcons::Warning );

                    return;
                }

                if
                    let Ok( conf ) = crate::config::read_store() &&
                    let Some( svencoopdir ) = conf.svencoopdir &&
                    let Err( e ) = PluginEntry::install_plugin( &script_path.to_string_lossy(), &std::path::PathBuf::from( svencoopdir ) )
                {
                    message_box( "Install Error", 
                    format!( "Failed to install plugin {}.\nReason:\n{}\n\nYou will need to manually add this file to the game.", name, e ).as_str(),
                    MessageButtons::Ok,
                    MessageIcons::Error );
                }
                // Write may have failed, but we can still update the plugins file
                let script_file = script_path
                    .file_name()
                    .and_then( |ostr| ostr.to_str() )
                    .expect( "script_path has UTF-8 filename" )// Shouldn't panic, I hope?
                    .trim_end_matches( ".as" );

                let new_plugin_entry = PluginEntry::add_plugin( name.as_str(), script_file );
                plugins_ref.insert( new_plugin_entry.0.clone(), new_plugin_entry.1 );
                // Finally, update listbox
                let mut list_of_plugins: Vec<String> = plugins_ref
                    .values()
                    .map( |p|
                    {
                        format!( "{}{}",
                        if p.state == plugin::PluginState::Enabled
                        {
                            CHECKED
                        }
                        else
                        {
                            UNCHECKED
                        },
                        p.name )
                    })
                .collect();

                list_of_plugins.sort();
                gui.lst_plugins.set_collection( list_of_plugins );
            }
            // Remove button clicked
            Event::OnButtonClick
            if handle == gui.btn_remove.handle =>
            {   // Remove selected plugin
                let selected_name = gui.lst_plugins.selection_string().unwrap_or_default();// Not sure how it can fail here
                let selected_name = selected_name.trim_start_matches( CHECKED ).trim_start_matches( UNCHECKED ).trim_start();

                if selected_name.is_empty()
                {
                    message_box( "No Plugin Selected",
                    "Please select a plugin from the list to remove.",
                    MessageButtons::Ok,
                    MessageIcons::Warning );

                    return;
                }

                if message_box( "Confirm Remove",
                format!( "Are you sure you want to remove the plugin '{}'", selected_name ).as_str(),
                MessageButtons::YesNo, 
                MessageIcons::Question ) == MessageChoice::No
                {
                    return;
                }

                let mut plugins_ref = gui.plugins.borrow_mut();
  
                if plugins_ref.remove( selected_name ).is_some()
                {   // Update listbox
                    let mut list_of_plugins: Vec<String> = plugins_ref
                        .values()
                        .map( |p|
                        {
                            format!( "{}{}", if p.state == plugin::PluginState::Enabled
                            {
                                CHECKED
                            }
                            else
                            {
                                UNCHECKED
                            },
                            p.name )
                        })
                    .collect();

                    list_of_plugins.sort();
                    gui.lst_plugins.set_collection( list_of_plugins );
                    // Clear fields
                    gui.txt_name.set_text( "" );
                    gui.txt_script.set_text( "" );
                    gui.txt_concommandns.set_text( "" );
                    gui.cb_adminlevel.set_selection( None );
                    gui.txt_maps_included.set_text( "" );
                    gui.txt_maps_excluded.set_text( "" );
                    gui.chk_enabled.set_check_state( CheckBoxState::Unchecked );
                }
            }
            // Apply button clicked
            Event::OnButtonClick
            if handle == gui.btn_apply.handle =>
            {   // Apply changes to selected plugin
                let Some( selected_name ) = gui.lst_plugins.selection_string()
                else
                {   // None selected
                    message_box( "No Plugin Selected", 
                    "Please select a plugin from the list to apply changes.", 
                    MessageButtons::Ok, 
                    MessageIcons::Warning );

                    return;
                };

                let mut plugins_ref = gui.plugins.borrow_mut();
                if let Some( selected_plugin ) = plugins_ref.get_mut( selected_name.as_str() )
                {
                    selected_plugin.name = gui.txt_name.text();
                    selected_plugin.script = gui.txt_script.text();
                    selected_plugin.concommandns = gui.txt_concommandns.text();
                    selected_plugin.adminlevel =
                    match gui.cb_adminlevel.selection()
                    {
                        Some( 0 ) => AdminLevel::AdminNo,
                        Some( 1 ) => AdminLevel::AdminYes,
                        Some( 2 ) => AdminLevel::AdminOwner,
                        _ => AdminLevel::AdminNo,
                    };

                    selected_plugin.maps_included = gui.txt_maps_included.text().lines().map( | s | s.to_string() ).collect();
                    selected_plugin.maps_excluded = gui.txt_maps_excluded.text().lines().map( | s | s.to_string() ).collect();
                }
            }
            // Save button clicked
            Event::OnButtonClick
            if handle == gui.btn_save.handle =>
            {
                match save_plugins( &gui.plugins.borrow() )
                {
                    Ok( _ ) =>
                    {
                        message_box( "Success",
                        "Plugins saved.",
                        MessageButtons::Ok,
                        MessageIcons::Info );

                        stop_thread_dispatch();
                    },
                    Err( e ) =>
                    {
                        if message_box( "Error", 
                        format!( "Failed to save changes to plugin.\nReason:{}", e ).as_str(),
                        MessageButtons::RetryCancel, 
                        MessageIcons::Error ) == MessageChoice::Cancel
                        {
                            stop_thread_dispatch();
                        }
                    }
                }
            }
            // Help button clicked
            Event::OnButtonClick
            if handle == gui.btn_help.handle =>
            {
                message_box( "Help", 
                    HELP_INFO, 
                    MessageButtons::Ok, 
                    MessageIcons::Question );
            }
            // Window close event
            Event::OnWindowClose =>
            {
                match message_box( "Confirm Save",
                "Save changes?",
                MessageButtons::YesNo,
                MessageIcons::Question )
                {
                    MessageChoice::Yes =>
                    {   // Save plugins
                        match save_plugins( &gui.plugins.borrow() )
                        {
                            Ok( _ ) => { },
                            Err( e ) =>
                            {
                                message_box( "Error",
                                format!( "Failed to save changes.\nReason:{}", e ).as_str(),
                                MessageButtons::Ok,
                                MessageIcons::Error );
                            }
                        }
                    }
                    // Do nothing and exit
                    _ => { }
                }

                stop_thread_dispatch();
            }

            _ => { }
        }
    });
    // Toss in heap so events can be handled as we go
    Box::leak( Box::new( gui ) );
    dispatch_thread_events();
}
