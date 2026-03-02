use crate::system::SystemStats;
use crate::wave_chart::{WaveChart, WaveData};
use iced::border::Radius;
use iced::widget::{column, container, row, text};
use iced::Element;
use iced::{Alignment, Length, Theme};

pub fn build_view<'a>(
    stats: &SystemStats,
    cpu_history: &'a WaveData,
    ram_history: &'a WaveData,
    temp_history: &'a WaveData,
    upload_history: &'a WaveData,
    download_history: &'a WaveData,
) -> Element<'a, crate::Message> {
    let stats_row = row![
        stat_card(
            "CPU",
            format!("{:.0}%", stats.cpu_usage),
            cpu_history,
            iced::Color::from_rgb(0.9, 0.3, 0.3),
        ),
        stat_card(
            "RAM",
            format!("{:.1} GB", stats.ram_used_gb),
            ram_history,
            iced::Color::from_rgb(0.3, 0.6, 0.9),
        ),
        temp_card(stats.temperature_celsius, temp_history),
        network_card(
            stats.upload_mbps,
            stats.download_mbps,
            upload_history,
            download_history,
        ),
    ]
    .spacing(15)
    .align_y(Alignment::Center);

    let content = column![stats_row].spacing(10).align_x(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(10)
        .into()
}

fn stat_card<'a>(
    label: &str,
    value: String,
    history: &'a WaveData,
    accent_color: iced::Color,
) -> Element<'a, crate::Message> {
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

    // Use wave chart instead of progress bar
    let wave = WaveChart::new(history.values(), accent_color)
        .height(Length::Fixed(40.0))
        .width(Length::Fill)
        .max_points(60);

    let card_content = column![label_text, value_text, wave]
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

fn temp_card<'a>(temp: f32, history: &'a WaveData) -> Element<'a, crate::Message> {
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

    // Use wave chart for temperature
    let wave = WaveChart::new(history.values(), color)
        .height(Length::Fixed(40.0))
        .width(Length::Fill)
        .max_points(60);

    let card_content = column![label, value_text, wave]
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

fn network_card<'a>(
    upload: f32,
    download: f32,
    upload_history: &'a WaveData,
    download_history: &'a WaveData,
) -> Element<'a, crate::Message> {
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

    // Create a row with both wave charts side by side
    let download_wave = WaveChart::new(download_history.values(), download_color)
        .height(Length::Fixed(18.0))
        .width(Length::FillPortion(1))
        .max_points(60);

    let upload_wave = WaveChart::new(upload_history.values(), upload_color)
        .height(Length::Fixed(18.0))
        .width(Length::FillPortion(1))
        .max_points(60);

    let waves_row = row![download_wave, upload_wave].spacing(8);

    let card_content = column![label, download_row, upload_row, waves_row]
        .spacing(4)
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
