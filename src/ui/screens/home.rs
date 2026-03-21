use iced::widget::{
    button, column, container, horizontal_rule, row, scrollable, text, text_input, Space,
};
use iced::{Alignment, Element, Length, Theme};

use crate::ui::app::{App, AuthState, ConnectionState, Message, Screen};
use crate::ui::theme::{self, Palette};
use crate::ui::widgets;

pub fn view(app: &App) -> Element<Message> {
    let header = view_header(app);

    let body: Element<Message> = match (&app.connection_state, &app.auth_state) {
        (ConnectionState::Disconnected, _) => view_disconnected(),
        (ConnectionState::Connecting, _) => view_connecting(),
        (ConnectionState::Connected, AuthState::Required | AuthState::Failed(_)) => {
            view_auth_required(app)
        }
        (ConnectionState::Connected, AuthState::Authenticating) => view_authenticating(),
        (ConnectionState::Connected, _) => view_credentials(app),
    };

    column![header, body]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_header(app: &App) -> Element<'_, Message> {
    let title = text("Gosh Authenticator").size(20);

    let nav_buttons = row![
        button(text("Settings").size(13))
            .on_press(Message::NavigateTo(Screen::Settings))
            .padding([4, 10])
            .style(button::text),
        button(text("About").size(13))
            .on_press(Message::NavigateTo(Screen::About))
            .padding([4, 10])
            .style(button::text),
    ]
    .spacing(4);

    let mut header_row = row![title, Space::with_width(Length::Fill), nav_buttons]
        .spacing(8)
        .align_y(Alignment::Center);

    // Add key info button if connected
    if app.connection_state == ConnectionState::Connected {
        header_row = row![
            text("Gosh Authenticator").size(20),
            Space::with_width(Length::Fill),
            button(text("Key Info").size(13))
                .on_press(Message::NavigateTo(Screen::KeyInfo))
                .padding([4, 10])
                .style(button::text),
            button(text("Settings").size(13))
                .on_press(Message::NavigateTo(Screen::Settings))
                .padding([4, 10])
                .style(button::text),
            button(text("About").size(13))
                .on_press(Message::NavigateTo(Screen::About))
                .padding([4, 10])
                .style(button::text),
        ]
        .spacing(4)
        .align_y(Alignment::Center);
    }

    container(header_row)
        .padding(16)
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let pair = theme.extended_palette().primary.strong;
            container::Style {
                background: Some(iced::Background::Color(pair.color)),
                text_color: Some(pair.text),
                ..Default::default()
            }
        })
        .into()
}

fn view_disconnected() -> Element<'static, Message> {
    let content = column![
        Space::with_height(80),
        text("No YubiKey Detected").size(24),
        Space::with_height(16),
        text("Insert your YubiKey and click Connect").size(14),
        Space::with_height(24),
        button(
            text("Connect")
                .size(16)
                .align_x(iced::alignment::Horizontal::Center)
        )
        .on_press(Message::ConnectYubiKey)
        .padding([10, 32])
        .style(button::primary),
    ]
    .spacing(4)
    .align_x(Alignment::Center)
    .width(Length::Fill);

    container(content)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn view_connecting() -> Element<'static, Message> {
    let content = column![
        Space::with_height(100),
        text("Connecting to YubiKey...").size(18),
    ]
    .spacing(8)
    .align_x(Alignment::Center)
    .width(Length::Fill);

    container(content)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn view_auth_required(app: &App) -> Element<Message> {
    let mut auth_column = column![
        Space::with_height(60),
        text("Authentication Required").size(22),
        Space::with_height(8),
        text("Enter your YubiKey password").size(14),
        Space::with_height(16),
    ]
    .spacing(4)
    .align_x(Alignment::Center);

    // Show error if authentication failed
    if let AuthState::Failed(ref msg) = app.auth_state {
        auth_column = auth_column.push(
            text(msg)
                .size(13)
                .color(Palette::ERROR),
        );
        auth_column = auth_column.push(Space::with_height(8));
    }

    let password_input = text_input("Password", &app.password_input)
        .on_input(Message::PasswordInputChanged)
        .on_submit(Message::SubmitPassword)
        .secure(true)
        .padding(10)
        .width(280);

    let submit_btn = button(
        text("Unlock")
            .size(15)
            .align_x(iced::alignment::Horizontal::Center),
    )
    .on_press(Message::SubmitPassword)
    .padding([8, 24])
    .style(button::primary);

    auth_column = auth_column
        .push(password_input)
        .push(Space::with_height(12))
        .push(submit_btn);

    container(auth_column)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn view_authenticating() -> Element<'static, Message> {
    let content = column![
        Space::with_height(100),
        text("Authenticating...").size(18),
    ]
    .spacing(8)
    .align_x(Alignment::Center)
    .width(Length::Fill);

    container(content)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn view_credentials(app: &App) -> Element<Message> {
    let filtered = app.filtered_credentials();

    // Search bar + action buttons
    let search_bar = row![
        text_input("Search credentials...", &app.search_query)
            .on_input(Message::SearchChanged)
            .padding(8)
            .width(Length::Fill),
        button(text("+").size(18))
            .on_press(Message::NavigateTo(Screen::AddCredential))
            .padding([6, 14])
            .style(button::primary),
        button(text("Refresh").size(13))
            .on_press(Message::RefreshCredentials)
            .padding([8, 12])
            .style(button::secondary),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(12);

    // TOTP countdown bar
    let countdown_color = theme::countdown_color(app.totp_remaining, 30);
    let countdown_bar = container(
        row![
            text(format!("TOTP refreshes in {}s", app.totp_remaining))
                .size(12),
            Space::with_width(Length::Fill),
            // Visual countdown bar
            container(Space::with_width(Length::Fixed(0.0)))
                .width(Length::Fixed(
                    (app.totp_remaining as f32 / 30.0) * 200.0,
                ))
                .height(4)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(countdown_color)),
                    border: iced::Border {
                        radius: 2.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .padding([4, 12]);

    // Credentials list
    let credentials_list: Element<Message> = if filtered.is_empty() {
        let msg = if app.credentials.is_empty() {
            "No credentials on this YubiKey"
        } else {
            "No matching credentials"
        };
        container(
            column![Space::with_height(40), text(msg).size(14),]
                .align_x(Alignment::Center)
                .width(Length::Fill),
        )
        .center_x(Length::Fill)
        .into()
    } else {
        let mut cred_column = column![].spacing(2).padding([0, 8]);

        for cred in &filtered {
            cred_column = cred_column.push(widgets::credential_card::view(
                cred,
                app.totp_remaining,
                &app.delete_confirm,
            ));
        }

        scrollable(cred_column)
            .height(Length::Fill)
            .into()
    };

    // Disconnect button at bottom
    let footer = container(
        button(text("Disconnect").size(13))
            .on_press(Message::Disconnect)
            .padding([6, 16])
            .style(button::secondary),
    )
    .padding(8)
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center);

    column![
        search_bar,
        countdown_bar,
        horizontal_rule(1),
        credentials_list,
        horizontal_rule(1),
        footer,
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
