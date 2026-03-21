use iced::widget::{
    button, column, container, horizontal_rule, pick_list, row, text, text_input, Space,
};
use iced::{Alignment, Element, Length};

use crate::ui::app::{App, ConnectionState, Message, Screen};

pub fn view(app: &App) -> Element<Message> {
    let header = container(
        row![
            button(text("<- Back").size(14))
                .on_press(Message::NavigateTo(Screen::Home))
                .padding([6, 12])
                .style(button::text),
            Space::with_width(Length::Fill),
            text("Settings").size(18),
            Space::with_width(Length::Fill),
            Space::with_width(80),
        ]
        .align_y(Alignment::Center),
    )
    .padding(12)
    .width(Length::Fill);

    // Appearance section
    let theme_options: Vec<String> = vec!["system".into(), "light".into(), "dark".into()];
    let selected_theme: Option<String> = Some(app.settings.theme_mode.clone());

    let appearance_section = column![
        text("Appearance").size(16),
        Space::with_height(8),
        row![
            text("Theme").size(14),
            Space::with_width(Length::Fill),
            pick_list(
                theme_options,
                selected_theme,
                |selected: String| Message::ThemeModeChanged(selected),
            )
            .padding(6),
        ]
        .align_y(Alignment::Center),
    ]
    .spacing(4);

    // Clipboard section
    let timeout_options: Vec<String> = vec![
        "0".into(), "10".into(), "15".into(), "30".into(), "60".into(), "120".into(),
    ];
    let selected_timeout: Option<String> = Some(app.settings.clipboard_timeout.to_string());

    let clipboard_section = column![
        text("Clipboard").size(16),
        Space::with_height(8),
        row![
            text("Clear after (seconds)").size(14),
            Space::with_width(Length::Fill),
            pick_list(
                timeout_options,
                selected_timeout,
                |selected: String| Message::ClipboardTimeoutChanged(selected),
            )
            .padding(6),
        ]
        .align_y(Alignment::Center),
        text("Set to 0 to never clear").size(12),
    ]
    .spacing(4);

    // YubiKey password section (only if connected)
    let password_section = if app.connection_state == ConnectionState::Connected {
        let mut section = column![
            text("YubiKey Password").size(16),
            Space::with_height(8),
        ]
        .spacing(4);

        if app.yubikey_has_password {
            section = section.push(
                text("Password protection is enabled on this YubiKey").size(13),
            );
            section = section.push(Space::with_height(8));
            section = section.push(
                button(text("Remove Password").size(13))
                    .on_press(Message::RemovePassword)
                    .padding([6, 16])
                    .style(button::danger),
            );
            section = section.push(Space::with_height(12));
            section = section.push(text("Change Password").size(14));
        } else {
            section = section.push(
                text("No password protection on this YubiKey").size(13),
            );
            section = section.push(Space::with_height(8));
            section = section.push(text("Set Password").size(14));
        }

        section = section
            .push(Space::with_height(4))
            .push(
                text_input("New password", &app.new_password)
                    .on_input(Message::NewPasswordChanged)
                    .secure(true)
                    .padding(8),
            )
            .push(Space::with_height(4))
            .push(
                text_input("Confirm password", &app.confirm_password)
                    .on_input(Message::ConfirmPasswordChanged)
                    .on_submit(Message::SubmitNewPassword)
                    .secure(true)
                    .padding(8),
            )
            .push(Space::with_height(8))
            .push(
                button(text("Set Password").size(13))
                    .on_press(Message::SubmitNewPassword)
                    .padding([6, 16])
                    .style(button::primary),
            );

        section
    } else {
        column![
            text("YubiKey Password").size(16),
            Space::with_height(8),
            text("Connect a YubiKey to manage password settings").size(13),
        ]
        .spacing(4)
    };

    let content = column![
        appearance_section,
        Space::with_height(16),
        horizontal_rule(1),
        Space::with_height(16),
        clipboard_section,
        Space::with_height(16),
        horizontal_rule(1),
        Space::with_height(16),
        password_section,
    ]
    .spacing(0)
    .padding(16);

    let scrollable_content = iced::widget::scrollable(content);

    column![header, horizontal_rule(1), scrollable_content]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
