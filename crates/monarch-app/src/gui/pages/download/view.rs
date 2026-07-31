use iced::mouse;
use iced::widget::canvas::{self, gradient, stroke, Frame, Geometry, Path, Stroke, Text};
use iced::widget::{button, column, container, image, row, scrollable, stack, text, Space};
use iced::{alignment, Color, Element, Length, Pixels, Point, Rectangle, Renderer, Theme};

use crate::gui::pages::download::{format_eta, DownloadPage, Message, QueueStatus};
use crate::gui::styles;

// Deep rich Monarch orange (same hue family as the header widget).
const DOWNLOAD_COLOR: Color = Color::from_rgb8(204, 86, 0);
// Electric blue-violet (less magenta / pink).
const WRITE_COLOR: Color = Color::from_rgb8(88, 36, 230);

struct SpeedGraph {
    download: Vec<f32>,
    write: Vec<f32>,
}

impl<Message> canvas::Program<Message> for SpeedGraph {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let pad_l = 64.0;
        let pad_r = 64.0;
        let pad_t = 14.0;
        let pad_b = 14.0;
        let width = (bounds.width - pad_l - pad_r).max(1.0);
        let height = (bounds.height - pad_t - pad_b).max(1.0);
        let baseline_y = pad_t + height;

        let peak = self
            .download
            .iter()
            .chain(self.write.iter())
            .copied()
            .fold(0.0_f32, f32::max);
        // Top of the scale sits ~20% above the peak, then snaps up to a clean 20.
        let (unit, divisor) = rate_unit((peak * 1.2).max(1.0));
        let peak_in_unit = (peak as f64 * 1_000_000.0) / divisor;
        let top = ceil_to_20(peak_in_unit * 1.2).max(20.0);
        let max_speed = ((top * divisor) / 1_000_000.0) as f32;

        // Soft horizontal grid + dynamic Y labels on both sides.
        for i in 0..5 {
            let t = i as f32 / 4.0;
            let y = pad_t + height * t;
            let grid = Path::line(Point::new(pad_l, y), Point::new(pad_l + width, y));
            frame.stroke(
                &grid,
                Stroke::default()
                    .with_width(1.0)
                    .with_color(Color::from_rgba8(255, 255, 255, 0.05)),
            );

            let tick_value = if i == 4 {
                0.0
            } else {
                round_to_10(top * (4 - i) as f64 / 4.0)
            };
            let label = format!("{:.0} {unit}", tick_value);

            let label_color = Color::from_rgba8(180, 180, 190, 0.9);
            frame.fill_text(Text {
                content: label.clone(),
                position: Point::new(pad_l - 8.0, y),
                color: label_color,
                size: Pixels(11.0),
                font: styles::fonts::REGULAR,
                align_x: alignment::Horizontal::Right.into(),
                align_y: alignment::Vertical::Center,
                ..Text::default()
            });
            frame.fill_text(Text {
                content: label,
                position: Point::new(pad_l + width + 8.0, y),
                color: label_color,
                size: Pixels(11.0),
                font: styles::fonts::REGULAR,
                align_x: alignment::Horizontal::Left.into(),
                align_y: alignment::Vertical::Center,
                ..Text::default()
            });
        }

        let to_points = |series: &[f32]| -> Vec<Point> {
            if series.is_empty() {
                return Vec::new();
            }
            let last = (series.len() - 1).max(1) as f32;
            series
                .iter()
                .enumerate()
                .map(|(i, value)| {
                    let x = pad_l + (i as f32) / last * width;
                    let y = pad_t + height - (value / max_speed).clamp(0.0, 1.0) * height;
                    Point::new(x, y)
                })
                .collect()
        };

        // Draw write under download so the orange series reads on top.
        draw_glow_area(
            &mut frame,
            &to_points(&self.write),
            WRITE_COLOR,
            baseline_y,
            pad_l,
        );
        draw_glow_area(
            &mut frame,
            &to_points(&self.download),
            DOWNLOAD_COLOR,
            baseline_y,
            pad_l,
        );

        vec![frame.into_geometry()]
    }
}

/// History values are stored as MB/s; pick a B/s prefix from the scale max.
fn rate_unit(max_mbps: f32) -> (&'static str, f64) {
    let bps = max_mbps as f64 * 1_000_000.0;
    if bps >= 1_000_000_000.0 {
        ("GB/s", 1_000_000_000.0)
    } else if bps >= 1_000_000.0 {
        ("MB/s", 1_000_000.0)
    } else if bps >= 1_000.0 {
        ("KB/s", 1_000.0)
    } else {
        ("B/s", 1.0)
    }
}

fn round_to_10(value: f64) -> f64 {
    (value / 10.0).round() * 10.0
}

fn ceil_to_20(value: f64) -> f64 {
    (value / 20.0).ceil() * 20.0
}

fn draw_glow_area(
    frame: &mut Frame,
    points: &[Point],
    color: Color,
    baseline_y: f32,
    left_x: f32,
) {
    if points.len() < 2 {
        return;
    }

    let last = *points.last().unwrap();

    // Closed angular area under the polyline.
    let area = Path::new(|builder| {
        builder.move_to(points[0]);
        append_polyline(builder, points);
        builder.line_to(Point::new(last.x, baseline_y));
        builder.line_to(Point::new(left_x.max(points[0].x), baseline_y));
        builder.close();
    });

    let fill = gradient::Linear::new(
        Point::new(points[0].x, pad_min_y(points)),
        Point::new(points[0].x, baseline_y),
    )
    .add_stop(
        0.0,
        Color {
            a: 0.48,
            ..color
        },
    )
    .add_stop(
        0.45,
        Color {
            a: 0.2,
            ..color
        },
    )
    .add_stop(
        1.0,
        Color {
            a: 0.0,
            ..color
        },
    );
    frame.fill(&area, fill);

    let line = Path::new(|builder| {
        builder.move_to(points[0]);
        append_polyline(builder, points);
    });

    // Tight neon halo — keep it narrow so corners stay sharp.
    frame.stroke(
        &line,
        Stroke {
            style: stroke::Style::Solid(Color { a: 0.22, ..color }),
            width: 5.0,
            line_cap: stroke::LineCap::Square,
            line_join: stroke::LineJoin::Miter,
            line_dash: stroke::LineDash::default(),
        },
    );
    frame.stroke(
        &line,
        Stroke {
            style: stroke::Style::Solid(Color { a: 0.45, ..color }),
            width: 3.0,
            line_cap: stroke::LineCap::Square,
            line_join: stroke::LineJoin::Miter,
            line_dash: stroke::LineDash::default(),
        },
    );

    // Hard core ridge.
    frame.stroke(
        &line,
        Stroke {
            style: stroke::Style::Solid(color),
            width: 1.6,
            line_cap: stroke::LineCap::Square,
            line_join: stroke::LineJoin::Miter,
            line_dash: stroke::LineDash::default(),
        },
    );
}

fn pad_min_y(points: &[Point]) -> f32 {
    points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min)
}

fn append_polyline(builder: &mut canvas::path::Builder, points: &[Point]) {
    for point in points.iter().skip(1) {
        builder.line_to(*point);
    }
}

impl DownloadPage {
    pub fn view(&self) -> Element<'_, Message> {
        let main = self.view_main_panel();
        let queue = self.view_queue_panel();

        row![
            container(main)
                .width(Length::FillPortion(3))
                .height(Length::Fill),
            container(queue)
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .style(styles::download::queue_panel),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_main_panel(&self) -> Element<'_, Message> {
        let Some(active) = &self.active else {
            return container(
                text("No downloads in progress")
                    .size(28)
                    .color(Color::from_rgb8(180, 180, 180))
                    .font(styles::fonts::REGULAR),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into();
        };

        const HERO_HEIGHT: f32 = 420.0;

        let background_image = container(
            image(active.artwork_path.clone())
                .width(Length::Fill)
                .height(Length::Fixed(HERO_HEIGHT))
                .content_fit(iced::ContentFit::Cover),
        )
        .width(Length::Fill)
        .height(Length::Fixed(HERO_HEIGHT))
        .clip(true);

        let hero_fade = container(Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(HERO_HEIGHT))
            .style(|_theme: &Theme| container::Style {
                background: Some(
                    iced::gradient::Linear::new(iced::Radians(std::f32::consts::PI))
                        .add_stop(0.0, Color::from_rgba8(10, 10, 17, 0.2))
                        .add_stop(0.55, Color::from_rgba8(10, 10, 17, 0.45))
                        .add_stop(1.0, Color::from_rgb8(10, 10, 17))
                        .into(),
                ),
                ..Default::default()
            });

        let store_badge = container(
            text(active.store.to_uppercase())
                .size(13)
                .color(Color::WHITE)
                .font(styles::fonts::REGULAR),
        )
        .padding(iced::Padding::from([5, 10]))
        .style(|_theme: &Theme| container::Style {
            background: Some(Color::from_rgb8(255, 127, 0).into()),
            border: iced::Border {
                radius: styles::radius::SUBTLE.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let progress_pct = format!("{:.0}%", active.progress * 100.0);
        let hero_meta = column![
            text(active.name.clone())
                .size(40)
                .color(Color::WHITE)
                .font(styles::fonts::REGULAR),
            row![
                store_badge,
                text(format!("{}  ·  {}", active.platform, active.store))
                    .size(14)
                    .color(Color::from_rgb8(210, 210, 215)),
            ]
            .spacing(12)
            .align_y(alignment::Vertical::Center),
            column![
                row![
                    text("Progress")
                        .size(12)
                        .color(Color::from_rgb8(180, 180, 180)),
                    Space::new().width(Length::Fill),
                    text(format!(
                        "{} / {}  ·  {}",
                        active.downloaded_label, active.total_label, progress_pct
                    ))
                    .size(12)
                    .color(Color::from_rgb8(220, 220, 220)),
                ],
                {
                    let filled = ((active.progress * 1000.0).round() as u16).max(1);
                    let empty = (1000u16).saturating_sub(filled).max(1);
                    container(
                        row![
                            container(Space::new().height(Length::Fixed(8.0)))
                                .width(Length::FillPortion(filled))
                                .style(styles::download::progress_fill),
                            container(Space::new().height(Length::Fixed(8.0)))
                                .width(Length::FillPortion(empty))
                                .style(styles::download::progress_track),
                        ]
                        .width(Length::Fill),
                    )
                    .width(Length::Fill)
                    .style(styles::download::progress_track)
                },
            ]
            .spacing(8),
        ]
        .spacing(14)
        .padding(
            iced::Padding::new(0.0)
                .right(40.0)
                .bottom(36.0)
                .left(40.0),
        );

        let hero_overlay = container(
            column![Space::new().height(Length::Fill), hero_meta].width(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fixed(HERO_HEIGHT));

        let hero = stack![background_image, hero_fade, hero_overlay]
            .width(Length::Fill)
            .height(Length::Fixed(HERO_HEIGHT));

        let stats = row![
            self.stat_card(
                "Download Speed",
                format!("{:.1} MB/s", active.download_speed_mbps),
                true,
                Length::Fill,
            ),
            self.stat_card(
                "Write Speed",
                format!("{:.1} MB/s", active.write_speed_mbps),
                false,
                Length::Fill,
            ),
            self.stat_card(
                "Time Remaining",
                format_eta(active.eta_secs),
                false,
                Length::Fill,
            ),
        ]
        .spacing(14);

        let details = row![
            self.stat_card(
                "Platform",
                active.platform.clone(),
                false,
                Length::FillPortion(1),
            ),
            self.stat_card(
                "Store",
                active.store.clone(),
                false,
                Length::FillPortion(1),
            ),
            self.stat_card(
                "Download Location",
                active.location.clone(),
                false,
                Length::FillPortion(2),
            ),
        ]
        .spacing(14);

        let legend = row![
            legend_swatch(DOWNLOAD_COLOR, "Download"),
            legend_swatch(WRITE_COLOR, "Write"),
        ]
        .spacing(16);

        let graph = container(
            column![
                row![
                    text("Speed Over Time")
                        .size(16)
                        .color(Color::from_rgb8(200, 200, 200))
                        .font(styles::fonts::SEMIBOLD),
                    Space::new().width(Length::Fill),
                    legend,
                ]
                .align_y(alignment::Vertical::Center),
                canvas::Canvas::new(SpeedGraph {
                    download: self.download_history.clone(),
                    write: self.write_history.clone(),
                })
                .width(Length::Fill)
                .height(Length::Fill),
            ]
            .spacing(12)
            .height(Length::Fill),
        )
        .padding(18)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(styles::download::graph_panel);

        let body = container(
            column![stats, details, graph]
                .spacing(18)
                .height(Length::Fill),
        )
        .padding(
            iced::Padding::new(0.0)
                .top(20.0)
                .right(20.0)
                .bottom(20.0)
                .left(20.0),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(styles::download::body_panel);

        column![hero, body]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_queue_panel(&self) -> Element<'_, Message> {
        let header = column![
            text("Queue")
                .size(22)
                .color(Color::WHITE)
                .font(styles::fonts::REGULAR),
            text(format!("{} items", self.queue.len()))
                .size(13)
                .color(Color::from_rgb8(140, 140, 140)),
        ]
        .spacing(4);

        let items: Element<'_, Message> = column(
            self.queue
                .iter()
                .map(|item| self.queue_item_view(item))
                .collect::<Vec<_>>(),
        )
        .spacing(10)
        .into();

        container(
            column![
                header,
                Space::new().height(Length::Fixed(16.0)),
                scrollable(items).height(Length::Fill),
            ]
            .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn queue_item_view(
        &self,
        item: &crate::gui::pages::download::QueuedItem,
    ) -> Element<'_, Message> {
        let is_selected = self.selected_id == item.id;
        let style = if is_selected {
            styles::download::queue_item_active
        } else {
            styles::download::queue_item
        };

        let status_label = match item.status {
            QueueStatus::Active => ("Downloading", Color::from_rgb8(255, 127, 0)),
            QueueStatus::Queued => ("Queued", Color::from_rgb8(160, 160, 170)),
            QueueStatus::Paused => ("Paused", Color::from_rgb8(255, 204, 0)),
        };

        let progress = if item.progress > 0.0 {
            text(format!("{:.0}%", item.progress * 100.0))
                .size(12)
                .color(Color::from_rgb8(180, 180, 180))
        } else {
            text(item.size_label.clone())
                .size(12)
                .color(Color::from_rgb8(140, 140, 140))
        };

        button(
            column![
                text(item.name.clone())
                    .size(15)
                    .color(Color::WHITE)
                    .font(styles::fonts::MEDIUM),
                row![
                    text(status_label.0)
                        .size(12)
                        .color(status_label.1),
                    Space::new().width(Length::Fill),
                    progress,
                ]
                .align_y(alignment::Vertical::Center),
                text(format!("{} · {}", item.store, item.platform))
                    .size(11)
                    .color(Color::from_rgb8(120, 120, 130)),
            ]
            .spacing(6)
            .width(Length::Fill),
        )
        .on_press(Message::SelectQueueItem(item.id))
        .padding(12)
        .width(Length::Fill)
        .style(style)
        .into()
    }

    fn stat_card<'a>(
        &self,
        label: &'a str,
        value: String,
        emphasize: bool,
        width: Length,
    ) -> Element<'a, Message> {
        let value_color = if emphasize {
            Color::from_rgb8(255, 127, 0)
        } else {
            Color::from_rgb8(230, 230, 230)
        };

        container(
            column![
                text(label)
                    .size(12)
                    .color(Color::from_rgb8(140, 140, 140)),
                text(value)
                    .size(20)
                    .color(value_color)
                    .font(styles::fonts::MEDIUM),
            ]
            .spacing(6),
        )
        .padding(16)
        .width(width)
        .style(styles::download::stat_card)
        .into()
    }
}

fn legend_swatch<'a>(color: Color, label: &'a str) -> Element<'a, Message> {
    row![
        container(Space::new().width(Length::Fixed(10.0)).height(Length::Fixed(10.0))).style(
            move |_theme: &Theme| container::Style {
                background: Some(color.into()),
                border: iced::Border {
                    radius: styles::radius::SUBTLE.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        ),
        text(label)
            .size(12)
            .color(Color::from_rgb8(160, 160, 170)),
    ]
    .spacing(6)
    .align_y(alignment::Vertical::Center)
    .into()
}
