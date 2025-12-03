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

use std::
{
    cell::RefCell,
    collections::HashMap,
    rc::Rc
};

use crate::plugin::
{
    PluginEntry,
    PluginState
};

const WINDOW_TITLE: &str = "Sven Co-op Plugin Manager";
const WINDOW_SIZE: (i32, i32) = ( 500, 450 );
const LISTBOX_SIZE: (i32, i32) = ( 150, 360 );
const LABEL_SIZE: (i32, i32) = ( 100, 30 );
const BUTTON_SIZE: (i32, i32) = ( 85, 30 );
const TEXTFIELD_SIZE: (i32, i32) = ( 200, 25 );

pub const UNCHECKED: &str = "☐";
pub const CHECKED:   &str = "✔";
pub const HELP_INFO: &str =
"This is a simple application to manage your Sven Co-op plugins.\n\n
Plugin List: Toggle checkboxes to enable/disable plugins.\n
Config Panel: Edit name, script, command namespace, admin level, and map filters.\n
Controls:\n
-'Add': Create new plugin\n
-'Remove': Remove selected plugin\n
-'Apply': Save current plugin changes\n
-'Save' Write all plugin states and exit\n
Tips:\n
-Use 'Apply' before switching plugins.\n
-'Save' commits all changes.\n
Use map filters to control plugin activation per map.\n\n
Thank you for using this app!\nIf you'd like to give feedback feel free to put them here: https://github.com/Outerbeast/SC-Plugin-Manager/issues";

#[derive(Default)]
pub struct MainWindow
{
    pub window: Window,
    pub lst_plugins: ListBox<String>,
    // Labels
    pub lbl_name: Label,
    pub lbl_script: Label,
    pub lbl_concommandns: Label,
    pub lbl_adminlevel: Label,
    pub lbl_maps_included: Label,
    pub lbl_maps_excluded: Label,
    // Text fields
    pub txt_name: TextInput,
    pub txt_script: TextInput,
    pub txt_concommandns: TextInput,
    pub txt_maps_included: TextBox,
    pub txt_maps_excluded: TextBox,
    // Interactive controls
    pub cb_adminlevel: ComboBox<String>,
    pub chk_enabled: CheckBox,
    pub btn_add: Button,
    pub btn_remove: Button,
    pub btn_apply: Button,
    pub btn_save: Button,
    pub btn_help: Button,
    // Plugin data, mutable during runtime
    pub plugins: Rc<RefCell<HashMap<String, PluginEntry>>>,
}

pub fn message_box(title: &str, body: &str, buttons: MessageButtons, icons: MessageIcons) -> MessageChoice
{
    let choice = message( &MessageParams
    {
        title: title,
        content: body,
        buttons: buttons,
        icons: icons,
    });

    choice
}

pub fn show_wait_splash() -> nwg::Window
{
    nwg::init().unwrap();

    let mut splash = nwg::Window::default();
    nwg::Window::builder()
        .size( ( 200, 0 ) )
        .position( ( nwg::Monitor::width() / 2 - 150, nwg::Monitor::height() / 2 - 50 ) )
        .title( "Initial setup, please wait..." ) // no title bar text
        .flags
        (
            nwg::WindowFlags::WINDOW
            | nwg::WindowFlags::VISIBLE
            | nwg::WindowFlags::POPUP, // no system menu, no buttons
        )
    .build( &mut splash ).unwrap();
    // !-UNDONE-!: Label doesn't show up for some reason
/*     let mut label = nwg::Label::default();
    nwg::Label::builder()
        .text( "Doing initial setup, please wait…" )
        .parent( &splash )
        .position( ( 20, 40 ) )
        .size( ( 260, 20 ) )
    .build( &mut label ).unwrap(); */

    splash
}

pub fn build_main_window(plugins: HashMap<String, PluginEntry>) -> Rc<RefCell<MainWindow>>
{
    let window = Rc::new( RefCell::new( MainWindow::default() ) );
    {
        let mut app_mut = window.borrow_mut();
        app_mut.plugins = Rc::new( RefCell::new( plugins ) );

        Window::builder()
            .size( WINDOW_SIZE )
            .position(
        {
                let center_x = ( Monitor::width() - WINDOW_SIZE.0 ) / 2;
                let center_y = ( Monitor::height() - WINDOW_SIZE.1 ) / 2;

                ( center_x, center_y )
            })
            .title( WINDOW_TITLE )
            .flags( WindowFlags::WINDOW | WindowFlags::VISIBLE )
        .build( &mut app_mut.window ).unwrap_or_default();
        // Left listbox
        ListBox::builder()
            .parent( &app_mut.window )
            .size( LISTBOX_SIZE )
            .position( ( 10, 20 ) )
            .collection(
            {
                let list_of_plugins = app_mut.plugins.borrow();
                // Collect (name, enabled) pairs, sort by name, then format with prefix
                let mut rows: Vec<(String, bool)> = list_of_plugins
                    .values()
                    .map( |p| ( p.name.clone(), p.state == PluginState::Enabled ) )
                .collect();
                // sort by name case-insensitively
                rows.sort_unstable_by_key( |(name, _)| name.to_ascii_lowercase() );
                // create final display strings with prefix
                rows.into_iter()
                    .map( |(name, enabled)|
                    {
                        format!( "{} {}", if enabled { CHECKED } else { UNCHECKED }, name )
                    })
                .collect::<Vec<String>>()
            })
        .build( &mut app_mut.lst_plugins ).unwrap_or_default();
        // Right side fields
        Label::builder()
            .text( "Name" )  
            .parent( &app_mut.window )
            .position( ( 170, 30 ) )
            .size( LABEL_SIZE )
        .build( &mut app_mut.lbl_name ).unwrap_or_default();

        TextInput::builder()
            .parent( &app_mut.window )
            .position( ( 280, 30 ) )
            .size( TEXTFIELD_SIZE )
        .build( &mut app_mut.txt_name ).unwrap_or_default();
        // Script field
        Label::builder()
            .text( "Script" )
            .parent( &app_mut.window )
            .position( ( 170, 70 ) )
            .size( LABEL_SIZE )
        .build( &mut app_mut.lbl_script ).unwrap_or_default();

        TextInput::builder()
            .parent( &app_mut.window )
            .position( ( 280, 70 ) )
            .size( TEXTFIELD_SIZE )
        .build( &mut app_mut.txt_script ).unwrap_or_default();
        // ConCommandNS field
        Label::builder()
            .text( "Command NS" )
            .parent( &app_mut.window )
            .position( ( 170, 110 ) )
            .size( LABEL_SIZE )
        .build( &mut app_mut.lbl_concommandns ).unwrap_or_default();

        TextInput::builder()
            .parent( &app_mut.window )
            .position( ( 280, 110 ) )
            .size( TEXTFIELD_SIZE )
        .build( &mut app_mut.txt_concommandns ).unwrap_or_default();
        // Enabled checkbox
        CheckBox::builder()
            .text( "Enabled" )
            .parent( &app_mut.window )
            .position(( 10, 380 ) )
            .size( ( 80, 25 ) )
        .build( &mut app_mut.chk_enabled ).unwrap_or_default();
        // Admin Level combo box
        Label::builder()
            .text( "Admin Level" )
            .parent( &app_mut.window )
            .position( ( 170, 150 ) )
            .size( LABEL_SIZE )
        .build( &mut app_mut.lbl_adminlevel ).unwrap_or_default();

        ComboBox::builder()
            .parent( &app_mut.window )
            .position( ( 280, 150 ) )
            .size( TEXTFIELD_SIZE )
            .collection( vec!["0: All".to_string(), "1: Players".to_string(), "2: Admins".to_string(), "3: Server Owner".to_string()] )
        .build( &mut app_mut.cb_adminlevel ).unwrap_or_default();
        // Maps included/excluded
        Label::builder()
            .text( "Included Maps" )
            .parent( &app_mut.window )
            .position(( 170, 180 ) )
            .size( ( 180, 25 ) )
        .build(&mut app_mut.lbl_maps_included ).unwrap_or_default();
        
        TextBox::builder()
            .parent( &app_mut.window )
            .position( ( 170, 200 ) )
            .size( ( 330, 80 ) )
        .build(&mut app_mut.txt_maps_included ).unwrap_or_default();

        Label::builder()
            .text("Excluded Maps")
            .parent( &app_mut.window )
            .position(( 170, 280 ) )
            .size( ( 180, 25 ) )
        .build( &mut app_mut.lbl_maps_excluded ).unwrap_or_default();

        TextBox::builder()
            .parent( &app_mut.window )
            .position(( 170, 300 ) )
            .size( ( 330, 80 ) )
        .build( &mut app_mut.txt_maps_excluded ).unwrap_or_default();
        // Buttons
        Button::builder()
            .text( "Add" )
            .parent( &app_mut.window )
            .position( ( 10, 410 ) )
            .size( BUTTON_SIZE )
        .build( &mut app_mut.btn_add ).unwrap_or_default();

        Button::builder()
            .text( "Remove" )
            .parent( &app_mut.window )
            .position( ( 110, 410 ) )
            .size( BUTTON_SIZE )
        .build( &mut app_mut.btn_remove ).unwrap_or_default();

        Button::builder()
            .text( "Apply" )
            .parent( &app_mut.window )
            .position( ( 300, 410 ) )
            .size( BUTTON_SIZE )
        .build( &mut app_mut.btn_apply ).unwrap_or_default();

        Button::builder()
            .text( "Save" )
            .parent( &app_mut.window )
            .position( ( 400, 410 ) )
            .size( BUTTON_SIZE )
        .build( &mut app_mut.btn_save ).unwrap_or_default();

        Button::builder()
            .text( "?" )
            .parent( &app_mut.window )
            .position( ( 455, 3 ) )
            .size( ( 25, 25 ) )
        .build( &mut app_mut.btn_help ).unwrap_or_default();
    }

    window
}
