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
pub mod controller;

use slint::
{
    ComponentHandle,
    PlatformError
};

use crate::
{
    alloc_locked,
    alloc_shared,
    plugin::PluginContext
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PopupButtons
{
    None = 0,
    Ok,
    OkCancel,
    YesNo,
    YesNoCancel,
    RetryCancel
}

pub type AppWindow = super::PluginManagerWindow;

pub fn popup(title: &str, message: &str, icon: &str, buttons: PopupButtons) -> i32
{
    let dialog = super::MessageDialog::new().unwrap();
    dialog.set_message_title( title.into() );
    dialog.set_message_text( message.into() );
    dialog.set_icon_text( icon.into() );
    dialog.set_buttons( buttons as i32 );

    let dialog_weak = dialog.as_weak();
    let result = alloc_locked!( 0i32 );

    let result_clone = result.clone();
    dialog.on_close_dialog( move |r|
    {
        if let Some( d ) = dialog_weak.upgrade()
        {
            *result_clone.lock().unwrap() = r;
            d.hide().ok();
        }
    });

    let _ = dialog.run();

    *result.lock().unwrap()
}

pub fn launch_gui(ctx: PluginContext) -> Result<(), PlatformError>
{
    let plugin_data = alloc_shared!( ctx );
    let app = super::PluginManagerWindow::new()?;
    app.show()?;
    
    controller::refresh_plugin_list( &app, &plugin_data );
    // Setup event handlers
    let app_weak = app.as_weak();
    let plugin_data_cloned = plugin_data.clone();
    // Plugin list selection
    app.on_plugin_selected( move |i|
    {
        if let Some( handle ) = app_weak.upgrade()
        {
            controller::on_plugin_selected( i, &handle, &plugin_data_cloned );
        }
    });

    let app_weak = app.as_weak();
    let gui_data_clone = plugin_data.clone();
    // Add button
    app.on_add_clicked( move ||
    {
        if let Some( handle ) = app_weak.upgrade()
        {
            controller::on_add_clicked( &handle, &gui_data_clone );
        }
    });

    let app_weak = app.as_weak();
    let gui_data_clone = plugin_data.clone();
    // Remove button
    app.on_remove_clicked( move ||
    {
        if let Some( handle ) = app_weak.upgrade()
        {
            controller::on_remove_clicked( &handle, &gui_data_clone );
        }
    });

    let app_weak = app.as_weak();
    let gui_data_clone = plugin_data.clone();
    // Apply button
    app.on_apply_clicked( move ||
    {
        if let Some( handle ) = app_weak.upgrade()
        {
            controller::on_apply_clicked( &handle, &gui_data_clone );
        }
    });

    let app_weak = app.as_weak();
    let gui_data_clone = plugin_data.clone();
    // Save button
    app.on_save_clicked( move ||
    {
        if let Some( handle ) = app_weak.upgrade()
        {
            let _ = controller::on_save_clicked( &handle, &gui_data_clone );
        }
    });

    let app_weak = app.as_weak();
    let gui_data_clone = plugin_data.clone();
    // Enabled checkbox
    app.on_enabled_toggled( move |_|
    {
        if let Some( handle ) = app_weak.upgrade()
        {
            controller::on_enabled_toggled( &handle, &gui_data_clone );
        }
    });

    app.run()
}
