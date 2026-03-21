use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length, Theme};

use crate::core::credential::{Credential, CredentialId};
use crate::ui::app::Message;
use crate::ui::theme::{self, Palette};

pub fn view<'a>(
    credential: &'a Credential,
    totp_remaining: u32,
    delete_confirm: &'a Option<CredentialId>,
) -> Element<'a, Message> {
    let is_confirming_delete = delete_confirm
        .as_ref()
        .map(|id| *id == credential.id)
        .unwrap_or(false);

    // Issuer label
    let issuer_text = if let Some(ref issuer) = credential.issuer {
        text(issuer).size(12)
    } else {
        text("").size(12)
    };

    // Account label
    let account_text = text(&credential.account).size(14);

    // Type badge
    let type_badge = text(credential.oath_type.display_name())
        .size(10);

    // Code display or action button
    let code_section: Element<'a, Message> = if let Some(ref code) = credential.code {
        let code_color = if credential.is_totp() {
            theme::countdown_color(totp_remaining, 30)
        } else {
            Palette::PRIMARY
        };

        row![
            text(code).size(24).color(code_color),
            Space::with_width(8),
            button(text("Copy").size(11))
                .on_press(Message::CopyCode(code.clone()))
                .padding([4, 8])
                .style(button::secondary),
        ]
        .align_y(Alignment::Center)
        .into()
    } else if credential.touch_required {
        button(text("Touch to reveal").size(12))
            .on_press(Message::CalculateCredential(credential.id.clone()))
            .padding([4, 10])
            .style(button::primary)
            .into()
    } else {
        button(text("Calculate").size(12))
            .on_press(Message::CalculateCredential(credential.id.clone()))
            .padding([4, 10])
            .style(button::secondary)
            .into()
    };

    // Delete button / confirmation
    let delete_section: Element<'a, Message> = if is_confirming_delete {
        row![
            text("Delete?").size(11).color(Palette::ERROR),
            button(text("Yes").size(11))
                .on_press(Message::DeleteCredential(credential.id.clone()))
                .padding([2, 8])
                .style(button::danger),
            button(text("No").size(11))
                .on_press(Message::CancelDelete)
                .padding([2, 8])
                .style(button::secondary),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
        .into()
    } else {
        button(text("Del").size(10))
            .on_press(Message::ConfirmDelete(credential.id.clone()))
            .padding([2, 6])
            .style(button::text)
            .into()
    };

    let card_content = column![
        // Top row: issuer + type + delete
        row![
            issuer_text,
            Space::with_width(4),
            type_badge,
            Space::with_width(Length::Fill),
            delete_section,
        ]
        .align_y(Alignment::Center),
        // Account name
        account_text,
        Space::with_height(4),
        // Code
        code_section,
    ]
    .spacing(2);

    container(card_content)
        .padding(12)
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(iced::Background::Color(palette.background.weak.color)),
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: palette.background.strong.color,
                },
                ..Default::default()
            }
        })
        .into()
}
