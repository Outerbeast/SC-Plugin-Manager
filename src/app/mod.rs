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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupChoice
{
    None,
    Ok,
    Cancel,
    Yes,
    No,
    Retry,
}

pub type AppWindow = super::PluginManagerWindow;

pub fn popup<F>(title: &str, message: &str, icon: &str, buttons: PopupButtons, on_close: F)
where F: Fn(PopupChoice) + 'static
{
    let dialog = super::MessageDialog::new().expect( "Dialog creation failed" );
    dialog.set_message_title( title.into() );
    dialog.set_message_text( message.into() );
    dialog.set_icon_text( icon.into() );
    dialog.set_buttons( buttons as i32 );
    // Set what happens when closing
    let dialog_weak = dialog.as_weak();
    dialog.on_close_dialog( move |r|
    {
        if let Some( d ) = dialog_weak.upgrade()
        {
            let choice = match r
            {
                1 => PopupChoice::Yes,
                2 => PopupChoice::No,
                _ => PopupChoice::None,
            };
            
            on_close( choice );
            d.hide().ok();
        }
    });

    let _ = dialog.run();
}

pub fn launch_gui(ctx: PluginContext) -> Result<(), PlatformError>
{
    let plugin_data = alloc_shared!( ctx );
    let app = super::PluginManagerWindow::new()?;
    app.show()?;
    
    controller::refresh_plugin_list( &app, &plugin_data );

    let app_weak = app.as_weak();
    let plugin_data_cloned = plugin_data.clone();
    app.on_plugin_selected( move |i|
    {
        if let Some( handle ) = app_weak.upgrade()
        {
            controller::on_plugin_selected( i, &handle, &plugin_data_cloned );
        }
    });

    let app_weak = app.as_weak();
    app.on_script_clicked( move ||
    {
        if let Some( handle ) = app_weak.upgrade()
        {
            controller::on_script_clicked( &handle );
        }
    });
    
    let app_weak = app.as_weak();
    let gui_data_clone = plugin_data.clone();
    app.on_add_clicked( move ||
    {
        if let Some( handle ) = app_weak.upgrade()
        {
            controller::on_add_clicked( &handle, &gui_data_clone );
        }
    });

    let app_weak = app.as_weak();
    let gui_data_clone = plugin_data.clone();
    app.on_remove_clicked( move ||
    {
        if let Some( handle ) = app_weak.upgrade()
        {
            let name = gui_data_clone.borrow().selected_plugin_name.clone().unwrap_or_default();
            let data_clone = gui_data_clone.clone();
            // Ask first before removal
            popup( "Confirm Remove",
                &format!( "Are you sure you want to remove the plugin '{}'?", name ),
                "❓",
                PopupButtons::YesNo,
                move |choice|
                {
                    if choice == PopupChoice::Yes
                    {
                        controller::on_remove_clicked( &handle, &data_clone );
                    }
                });
        }
    });

    let app_weak = app.as_weak();
    let gui_data_clone = plugin_data.clone();
    app.on_apply_clicked( move ||
    {
        if let Some( handle ) = app_weak.upgrade()
        {
            controller::on_apply_clicked( &handle, &gui_data_clone );
        }
    });
    
    let app_weak = app.as_weak();
    let gui_data_clone = plugin_data.clone();
    app.on_save_clicked( move ||
    {
        if let Some( handle ) = app_weak.upgrade()
        {
            let _ = controller::on_save_clicked( &handle, &gui_data_clone );
        }
    });

    let app_weak = app.as_weak();
    let gui_data_clone = plugin_data.clone();
    app.on_enabled_toggled( move |_|
    {
        if let Some( handle ) = app_weak.upgrade()
        {
            controller::on_enabled_toggled( &handle, &gui_data_clone );
        }
    });

    app.run()
}
