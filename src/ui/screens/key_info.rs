use iced::widget::{button, column, container, horizontal_rule, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::ui::app::{App, Message, Screen};

pub fn view(app: &App) -> Element<Message> {
    let header = container(
        row![
            button(text("<- Back").size(14))
                .on_press(Message::NavigateTo(Screen::Home))
                .padding([6, 12])
                .style(button::text),
            Space::with_width(Length::Fill),
            text("YubiKey Info").size(18),
            Space::with_width(Length::Fill),
            Space::with_width(80),
        ]
        .align_y(Alignment::Center),
    )
    .padding(12)
    .width(Length::Fill);

    let content = if let Some(version) = app.yubikey_version {
        let device_id_str = app
            .yubikey_device_id
            .map(|id| {
                id.iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(":")
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let version_str = format!("{}.{}.{}", version.0, version.1, version.2);

        let credential_count = app.credentials.len();
        let totp_count = app.credentials.iter().filter(|c| c.is_totp()).count();
        let hotp_count = credential_count - totp_count;
        let touch_count = app.credentials.iter().filter(|c| c.touch_required).count();

        let password_status = if app.yubikey_has_password {
            "Enabled"
        } else {
            "Disabled"
        };

        column![
            info_row("OATH Version", version_str),
            Space::with_height(8),
            info_row("Device ID", device_id_str),
            Space::with_height(8),
            info_row("Password Protection", password_status.to_string()),
            Space::with_height(16),
            horizontal_rule(1),
            Space::with_height(16),
            text("Credentials Summary").size(16),
            Space::with_height(8),
            info_row("Total Credentials", credential_count.to_string()),
            Space::with_height(4),
            info_row("TOTP", totp_count.to_string()),
            Space::with_height(4),
            info_row("HOTP", hotp_count.to_string()),
            Space::with_height(4),
            info_row("Touch Required", touch_count.to_string()),
            Space::with_height(24),
            row![
                button(text("Refresh").size(13))
                    .on_press(Message::RefreshCredentials)
                    .padding([8, 16])
                    .style(button::secondary),
                Space::with_width(8),
                button(text("Disconnect").size(13))
                    .on_press(Message::Disconnect)
                    .padding([8, 16])
                    .style(button::danger),
            ]
            .spacing(8),
        ]
        .spacing(0)
        .padding(16)
    } else {
        column![
            Space::with_height(40),
            text("No YubiKey connected").size(16),
            Space::with_height(16),
            button(text("Connect").size(14))
                .on_press(Message::ConnectYubiKey)
                .padding([8, 24])
                .style(button::primary),
        ]
        .spacing(4)
        .align_x(Alignment::Center)
        .padding(16)
    };

    column![header, horizontal_rule(1), content]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn info_row(label: &str, value: String) -> Element<'static, Message> {
    row![
        text(label.to_string()).size(14),
        Space::with_width(Length::Fill),
        text(value).size(14),
    ]
    .align_y(Alignment::Center)
    .into()
}
