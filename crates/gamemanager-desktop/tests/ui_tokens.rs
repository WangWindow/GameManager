use gamemanager_core::ThemeMode;
use gamemanager_desktop::{
    message::SystemTheme,
    state::{AppTheme, ShellState},
    ui::tokens::UiTokens,
};

#[test]
fn v093_geometry_is_a_single_shared_contract() {
    assert_eq!(UiTokens::TITLE_BAR_HEIGHT, 40.0);
    assert_eq!(UiTokens::TITLE_TEXT_SIZE, 14.0);
    assert_eq!(UiTokens::TITLE_ACTION_TEXT_SIZE, 12.0);
    assert_eq!(UiTokens::CARD_COVER_SIZE, 48.0);
    assert_eq!(UiTokens::CARD_EMPTY_ICON_SIZE, 24.0);
    assert_eq!(UiTokens::EMPTY_STATE_ICON_SIZE, 40.0);
    assert_eq!(UiTokens::BRAND_ICON_BOX_WIDTH, 28.0);
    assert_eq!(UiTokens::BRAND_ICON_BOX_HEIGHT, 28.0);
    assert_eq!(UiTokens::BRAND_ICON_SIZE, 18.0);
    assert_ne!(UiTokens::BRAND_ICON_BOX_WIDTH, UiTokens::CARD_COVER_SIZE);
    assert_eq!(UiTokens::CARD_ACTION_SIZE, 28.0);
    assert_eq!(UiTokens::CARD_TITLE_SIZE, 14.0);
    assert_eq!(UiTokens::CARD_METADATA_SIZE, 12.0);
    assert_eq!(UiTokens::RESIZE_EDGE_SIZE, 6.0);
    assert_eq!(UiTokens::RESIZE_CORNER_SIZE, 10.0);
    assert_eq!(UiTokens::SEARCH_MAX_WIDTH, 512.0);
    assert_eq!(UiTokens::DIALOG_MAX_NARROW, 448.0);
    assert_eq!(UiTokens::DIALOG_MAX_WIDE, 512.0);
}

#[test]
fn system_theme_drives_iced_and_shadcn_from_the_same_resolution() {
    let mut shell = ShellState::with_theme_mode(ThemeMode::System);
    shell.apply_system_theme(SystemTheme::Light);

    assert_eq!(shell.resolved_theme(), AppTheme::Light);
    assert_eq!(shell.iced_theme(), iced::Theme::Light);
}
