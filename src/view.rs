use crate::system::SystemStats;
use iced::border::Radius;
use iced::widget::{column, container, progress_bar, row, text};
use iced::Element;
use iced::{Alignment, Length, Theme};

pub fn build_view(stats: &SystemStats) -> Element<'static, crate::Message> {
    let stats_row = row![
        stat_card(
            "CPU",
            format!("{:.0}%", stats.cpu_usage),
            stats.cpu_usage,
            iced::Color::from_rgb(0.9, 0.3, 0.3),
        ),
        stat_card(
            "RAM",
            format!("{:.1} GB", stats.ram_used_gb),
            stats.ram_usage_percent,
            iced::Color::from_rgb(0.3, 0.6, 0.9),
        ),
        temp_card(stats.temperature_celsius),
        network_card(stats.upload_mbps, stats.download_mbps),
    ]
    .spacing(15)
    .align_y(Alignment::Center);

    let content = column![stats_row]
        .spacing(10)
        .align_x(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(10)
        .into()
}

fn stat_card(
    label: &str,
    value: String,
    percent: f32,
    accent_color: iced::Color,
) -> Element<'static, crate::Message> {
    let normalized_percent = percent.clamp(0.0, 100.0) / 100.0;
    let label_owned = label.to_owned();

    let label_text = text(label_owned)
        .size(28)
        .style(|theme: &Theme| iced::widget::text::Style {
            color: Some(theme.palette().text.scale_alpha(0.7)),
        });

    let value_text = text(value)
        .size(72)
        .style(move |_theme: &Theme| iced::widget::text::Style {
            color: Some(accent_color),
        });

    let bar = progress_bar(0.0..=1.0, normalized_percent)
        .height(Length::Fixed(4.0))
        .style(move |theme: &Theme| iced::widget::progress_bar::Style {
            bar: accent_color.into(),
            background: theme.palette().background.scale_alpha(0.3).into(),
            border: iced::Border {
                radius: Radius::new(2.0),
                ..Default::default()
            },
        });

    let card_content = column![label_text, value_text, bar]
        .spacing(6)
        .align_x(Alignment::Center);

    container(card_content)
        .width(Length::Fixed(220.0))
        .height(Length::Fixed(150.0))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(12)
        .style(|theme: &Theme| {
            let background = theme.palette().background;
            iced::widget::container::Style {
                background: Some(background.scale_alpha(0.5).into()),
                border: iced::Border {
                    color: theme.palette().text.scale_alpha(0.1),
                    width: 1.0,
                    radius: Radius::new(12.0),
                },
                ..Default::default()
            }
        })
        .into()
}

fn temp_card(temp: f32) -> Element<'static, crate::Message> {
    let label = text("TEMP")
        .size(28)
        .style(|theme: &Theme| iced::widget::text::Style {
            color: Some(theme.palette().text.scale_alpha(0.7)),
        });

    let color = if temp > 80.0 {
        iced::Color::from_rgb(1.0, 0.2, 0.2)
    } else if temp > 60.0 {
        iced::Color::from_rgb(1.0, 0.6, 0.2)
    } else {
        iced::Color::from_rgb(0.2, 0.9, 0.4)
    };

    let value = if temp > 0.0 {
        format!("{:.0}°C", temp)
    } else {
        "--".to_string()
    };

    let value_text = text(value)
        .size(72)
        .style(move |_theme: &Theme| iced::widget::text::Style { color: Some(color) });

    let indicator = container(text(""))
        .width(Length::Fixed(30.0))
        .height(Length::Fixed(4.0))
        .style(move |_theme: &Theme| iced::widget::container::Style {
            background: Some(color.into()),
            border: iced::Border {
                radius: Radius::new(2.0),
                ..Default::default()
            },
            ..Default::default()
        });

    let card_content = column![label, value_text, indicator]
        .spacing(6)
        .align_x(Alignment::Center);

    container(card_content)
        .width(Length::Fixed(170.0))
        .height(Length::Fixed(150.0))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(12)
        .style(|theme: &Theme| {
            let background = theme.palette().background;
            iced::widget::container::Style {
                background: Some(background.scale_alpha(0.5).into()),
                border: iced::Border {
                    color: theme.palette().text.scale_alpha(0.1),
                    width: 1.0,
                    radius: Radius::new(12.0),
                },
                ..Default::default()
            }
        })
        .into()
}

fn network_card(upload: f32, download: f32) -> Element<'static, crate::Message> {
    let label = text("NETWORK")
        .size(28)
        .style(|theme: &Theme| iced::widget::text::Style {
            color: Some(theme.palette().text.scale_alpha(0.7)),
        });

    let download_color = iced::Color::from_rgb(0.3, 0.8, 0.5);
    let upload_color = iced::Color::from_rgb(0.8, 0.5, 0.3);

    let download_row = row![
        text("▼").size(28).style(move |_theme: &Theme| {
            iced::widget::text::Style {
                color: Some(download_color),
            }
        }),
        text(format_download(download)).size(42),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let upload_row = row![
        text("▲").size(28).style(move |_theme: &Theme| {
            iced::widget::text::Style {
                color: Some(upload_color),
            }
        }),
        text(format_upload(upload)).size(42),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let card_content = column![label, download_row, upload_row]
        .spacing(8)
        .align_x(Alignment::Center);

    container(card_content)
        .width(Length::Fixed(240.0))
        .height(Length::Fixed(150.0))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(12)
        .style(|theme: &Theme| {
            let background = theme.palette().background;
            iced::widget::container::Style {
                background: Some(background.scale_alpha(0.5).into()),
                border: iced::Border {
                    color: theme.palette().text.scale_alpha(0.1),
                    width: 1.0,
                    radius: Radius::new(12.0),
                },
                ..Default::default()
            }
        })
        .into()
}

fn format_download(mbps: f32) -> String {
    if mbps >= 1000.0 {
        format!("{:.1} Gb/s", mbps / 1000.0)
    } else if mbps >= 1.0 {
        format!("{:.1} Mb/s", mbps)
    } else {
        format!("{:.0} Kb/s", mbps * 1000.0)
    }
}

fn format_upload(mbps: f32) -> String {
    if mbps >= 1000.0 {
        format!("{:.1} Gb/s", mbps / 1000.0)
    } else if mbps >= 1.0 {
        format!("{:.1} Mb/s", mbps)
    } else {
        format!("{:.0} Kb/s", mbps * 1000.0)
    }
}
