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
    cell::RefCell,
    collections::HashMap,
    path::PathBuf,
    rc::Rc
};

use slint::
{
    ComponentHandle,
    VecModel,
    StandardListViewItem,
    ModelRc,
    PlatformError
};

use crate::
{
    config,
    plugin::
    {
        PluginEntry,
        PluginState,
        AdminLevel,
        save_plugins,
        CHECKED,
        UNCHECKED,
    }
};

use super::
{
    popup,
    AppWindow,
    PluginContext,
    PopupButtons,
};

pub fn make_plugin_list(plugins: &HashMap<String, PluginEntry>) -> Vec<StandardListViewItem>
{
    let mut list: Vec<(String, bool)> = plugins
        .values()
        .map( |p| ( p.name.clone(), p.state == PluginState::Enabled ) )
    .collect();
    
    list.sort_unstable_by_key( |(name, _)| name.to_ascii_lowercase() );
    
    list.into_iter().map( |(name, enabled)|
    {
        let display = format!( "{} {}", if enabled { CHECKED } else { UNCHECKED }, name );
        StandardListViewItem::from( display.as_str() )
    })
    .collect()
}

pub(crate) fn refresh_plugin_list(app: &AppWindow, plugin_data: &Rc<RefCell<PluginContext>>)
{
    app.set_plugin_list( ModelRc::new( VecModel::from( make_plugin_list( &plugin_data.borrow().plugins ) ) ) );
}
// Event handlers for UI
pub(crate) fn on_plugin_selected(index: i32, app: &AppWindow, plugin_data: &Rc<RefCell<PluginContext>>)
{
    let data = plugin_data.borrow();
    let mut sorted_plugins: Vec<_> = data.plugins.values().collect::<Vec<_>>();
    sorted_plugins.sort_unstable_by_key( |p| p.name.to_ascii_lowercase() );
    
    if let Some( plugin ) = sorted_plugins.get( index as usize )
    {
        let name = plugin.name.clone();
        let script = plugin.script.clone();
        let concommandns = plugin.concommandns.clone();
        let adminlevel = plugin.adminlevel as i32;
        let maps_included = plugin.maps_included.clone();
        let maps_excluded = plugin.maps_excluded.clone();
        let enabled = plugin.state == PluginState::Enabled;
        
        drop( data );
        
        let mut data = plugin_data.borrow_mut();
        data.selected_plugin_name = Some( name.clone() );
        
        app.set_txt_name( name.into() );
        app.set_txt_script( script.into() );
        app.set_txt_concommandns( concommandns.into() );
        app.set_cb_adminlevel( adminlevel );
        app.set_txt_maps_included( maps_included.into() );
        app.set_txt_maps_excluded( maps_excluded.into() );
        app.set_chk_enabled( enabled );
    }
}

pub(crate) fn on_add_clicked(app: &AppWindow, plugin_data: &Rc<RefCell<PluginContext>>)
{
    let config =
    match config::read_store()
    {
        Ok( cfg ) => cfg,
        Err( e ) =>
        {
            popup("Error reading config",
                  &format!( "Failed to obtain plugin config\nReason: {}", e ),
                  "❌", PopupButtons::Ok );
            return;
        }
    };

    if let Some( svencoopdir ) = config.svencoopdir
    && let Some( path ) = rfd::FileDialog::new()
        .add_filter( "Plugin scripts", &["as"] )
        .add_filter( "All files", &["*"] )
    .pick_file()
    {
        let name = path.file_stem()
            .and_then( |s| s.to_str() )
            .unwrap_or( "" )
        .to_string();

        if name.trim().is_empty()
        {
            popup("Invalid Plugin File",
                  "Failed to extract plugin name from the selected file.",
                  "❌", PopupButtons::Ok );
            return;
        }

        let mut data = plugin_data.borrow_mut();

        if data.plugins.contains_key( &name )
        {
            popup("Plugin Already Exists", "A plugin with this name already exists. Please choose a different name.", "❌", PopupButtons::Ok);
            return;
        }
        // Install the plugin first physically
        if let Err( e ) =
            PluginEntry::install_plugin( &path.to_string_lossy(), &PathBuf::from( &svencoopdir ) )
        {
            popup("Install Error",
                  &format!( "Failed to install plugin {}.\nReason:\n{}\n\n\
                        You will need to manually add this file to the game.", name, e ),
                  "❌", PopupButtons::Ok );

            return;
        }

        let script_file = path.file_name()
            .and_then( |s| s.to_str() )
            .unwrap_or( "" )
        .trim_end_matches( ".as" );

        let (key, plugin) = PluginEntry::add_plugin( &name, script_file );
        data.plugins.insert( key.clone(), plugin );

        drop( data );
        refresh_plugin_list(app, plugin_data);

        if let Err( e ) = save_plugins( &plugin_data.borrow() )
        {
            popup( "Save Error", &format!("Failed to save plugins.\nReason: {e}"),
                  "❌", PopupButtons::Ok );
        }
    }
}

pub(crate) fn on_remove_clicked(app: &AppWindow, plugin_data: &Rc<RefCell<PluginContext>>)
{
    let data = plugin_data.borrow();

    let selected_name =
    match data.selected_plugin_name.clone()
    {
        Some( name ) => name,
        None => return
    };

    drop( data );
    
    if popup("Confirm Remove",
             &format!( "Are you sure you want to remove the plugin '{}'?", selected_name ),
             "❓", PopupButtons::YesNo ) == 2
    {
        return;
    }
    
    let mut data = plugin_data.borrow_mut();
    
    if data.plugins.remove( &selected_name ).is_some()
    {
        data.selected_plugin_name = None;
        drop( data );
        refresh_plugin_list(app, plugin_data);
        
        app.set_txt_name( "".into() );
        app.set_txt_script( "".into() );
        app.set_txt_concommandns( "".into() );
        app.set_cb_adminlevel( 0 );
        app.set_txt_maps_included( "".into() );
        app.set_txt_maps_excluded( "".into() );
        app.set_chk_enabled( false );
    }
}

pub(crate) fn on_apply_clicked(app: &AppWindow, plugin_data: &Rc<RefCell<PluginContext>>)
{
    let data = plugin_data.borrow();// <- should this be made mutable to start with?

    let selected_name =
    match data.selected_plugin_name.clone()
    {
        Some( name ) => name,
        None => return
    };
    
    if !data.plugins.contains_key( &selected_name )
    {
        popup("Plugin Not Found", "Selected plugin could not be found in the internal store.",
              "❌", PopupButtons::Ok );
        return;
    }

    drop( data );// Release this as it cannot be borrowed twice!

    let mut plugins = plugin_data.borrow_mut();
    let mut plugin = plugins.plugins.remove( &selected_name )
        .expect( "The plugin should be selected at this point?" );
    
    plugin.name = app.get_txt_name().to_string();
    plugin.script = app.get_txt_script().to_string();
    plugin.concommandns = app.get_txt_concommandns().to_string();
    plugin.adminlevel = AdminLevel::from( app.get_cb_adminlevel() as isize );
    plugin.maps_included = app.get_txt_maps_included().to_string();
    plugin.maps_excluded = app.get_txt_maps_excluded().to_string();
    
    let new_key = plugin.name.clone();
    plugins.plugins.insert( new_key.clone(), plugin );
    plugins.selected_plugin_name = Some( new_key );
    
    drop( plugins );
    
    refresh_plugin_list(app, plugin_data);
    
    if let Err( e ) = save_plugins( &plugin_data.borrow() )
    {
        popup("Save Error", &
            format!( "Failed to save plugins.\nReason: {}", e), "❌", PopupButtons::Ok );
    }
}

pub(crate) fn on_save_clicked(app: &AppWindow, plugin_data: &Rc<RefCell<PluginContext>>) -> Result<(), PlatformError>
{
    match save_plugins( &plugin_data.borrow() )
    {
        Ok( () ) => popup("Success", "Plugins saved.", "ℹ️", PopupButtons::Ok ),
        Err( e ) =>
        {
            popup("Error",
                  &format!( "Failed to save changes to plugin.\nReason:{}", e ),
                  "❌", PopupButtons::Ok );

            return Err( PlatformError::Other( e.to_string() ) );
        }
    };

    app.hide()
}

pub(crate) fn on_enabled_toggled(app: &AppWindow, plugin_data: &Rc<RefCell<PluginContext>>)
{
    let mut plugin_data = plugin_data.borrow_mut();
    
    if let Some( ref name ) = plugin_data.selected_plugin_name.clone()
    && let Some( plugin ) = plugin_data.plugins.get_mut( name )
    {
        plugin.toggle_state();
        app.set_plugin_list( ModelRc::new( VecModel::from( make_plugin_list( &plugin_data.plugins ) ) ) );
    }
}
