use iced::{Theme, widget::Text};
use lucide_icons::iced::{
    icon_app_window, icon_chevron_down, icon_chevron_up, icon_circle_help, icon_code_xml,
    icon_ellipsis_vertical, icon_file, icon_folder, icon_folder_open, icon_folder_search,
    icon_gamepad_2, icon_globe, icon_layout_grid, icon_list, icon_loader_circle, icon_maximize,
    icon_minus, icon_monitor, icon_package_open, icon_play, icon_plus, icon_refresh_cw,
    icon_search, icon_settings, icon_trash_2, icon_x,
};

pub type IconText<'a> = Text<'a, Theme>;

pub fn app_window<'a>() -> IconText<'a> {
    icon_app_window::<Theme>()
}
pub fn chevron_down<'a>() -> IconText<'a> {
    icon_chevron_down::<Theme>()
}
pub fn chevron_up<'a>() -> IconText<'a> {
    icon_chevron_up::<Theme>()
}
pub fn circle_help<'a>() -> IconText<'a> {
    icon_circle_help::<Theme>()
}
pub fn code_xml<'a>() -> IconText<'a> {
    icon_code_xml::<Theme>()
}
pub fn ellipsis_vertical<'a>() -> IconText<'a> {
    icon_ellipsis_vertical::<Theme>()
}
pub fn file<'a>() -> IconText<'a> {
    icon_file::<Theme>()
}
pub fn folder<'a>() -> IconText<'a> {
    icon_folder::<Theme>()
}
pub fn folder_open<'a>() -> IconText<'a> {
    icon_folder_open::<Theme>()
}
pub fn folder_search<'a>() -> IconText<'a> {
    icon_folder_search::<Theme>()
}
pub fn gamepad_2<'a>() -> IconText<'a> {
    icon_gamepad_2::<Theme>()
}
pub fn globe<'a>() -> IconText<'a> {
    icon_globe::<Theme>()
}
pub fn layout_grid<'a>() -> IconText<'a> {
    icon_layout_grid::<Theme>()
}
pub fn list<'a>() -> IconText<'a> {
    icon_list::<Theme>()
}
pub fn loader_circle<'a>() -> IconText<'a> {
    icon_loader_circle::<Theme>()
}
pub fn maximize<'a>() -> IconText<'a> {
    icon_maximize::<Theme>()
}
pub fn minus<'a>() -> IconText<'a> {
    icon_minus::<Theme>()
}
pub fn monitor<'a>() -> IconText<'a> {
    icon_monitor::<Theme>()
}
pub fn package_open<'a>() -> IconText<'a> {
    icon_package_open::<Theme>()
}
pub fn play<'a>() -> IconText<'a> {
    icon_play::<Theme>()
}
pub fn plus<'a>() -> IconText<'a> {
    icon_plus::<Theme>()
}
pub fn refresh_cw<'a>() -> IconText<'a> {
    icon_refresh_cw::<Theme>()
}
pub fn search<'a>() -> IconText<'a> {
    icon_search::<Theme>()
}
pub fn settings<'a>() -> IconText<'a> {
    icon_settings::<Theme>()
}
pub fn trash_2<'a>() -> IconText<'a> {
    icon_trash_2::<Theme>()
}
pub fn x<'a>() -> IconText<'a> {
    icon_x::<Theme>()
}

pub fn engine<'a>(engine_type: &str) -> IconText<'a> {
    match engine_type.to_ascii_lowercase().as_str() {
        "electron" => app_window(),
        "godot" | "unity" | "unreal" | "rpgmakervx" | "rpgmakervxace" => gamepad_2(),
        "html" | "rpgmakermv" | "rpgmakermz" => code_xml(),
        "renpy" => globe(),
        _ => circle_help(),
    }
}
