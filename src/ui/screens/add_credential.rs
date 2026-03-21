use iced::widget::{
    button, checkbox, column, container, horizontal_rule, pick_list, row, text, text_input, Space,
};
use iced::{Alignment, Element, Length};

use crate::ui::app::{App, Message, Screen};
use crate::ui::theme::Palette;

pub fn view(app: &App) -> Element<Message> {
    let header = container(
        row![
            button(text("<- Back").size(14))
                .on_press(Message::NavigateTo(Screen::Home))
                .padding([6, 12])
                .style(button::text),
            Space::with_width(Length::Fill),
            text("Add Credential").size(18),
            Space::with_width(Length::Fill),
            Space::with_width(80),
        ]
        .align_y(Alignment::Center),
    )
    .padding(12)
    .width(Length::Fill);

    // URI import section
    let uri_section = column![
        text("Import from URI or QR Code").size(14),
        Space::with_height(4),
        row![
            text_input("otpauth://totp/...", &app.add_form.uri_input)
                .on_input(Message::AddFormUriChanged)
                .on_submit(Message::AddFormParseUri)
                .padding(8)
                .width(Length::Fill),
            button(text("Import").size(13))
                .on_press(Message::AddFormParseUri)
                .padding([8, 12]),
        ]
        .spacing(8),
        button(text("Scan QR Code from Image...").size(13))
            .on_press(Message::AddFormScanQr)
            .padding([6, 12])
            .style(button::secondary),
    ]
    .spacing(4);

    // OTP Type pick list
    let type_options: Vec<String> = vec!["TOTP".into(), "HOTP".into()];
    let selected_type: Option<String> = Some(
        match app.add_form.oath_type_index {
            1 => "HOTP",
            _ => "TOTP",
        }
        .into(),
    );

    // Algorithm pick list
    let algo_options: Vec<String> = vec!["SHA-1".into(), "SHA-256".into(), "SHA-512".into()];
    let selected_algo: Option<String> = Some(
        match app.add_form.algorithm_index {
            1 => "SHA-256",
            2 => "SHA-512",
            _ => "SHA-1",
        }
        .into(),
    );

    // Digits pick list
    let digits_options: Vec<String> = vec!["6".into(), "7".into(), "8".into()];
    let selected_digits: Option<String> = Some(
        match app.add_form.digits_index {
            1 => "7",
            2 => "8",
            _ => "6",
        }
        .into(),
    );

    // Manual entry form
    let form = column![
        horizontal_rule(1),
        Space::with_height(8),
        text("Manual Entry").size(14),
        Space::with_height(8),
        // Issuer
        text("Issuer (optional)").size(13),
        text_input("e.g., Google, GitHub", &app.add_form.issuer)
            .on_input(Message::AddFormIssuerChanged)
            .padding(8),
        Space::with_height(4),
        // Account
        text("Account *").size(13),
        text_input("e.g., user@example.com", &app.add_form.account)
            .on_input(Message::AddFormAccountChanged)
            .padding(8),
        Space::with_height(4),
        // Secret
        text("Secret Key (Base32) *").size(13),
        text_input("e.g., JBSWY3DPEHPK3PXP", &app.add_form.secret)
            .on_input(Message::AddFormSecretChanged)
            .padding(8),
        Space::with_height(8),
        // OTP Type, Algorithm, Digits
        row![
            column![
                text("Type").size(13),
                pick_list(type_options, selected_type, |selected: String| {
                    Message::AddFormOathTypeChanged(if selected == "HOTP" { 1 } else { 0 })
                })
                .padding(6),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                text("Algorithm").size(13),
                pick_list(algo_options, selected_algo, |selected: String| {
                    Message::AddFormAlgorithmChanged(match selected.as_str() {
                        "SHA-256" => 1,
                        "SHA-512" => 2,
                        _ => 0,
                    })
                })
                .padding(6),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            column![
                text("Digits").size(13),
                pick_list(digits_options, selected_digits, |selected: String| {
                    Message::AddFormDigitsChanged(match selected.as_str() {
                        "7" => 1,
                        "8" => 2,
                        _ => 0,
                    })
                })
                .padding(6),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(12),
        Space::with_height(8),
        // Touch required
        checkbox("Require touch", app.add_form.require_touch)
            .on_toggle(Message::AddFormTouchToggled),
    ]
    .spacing(4);

    // HOTP counter (conditional)
    let counter_section = if app.add_form.oath_type_index == 1 {
        column![
            Space::with_height(4),
            text("Initial Counter").size(13),
            text_input("0", &app.add_form.initial_counter)
                .on_input(Message::AddFormCounterChanged)
                .padding(8),
        ]
        .spacing(4)
    } else {
        column![]
    };

    // Error message
    let error_section = if let Some(ref err) = app.add_form.error {
        container(text(err.clone()).size(13).color(Palette::ERROR)).padding([8, 0])
    } else {
        container(Space::with_height(0))
    };

    // Submit button
    let submit_section = container(
        button(
            text("Add Credential")
                .size(15)
                .align_x(iced::alignment::Horizontal::Center),
        )
        .on_press(Message::AddFormSubmit)
        .padding([10, 32])
        .style(button::primary)
        .width(Length::Fill),
    )
    .padding([12, 0]);

    let content = column![
        uri_section,
        Space::with_height(8),
        form,
        counter_section,
        error_section,
        submit_section,
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
