use iced::{Background, Border, Color, widget::container};
use iced_shadcn_v2::Theme;

use super::UiTokens;

pub fn card_surface(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme.palette.card)),
        text_color: Some(theme.palette.card_foreground),
        border: Border {
            color: theme.palette.border,
            width: UiTokens::WINDOW_BORDER,
            radius: UiTokens::CARD_RADIUS.into(),
        },
        ..container::Style::default()
    }
}

pub fn dialog_surface(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme.palette.background)),
        text_color: Some(theme.palette.foreground),
        border: Border {
            color: theme.palette.border,
            width: UiTokens::WINDOW_BORDER,
            radius: UiTokens::WINDOW_RADIUS.into(),
        },
        ..container::Style::default()
    }
}

pub fn brand_icon_surface(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme.palette.primary)),
        text_color: Some(theme.palette.primary_foreground),
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

pub fn muted_color(theme: &Theme) -> Color {
    theme.palette.muted_foreground
}

pub fn error_color(theme: &Theme) -> Color {
    theme.palette.destructive
}
