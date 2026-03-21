use iced::widget::{button, column, container, horizontal_rule, row, text, Space};
use iced::{Alignment, Element, Length};

use crate::ui::app::{Message, Screen};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPOSITORY: &str = "https://github.com/goshitsarch-eng/Gosh-Yubico-Authenticator-for-Linux";

pub fn view(_app: &crate::ui::app::App) -> Element<Message> {
    let header = container(
        row![
            button(text("<- Back").size(14))
                .on_press(Message::NavigateTo(Screen::Home))
                .padding([6, 12])
                .style(button::text),
            Space::with_width(Length::Fill),
            text("About").size(18),
            Space::with_width(Length::Fill),
            Space::with_width(80),
        ]
        .align_y(Alignment::Center),
    )
    .padding(12)
    .width(Length::Fill);

    let content = column![
        Space::with_height(32),
        text("Gosh Authenticator").size(28),
        Space::with_height(8),
        text(format!("Version {}", VERSION)).size(14),
        Space::with_height(24),
        text("A pure Rust desktop application for managing").size(14),
        text("OATH (TOTP/HOTP) credentials on YubiKey devices.").size(14),
        Space::with_height(24),
        horizontal_rule(1),
        Space::with_height(24),
        text("Features").size(16),
        Space::with_height(8),
        text("  - TOTP and HOTP credential management").size(13),
        text("  - SHA-1, SHA-256, SHA-512 algorithms").size(13),
        text("  - Touch-required credentials").size(13),
        text("  - YubiKey password protection").size(13),
        text("  - QR code import from images").size(13),
        text("  - Cross-platform (Linux, Windows, macOS)").size(13),
        text("  - Clipboard with auto-clear").size(13),
        Space::with_height(24),
        horizontal_rule(1),
        Space::with_height(24),
        text("Built With").size(16),
        Space::with_height(8),
        text("  - Rust + Iced GUI framework").size(13),
        text("  - PC/SC for YubiKey communication").size(13),
        Space::with_height(24),
        horizontal_rule(1),
        Space::with_height(24),
        text("License").size(16),
        Space::with_height(8),
        text("GPL-3.0-or-later").size(13),
        Space::with_height(8),
        text(REPOSITORY).size(11),
        Space::with_height(24),
        text("Copyright (c) 2024-2025 Goshitsarch").size(12),
    ]
    .spacing(0)
    .align_x(Alignment::Center)
    .padding(16)
    .width(Length::Fill);

    let scrollable_content = iced::widget::scrollable(content);

    column![header, horizontal_rule(1), scrollable_content]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
