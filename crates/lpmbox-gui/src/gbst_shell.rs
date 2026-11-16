// SPDX-License-Identifier: GPL-3.0-only
// Exact shared visual primitives copied from GBST v2.0.0 and rebound to LPMBox ROM workflows.
// GUI-only: no change to ROM, ADB, SP Flash Tool, firmware or recovery functions.

#[derive(Clone, Copy)]
struct ShellColors {
    page: Color,
    panel: Color,
    sidebar: Color,
    row: Color,
    text: Color,
    muted: Color,
    border: Color,
    accent: Color,
    tint: Color,
    success: Color,
}

// Independently authored Material 3 palette matching LTBox's visible UI.
fn shell_colors(theme: &Theme) -> ShellColors {
    let p = theme.palette();
    let dark = p.background.r + p.background.g + p.background.b < 1.55;
    if dark {
        ShellColors {
            page: Color::from_rgb8(19, 19, 24),
            panel: Color::from_rgb8(32, 31, 38),
            sidebar: Color::from_rgb8(27, 27, 33),
            row: Color::from_rgb8(42, 41, 48),
            text: Color::from_rgb8(228, 225, 233),
            muted: Color::from_rgb8(199, 197, 208),
            border: Color::from_rgb8(71, 70, 79),
            accent: p.primary,
            tint: if p.primary.g > 0.45 { Color::from_rgb8(40, 74, 76) }
                  else if p.primary.r > 0.55 { Color::from_rgb8(83, 48, 66) }
                  else { Color::from_rgb8(67, 69, 89) },
            success: Color::from_rgb8(138, 218, 149),
        }
    } else {
        ShellColors {
            page: Color::from_rgb8(251, 248, 253),
            panel: Color::from_rgb8(239, 236, 241),
            sidebar: Color::from_rgb8(245, 242, 247),
            row: Color::from_rgb8(233, 231, 235),
            text: Color::from_rgb8(27, 27, 33),
            muted: Color::from_rgb8(71, 70, 79),
            border: Color::from_rgb8(199, 197, 208),
            accent: p.primary,
            tint: if p.primary.g > 0.45 { Color::from_rgb8(203, 233, 231) }
                  else if p.primary.r > 0.55 { Color::from_rgb8(255, 215, 230) }
                  else { Color::from_rgb8(224, 225, 249) },
            success: Color::from_rgb8(33, 108, 42),
        }
    }
}

fn shell_border(theme: &Theme, radius: f32) -> iced::Border {
    iced::Border {
        radius: radius.into(),
        width: 1.0,
        color: shell_colors(theme).border,
    }
}

fn shell_main_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: Some(Background::Color(c.page)),
        text_color: Some(c.text),
        ..container::Style::default()
    }
}
fn shell_sidebar_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: Some(Background::Color(c.sidebar)),
        text_color: Some(c.text),
        ..container::Style::default()
    }
}
fn shell_panel_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: Some(Background::Color(c.panel)),
        text_color: Some(c.text),
        border: shell_border(theme, 14.0),
        ..container::Style::default()
    }
}
fn shell_row_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: Some(Background::Color(c.row)),
        text_color: Some(c.text),
        border: iced::Border {
            radius: 7.0.into(),
            ..iced::Border::default()
        },
        ..container::Style::default()
    }
}
fn shell_highlight_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: Some(Background::Color(c.tint)),
        text_color: Some(c.text),
        border: iced::Border {
            radius: 10.0.into(),
            width: 1.0,
            color: Color::from_rgba(c.accent.r, c.accent.g, c.accent.b, 0.48),
        },
        ..container::Style::default()
    }
}
// Semantic warning colors follow LTBox's yellow/amber M3 warning role,
 // not the selected Indigo/Teal/Rose accent. Keep adequate contrast in
 // both light and dark themes.
fn shell_warning_palette(theme: &Theme) -> (Color, Color, Color) {
    let background = theme.palette().background;
    let is_dark = background.r + background.g + background.b < 1.55;
    if is_dark {
        (
            Color::from_rgb8(77, 55, 22),   // warning container
            Color::from_rgb8(255, 227, 168), // on warning container
            Color::from_rgb8(245, 190, 73), // warning accent border
        )
    } else {
        (
            Color::from_rgb8(255, 243, 207),
            Color::from_rgb8(91, 62, 5),
            Color::from_rgb8(181, 127, 8),
        )
    }
}

fn shell_warning_outer_style(theme: &Theme) -> container::Style {
    let (_, _, accent) = shell_warning_palette(theme);
    container::Style {
        background: Some(Background::Color(accent)),
        border: iced::Border {
            radius: 12.0.into(),
            ..iced::Border::default()
        },
        ..container::Style::default()
    }
}

fn shell_warning_inner_style(theme: &Theme) -> container::Style {
    let (surface, foreground, _) = shell_warning_palette(theme);
    container::Style {
        background: Some(Background::Color(surface)),
        text_color: Some(foreground),
        border: iced::Border {
            radius: 11.0.into(),
            ..iced::Border::default()
        },
        ..container::Style::default()
    }
}

fn shell_warning_text_style(theme: &Theme) -> iced::widget::text::Style {
    let (_, foreground, _) = shell_warning_palette(theme);
    iced::widget::text::Style { color: Some(foreground) }
}

fn shell_warning_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let (surface, foreground, _) = shell_warning_palette(theme);
    let disabled = matches!(status, button::Status::Disabled);
    let hovered = matches!(status, button::Status::Hovered);
    let background = if disabled {
        Color::from_rgba(foreground.r, foreground.g, foreground.b, 0.24)
    } else if hovered {
        Color::from_rgba(foreground.r, foreground.g, foreground.b, 0.87)
    } else {
        foreground
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: if disabled { foreground } else { surface },
        border: iced::Border {
            radius: 8.0.into(),
            ..iced::Border::default()
        },
        ..button::Style::default()
    }
}

fn shell_divider_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: Some(Background::Color(c.border)),
        ..container::Style::default()
    }
}
fn shell_footer_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: Some(Background::Color(c.sidebar)),
        text_color: Some(c.muted),
        border: iced::Border {
            width: 1.0,
            color: c.border,
            ..iced::Border::default()
        },
        ..container::Style::default()
    }
}
fn shell_muted<'a>(label: impl Into<String>, size: u32) -> iced::widget::Text<'a> {
    text(label.into())
        .size(size)
        .style(|theme: &Theme| iced::widget::text::Style {
            color: Some(shell_colors(theme).muted),
        })
}
fn shell_action<'a>(
    label: String,
    message: Message,
    enabled: bool,
    primary: bool,
) -> Element<'a, Message> {
    let glyphs = label.chars().count();
    let font_size = if glyphs > 36 { 10 } else if glyphs > 24 { 11 }
        else if glyphs > 16 { 12 } else { 13 };
    let btn = button(
        container(
            text(label).size(font_size).width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
        )
        .width(Length::Fill).height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
    )
    .padding([7.0, 12.0])
    .height(Length::Fixed(56.0))
    .style(move |theme: &Theme, status| {
        let c = shell_colors(theme);
        let hovered = matches!(status, iced::widget::button::Status::Hovered);
        button::Style {
            background: Some(Background::Color(if !enabled {
                c.row
            } else if primary {
                if hovered {
                    Color::from_rgba(c.accent.r, c.accent.g, c.accent.b, 0.87)
                } else {
                    c.accent
                }
            } else if hovered {
                c.tint
            } else {
                c.panel
            })),
            text_color: if !enabled {
                c.muted
            } else if primary {
                Color::WHITE
            } else {
                c.text
            },
            border: iced::Border {
                radius: 11.0.into(),
                width: if primary { 0.0 } else { 1.0 },
                color: if primary {
                    Color::TRANSPARENT
                } else {
                    c.border
                },
            },
            ..button::Style::default()
        }
    });
    if enabled {
        btn.on_press(message).into()
    } else {
        btn.into()
    }
}
fn shell_text_button_style(theme:&Theme,status:button::Status)->button::Style {
    let c=shell_colors(theme);
    let hover=matches!(status, button::Status::Hovered);
    button::Style {
        background: if hover {Some(Background::Color(c.tint))} else {None},
        text_color:c.accent,
        border:iced::Border { radius:8.0.into(), ..Default::default() },
        ..Default::default()
    }
}
fn shell_choice<'a>(label: String, message: Message, selected: bool) -> Element<'a, Message> {
    button(text(label).size(12))
        .padding([10.0, 13.0])
        .style(move |theme: &Theme, status| {
            let c = shell_colors(theme);
            button::Style {
                background: Some(Background::Color(if selected {
                    c.tint
                } else if matches!(status, iced::widget::button::Status::Hovered) {
                    c.row
                } else {
                    c.panel
                })),
                text_color: if selected { c.accent } else { c.text },
                border: iced::Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: if selected { c.accent } else { c.border },
                },
                ..button::Style::default()
            }
        })
        .on_press(message)
        .into()
}

static LPMBOX_ABOUT_APP_ICON: std::sync::LazyLock<iced::widget::image::Handle> =
    std::sync::LazyLock::new(|| iced::widget::image::Handle::from_bytes(APP_ICON_PNG_BYTES));

static GBST_MODEL_UNKNOWN_ICON: std::sync::LazyLock<iced::widget::image::Handle> =
    std::sync::LazyLock::new(|| iced::widget::image::Handle::from_bytes(
        include_bytes!("../assets/models/gbst_Etc.png").as_slice()
    ));
static GBST_MODEL_UNKNOWN_DARK_ICON: std::sync::LazyLock<iced::widget::image::Handle> =
    std::sync::LazyLock::new(|| dark_nav_png_handle_with_white_knockout(
        include_bytes!("../assets/models/gbst_Etc.png"),92,72
    ));

// Stable SVG handles keep texture IDs fixed during the one-second worker ticks.
static ABOUT_GBST_REPO_ICON: std::sync::LazyLock<iced::widget::svg::Handle> =
    std::sync::LazyLock::new(|| iced::widget::svg::Handle::from_memory(
        include_bytes!("../assets/icons/about_lpmbox_repo.svg").as_slice()
    ));
static ABOUT_ISSUE_ICON: std::sync::LazyLock<iced::widget::svg::Handle> =
    std::sync::LazyLock::new(|| iced::widget::svg::Handle::from_memory(
        include_bytes!("../assets/icons/about_issue.svg").as_slice()
    ));
static ABOUT_LTBOX_ICON: std::sync::LazyLock<iced::widget::svg::Handle> =
    std::sync::LazyLock::new(|| iced::widget::svg::Handle::from_memory(
        include_bytes!("../assets/icons/about_ltbox.svg").as_slice()
    ));

fn shell_about_icon_link<'a>(
    glyph: iced::widget::svg::Handle,
    tooltip_text: String,
    message: Message,
) -> Element<'a, Message> {
    let button_icon = button(
        container(iced::widget::svg(glyph)
            .width(Length::Fixed(21.0)).height(Length::Fixed(21.0)))
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(40.0))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .padding(0)
    .on_press(message)
    .style(|theme: &Theme, status| {
        let c = shell_colors(theme);
        let hovered = matches!(status, button::Status::Hovered);
        button::Style {
            background: Some(Background::Color(if hovered { c.tint } else { c.row })),
            text_color: c.text,
            border: iced::Border {
                radius: 10.0.into(),
                ..iced::Border::default()
            },
            ..Default::default()
        }
    });
    iced::widget::tooltip(
        button_icon,
        container(text(tooltip_text).size(11)).padding([6.0, 9.0])
            .style(shell_panel_style),
        iced::widget::tooltip::Position::Top,
    )
    .into()
}

fn shell_settings_card<'a>(
    title: String,
    rows: Vec<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut content = column![
        container(text(title).size(14).font(lpm_bold_font()))
            .width(Length::Fill).padding([14.0, 16.0])
    ].spacing(0);
    for row in rows {
        content = content.push(
            container(Space::new())
                .width(Length::Fill).height(Length::Fixed(1.0))
                .style(shell_divider_style)
        );
        content = content.push(row);
    }
    container(content).width(Length::Fill)
        .style(shell_panel_style).into()
}

fn shell_segment_group<'a>(
    choices: Vec<(String, Message, bool)>,
) -> Element<'a, Message> {
    let mut segments = row![].spacing(0);
    for (label, message, selected) in choices {
        segments = segments.push(
            button(text(label).size(12))
                .height(Length::Fixed(39.0))
                .padding([10.0, 13.0])
                .on_press(message)
                .style(move |theme: &Theme, status| {
                    let c = shell_colors(theme);
                    let hovered = matches!(status, button::Status::Hovered);
                    button::Style {
                        background: Some(Background::Color(if selected {
                            c.tint
                        } else if hovered {
                            c.row
                        } else {
                            c.panel
                        })),
                        text_color: if selected { c.accent } else { c.text },
                        border: iced::Border {
                            width: 0.5,
                            color: c.border,
                            radius: 0.0.into()
                        },
                        ..Default::default()
                    }
                })
        );
    }
    container(segments)
        .style(|theme: &Theme| {
            let c = shell_colors(theme);
            container::Style {
                border: iced::Border {
                    color: c.border,
                    width: 1.0,
                    radius: 20.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

fn settings_path_display(path: &str, max_chars: usize) -> String {
    let chars: Vec<char> = path.chars().collect();
    if chars.len() <= max_chars || max_chars < 7 {
        return path.to_string();
    }
    // Show the Windows drive/volume and the most useful trailing component.
    let head: String = chars.iter().take(3).collect();
    let tail: String = chars[chars.len() - (max_chars - 4)..].iter().collect();
    format!("{head}…{tail}")
}

fn shell_settings_path<'a>(
    path: String,
    open_label: String,
    message: Message,
) -> Element<'a, Message> {
    let shortened = settings_path_display(&path, 25);
    let path_control = container(
        text(shortened)
            .size(11)
            .wrapping(iced::widget::text::Wrapping::None)
    )
    .height(Length::Fixed(40.0))
    .width(Length::Fixed(202.0))
    .padding([12.0, 9.0])
    .clip(true)
    .style(|theme: &Theme| {
        let c = shell_colors(theme);
        container::Style {
            background: Some(Background::Color(c.row)),
            border: iced::Border {
                radius: 7.0.into(),
                color: c.border,
                width: 1.0,
            },
            ..Default::default()
        }
    });
    let path_tooltip = iced::widget::tooltip(
        path_control,
        container(
            text(path).size(11)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
        )
            .padding([8.0, 12.0])
            .max_width(420.0)
            .style(shell_panel_style),
        iced::widget::tooltip::Position::Top,
    );
    row![
        path_tooltip,
        shell_action(open_label, message, true, false)
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

fn shell_detail<'a>(label: String, value: String) -> Element<'a, Message> {
    container(
        row![
            shell_muted(label, 12).width(Length::Fixed(152.0)),
            text(value).size(12).width(Length::Fill),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding([12.0, 16.0])
    .style(shell_detail_row_style)
    .into()
}
fn shell_detail_row_style(theme: &Theme) -> container::Style {
    let c = shell_colors(theme);
    container::Style {
        background: None,
        text_color: Some(c.text),
        border: iced::Border {
            color: c.border,
            width: 0.7,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}
fn shell_setting_row<'a>(
    label: String,
    info: String,
    control: Element<'a, Message>,
) -> Element<'a, Message> {
    container(row![
        column![text(label).size(13), shell_muted(info, 11)]
            .width(Length::Fill).spacing(4),
        control,
    ].spacing(12).align_y(iced::Alignment::Center))
        .width(Length::Fill)
        .padding([13.0, 16.0])
        .style(|theme:&Theme| {
            let c=shell_colors(theme);
            container::Style { text_color:Some(c.text), ..container::Style::default() }
        })
        .into()
}
fn shell_section<'a>(title: String, content: Element<'a, Message>) -> Element<'a, Message> {
    container(column![
        container(text(title).size(14).font(lpm_bold_font()))
            .width(Length::Fill).padding([14.0, 16.0]),
        container(Space::new())
            .width(Length::Fill).height(Length::Fixed(1.0))
            .style(shell_divider_style),
        content,
    ].spacing(0))
        .width(Length::Fill).style(shell_panel_style).into()
}


fn is_compact_window(window_width: f32) -> bool {
    window_width - SIDEBAR_RAIL_WIDTH < EXPANDED_CONTENT_WIDTH
}

fn sidebar_width_for_window(window_width: f32, progress: f32) -> f32 {
    if is_compact_window(window_width) {
        SIDEBAR_RAIL_WIDTH + (SIDEBAR_EXPANDED_WIDTH - SIDEBAR_RAIL_WIDTH)
            * progress.clamp(0.0, 1.0)
    } else {
        SIDEBAR_EXPANDED_WIDTH
    }
}



impl App {
    fn gbst_sidebar_label_alpha(&self) -> f32 {
        if !self.window_is_compact() { return 1.0; }
        let t = ((self.sidebar_anim - 0.4) / 0.5).clamp(0.0, 1.0);
        ease_out_cubic(t)
    }

    fn gbst_sidebar_width(&self) -> f32 {
        sidebar_width_for_window(self.window_size.0, self.sidebar_anim)
    }

    fn gbst_nav_item(&self, page: NavPage, key: &str) -> Element<'_, Message> {
        let palette = self.theme().palette();
        let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.55;
        let handle = if is_dark {
            self.nav_dark_handles.iter()
                .find(|(item,_)| *item==page)
                .map(|(_,handle)|handle.clone())
                .unwrap_or_else(||self.nav_icon_handle(page))
        } else {
            self.nav_icon_handle(page)
        };
        let icon: Element<'_, Message> = iced::widget::image(handle)
            .width(Length::Fixed(21.0))
            .height(Length::Fixed(21.0))
            .into();

        let selected = self.active_nav == page;
        let collapsed = self.window_is_compact() && !self.sidebar_expanded;
        let label_alpha = self.gbst_sidebar_label_alpha();
        let icon_slot: Element<'_, Message> = container(icon)
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .into();
        let mut inner = row![icon_slot]
            .spacing(12)
            .align_y(iced::Alignment::Center);
        if !collapsed && label_alpha > 0.0 {
            let title = self.catalog(key);
            let glyphs = title.chars().count();
            // Use smaller type and allow wrapping when a language has a
            // longer navigation label (Georgian, Greek, Hindi, Arabic, etc.).
            let font_size = if glyphs > 36 { 10 } else if glyphs > 24 { 11 }
                else if glyphs > 17 { 12 } else { 14 };
            let label = text(title)
                .size(font_size)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_y(iced::alignment::Vertical::Center)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
                .style(move |theme: &Theme| {
                    let base = shell_colors(theme).text;
                    iced::widget::text::Style {
                        color: Some(Color::from_rgba(
                            base.r, base.g, base.b, label_alpha
                        ))
                    }
                });
            inner = inner.push(label);
        }

        let content: Element<'_, Message> = container(inner)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Center)
            .clip(true)
            .into();

        let width = if collapsed {
            Length::Fixed(56.0)
        } else {
            Length::Fill
        };
        // The selected indicator has a compact 56x32 pill and a full 56px
        // expanded pill. It is behind the button to prevent icon drift.
        let selection_height = if collapsed { 32.0 } else { 48.0 };
        let selection: Element<'_, Message> = container(
            container(Space::new())
                .width(width)
                .height(Length::Fixed(selection_height))
                .style(move |theme: &Theme| {
                    let c = shell_colors(theme);
                    container::Style {
                        background: if selected {
                            Some(Background::Color(c.tint))
                        } else {
                            None
                        },
                        border: iced::Border {
                            radius: 28.0.into(),
                            ..iced::Border::default()
                        },
                        ..container::Style::default()
                    }
                })
        )
        .width(width)
        .height(Length::Fixed(48.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into();

        let button: Element<'_, Message> = button(content)
            .padding([0.0, if collapsed { 16.0 } else { 20.0 }])
            .width(width)
            .height(Length::Fixed(48.0))
            .style(move |theme: &Theme, state| {
                let c = shell_colors(theme);
                button::Style {
                    background: if matches!(state, button::Status::Hovered) {
                        Some(Background::Color(Color::from_rgba(
                            c.text.r, c.text.g, c.text.b, 0.07
                        )))
                    } else {
                        None
                    },
                    text_color: c.text,
                    border: iced::Border {
                        radius: 28.0.into(),
                        ..iced::Border::default()
                    },
                    ..button::Style::default()
                }
            })
            .on_press(Message::SelectNav(page))
            .into();

        iced::widget::Stack::with_children(vec![selection, button])
            .width(width)
            .height(Length::Fixed(48.0))
            .into()
    }

    fn gbst_sidebar(&self) -> Element<'_, Message> {
        let compact = self.window_is_compact();
        let collapsed = compact && !self.sidebar_expanded;
        let alpha = self.gbst_sidebar_label_alpha();

        // Keep the same menu item and section heights during hover
        // transitions. Never recenter the icons when labels appear.
        let section: Element<'_, Message> = if collapsed {
            container(
                container(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fixed(1.0))
                    .style(shell_divider_style)
            )
            .width(Length::Fill)
            .height(Length::Fixed(48.0))
            .align_y(iced::alignment::Vertical::Center)
            .into()
        } else {
            container(
                shell_muted(self.catalog("ui.group_tools"), 11)
                    .wrapping(iced::widget::text::Wrapping::None)
                    .style(move |theme: &Theme| {
                        let c = shell_colors(theme).muted;
                        iced::widget::text::Style {
                            color: Some(Color::from_rgba(c.r, c.g, c.b, alpha))
                        }
                    })
            )
            .width(Length::Fill)
            .height(Length::Fixed(48.0))
            .padding([0.0, 20.0])
            .align_y(iced::alignment::Vertical::Center)
            .into()
        };

        let list = column![
            self.gbst_nav_item(NavPage::Dashboard, "ui.dashboard"),
            self.gbst_nav_item(NavPage::Rom, "lpm.nav.rom"),
            // Root and Unroot are deliberately absent from the active UI.
            // Internal implementations are retained for future compatibility.
            self.gbst_nav_item(NavPage::Driver, "lpm.nav.driver"),
            section,
            // Firmware download remains internal; omit it from the active sidebar.
            self.gbst_nav_item(NavPage::Qna, "lpm.nav.qna"),
            self.gbst_nav_item(NavPage::Log, "ui.logs"),
            self.gbst_nav_item(NavPage::Settings, "ui.settings"),
            self.gbst_nav_item(NavPage::About, "ui.about"),
        ]
        .spacing(1)
        .padding([12.0, if collapsed { 12.0 } else { 8.0 }])
        .align_x(iced::Alignment::Start);

        let panel_width = self.gbst_sidebar_width();
        // At the minimum 820x720 window the six navigation entries fit.
        // A Scrollable wrapper creates an unwanted rail on the side menu.
        let panel = container(
            list.width(Length::Fill)
        )
        .width(Length::Fixed(panel_width))
        .height(Length::Fill)
        .style(shell_sidebar_style);

        let edge = container(Space::new())
            .height(Length::Fill)
            .width(Length::Fixed(1.0))
            .style(shell_divider_style);
        let mut shell = row![panel, edge].height(Length::Fill);
        if compact && self.sidebar_anim > 0.0 {
            let fade = self.sidebar_anim;
            shell = shell.push(
                container(Space::new())
                    .width(Length::Fixed(14.0 * fade))
                    .height(Length::Fill)
                    .style(move |theme: &Theme| {
                        let fg = shell_colors(theme).text;
                        container::Style {
                            background: Some(Background::Color(
                                Color::from_rgba(fg.r, fg.g, fg.b, 0.07 * fade)
                            )),
                            ..container::Style::default()
                        }
                    })
            );
        }

        if compact {
            iced::widget::mouse_area(shell)
                .on_enter(Message::SidebarHoverEnter)
                .on_exit(Message::SidebarHoverExit)
                .on_press(Message::Noop)
                .interaction(iced::mouse::Interaction::Idle)
                .into()
        } else {
            shell.into()
        }
    }
    // Country confirmation needs taller multilingual buttons than the 40px
    // dashboard action pills; word breaks must not draw over the popup footer.
    fn gbst_modal_action(
        &self,
        label: String,
        message: Message,
        enabled: bool,
        primary: bool,
    ) -> Element<'static, Message> {
        let content = container(
            text(label).size(12).width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
        )
        .width(Length::Fill).height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);
        let action = button(content).padding([7.0, 10.0])
            .width(Length::Fixed(136.0)).height(Length::Fixed(64.0))
            .style(move |theme: &Theme, status| {
                let c = shell_colors(theme);
                let hovered = matches!(status, button::Status::Hovered);
                button::Style {
                    background: Some(Background::Color(if !enabled { c.row }
                        else if primary { c.accent }
                        else if hovered { c.tint } else { c.panel })),
                    text_color: if !enabled { c.muted }
                        else if primary { Color::WHITE } else { c.text },
                    border: iced::Border {
                        radius: 11.0.into(),
                        width: if primary { 0.0 } else { 1.0 },
                        color: c.border
                    },
                    ..button::Style::default()
                }
            });
        if enabled { action.on_press(message).into() } else { action.into() }
    }

    fn gbst_page_header(&self, title_key: &str, subtitle_key: &str) -> Element<'_, Message> {
        let title = lpm_translate_owned(title_key.to_string());
        let subtitle = lpm_translate_owned(subtitle_key.to_string());
        let compact = self.window_is_compact();
        let title_text = text(title).size(if compact { 20 } else { 24 })
            .font(lpm_bold_font())
            .width(if compact { Length::Fill } else { Length::Shrink })
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph);
        let subtitle_text = shell_muted(subtitle, 12)
            .width(Length::Fill)
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph);
        let heading: Element<'_, Message> = if subtitle_key.trim().is_empty() {
            title_text.into()
        } else if compact {
            column![title_text, subtitle_text].spacing(4).width(Length::Fill).into()
        } else {
            row![title_text, subtitle_text].spacing(12).align_y(iced::Alignment::Center).into()
        };
        container(heading)
        .width(Length::Fill)
        .height(if compact { Length::Shrink } else { Length::Fixed(64.0) })
        .padding([12.0, 24.0])
        .style(|theme: &Theme| {
            let c = shell_colors(theme);
            container::Style {
                background: Some(Background::Color(c.sidebar)),
                text_color: Some(c.text),
                border: iced::Border { width: 1.0, color: c.border,
                    ..iced::Border::default() },
                ..container::Style::default()
            }
        })
        .into()
    }
    fn gbst_dashboard_action_card(
        &self,
        heading_key: &str,
        description_key: &str,
        button_key: &str,
        action: Message,
        primary: bool,
    ) -> Element<'_, Message> {
        let body = column![
            container(
                text(self.catalog(heading_key))
                    .size(17)
                    .font(lpm_bold_font())
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
                    .align_x(iced::alignment::Horizontal::Center)
            )
            // Keep every card title centered on the same horizontal row,
            // regardless of whether it wraps to one or two lines.
            .width(Length::Fill)
            .height(Length::Fixed(72.0))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
            Space::new().height(Length::Fill),
            container(
                shell_muted(self.catalog(description_key), 13)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
                    .align_x(iced::alignment::Horizontal::Center),
            ).width(Length::Fill).align_x(iced::alignment::Horizontal::Center),
            Space::new().height(Length::Fill),
            shell_action(self.catalog(button_key), action, true, primary),
        ]
        .spacing(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::Alignment::Center);
        container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([18.0, 14.0])
            .style(shell_panel_style)
            .into()
    }


// Expose the two-column dashboard at 1056 px with either compact or
// expanded sidebar, but keep labels readable when the viewport is narrower.
fn shell_device_details_two_columns(window_width: f32, sidebar_width: f32) -> bool {
    window_width - sidebar_width >= 785.0
}

    fn gbst_device_card(&self) -> Element<'_, Message> {
        let d = &self.dashboard_info;
        let connected = d.connected;
        let model = if connected && !d.model_name.trim().is_empty() {
            d.model_name.clone()
        } else {
            self.catalog("ui.device_not_found")
        };
        let subtitle = if connected {
            format!("Lenovo · Android {}", d.android_version)
        } else {
            "USB ADB".to_owned()
        };
        let palette = self.theme().palette();
        let is_dark = palette.background.r + palette.background.g + palette.background.b < 1.55;
        let image_handle = if connected {
            self.dashboard_model_image_handle()
        } else if is_dark {
            GBST_MODEL_UNKNOWN_DARK_ICON.clone()
        } else {
            GBST_MODEL_UNKNOWN_ICON.clone()
        };
        let image = iced::widget::image(image_handle)
            .width(Length::Fixed(92.0)).height(Length::Fixed(72.0));
        let portrait = row![
            container(image)
                .width(Length::Fixed(100.0)).height(Length::Fixed(82.0))
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
            column![
                text(model).size(16).font(lpm_bold_font()),
                shell_muted(subtitle, 14),
            ].spacing(6).width(Length::Fill),
        ].spacing(15).align_y(iced::Alignment::Center);
        // Values are sourced from the existing DashboardDeviceInfo and the
        // original LPMBox v3.1.2 ADB snapshot. No additional device commands.
        let original_rom = lpm_translate_owned(d.original_rom.clone());
        let installed_rom = lpm_translate_owned(d.installed_rom.clone());
        let ota_status = match d.ota_status.as_str() {
            "차단 필요" => self.catalog("lpm.device.ota.block_required"),
            "차단 완료" => self.catalog("lpm.device.ota.blocked"),
            "사용 중" => self.catalog("lpm.device.ota.in_use"),
            "사용 하지 않음" => self.catalog("lpm.device.ota.not_in_use"),
            "감지됨" => self.catalog("lpm.device.ota.detected"),
            _ => lpm_translate_owned(d.ota_status.clone()),
        };

        let left_details = [
            (self.catalog("ui.device.model"), d.model_name.clone()),
            (self.catalog("ui.device.android"), d.android_version.clone()),
            (self.catalog("ui.device.rom"), installed_rom),
            (self.catalog("lpm.device.firmware"), d.firmware_version.clone()),
        ];
        let right_details = [
            (self.catalog("lpm.device.original_rom"), original_rom),
            (self.catalog("lpm.device.slot_suffix"), d.slot_suffix.clone()),
            (self.catalog("lpm.device.country_code"), d.country_code.clone()),
            (self.catalog("lpm.device.ota_status"), ota_status),
        ];

        // Keep the four original rows on the left and the four requested rows
        // on the right at normal widths. Stack at narrow window/sidebar sizes
        // to avoid clipping in longer locales such as Georgian and Arabic.
        let render_item = |label: String, value: String| -> Element<'static, Message> {
            container(
                row![
                    shell_muted(label, 11)
                        .width(Length::Fixed(126.0))
                        .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
                    text(value).size(12).width(Length::Fill)
                        .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
                ].spacing(8).width(Length::Fill)
                    .align_y(iced::Alignment::Center)
            ).padding([8.0, 0.0]).width(Length::Fill).into()
        };
        let separator = || {
            container(Space::new())
                .width(Length::Fill).height(Length::Fixed(1.0))
                .style(shell_divider_style)
        };
        let mut detail_grid = column![].spacing(0).width(Length::Fill);
        if Self::shell_device_details_two_columns(self.window_size.0, self.gbst_sidebar_width()) {
            for ((left_label, left_value), (right_label, right_value)) in
                left_details.into_iter().zip(right_details.into_iter())
            {
                detail_grid = detail_grid.push(separator());
                detail_grid = detail_grid.push(
                    row![
                        render_item(left_label, left_value),
                        render_item(right_label, right_value),
                    ].spacing(18).width(Length::Fill)
                        .align_y(iced::Alignment::Center)
                );
            }
        } else {
            for (label, value) in left_details.into_iter().chain(right_details.into_iter()) {
                detail_grid = detail_grid.push(separator());
                detail_grid = detail_grid.push(render_item(label, value));
            }
        }
        container(column![portrait, detail_grid].spacing(8))
            .width(Length::Fill).padding(20).style(shell_panel_style).into()
    }

    fn gbst_dashboard_panel(&self) -> Element<'_, Message> {
        let cards = row![
            self.gbst_dashboard_action_card(
                "ui.dashboard.donate_title","lpm.dashboard.donate_desc",
                "ui.dashboard.go",Message::OpenDonate,true),
            self.gbst_dashboard_action_card(
                "lpm.dashboard.rom_title","lpm.dashboard.rom_desc",
                "ui.dashboard.start",Message::SelectNav(NavPage::Rom),true),
            self.gbst_dashboard_action_card(
                "ui.dashboard.youtube_title","ui.dashboard.youtube_desc",
                "ui.dashboard.go",Message::OpenDeveloperYoutube,true),
        ].spacing(8).width(Length::Fill).height(Length::Fill);
        // Like LTBox's Dashboard, a blocking ADB server warning is the first
        // content item, above the device summary. The action stays opt-in and
        // still calls LPMBox's original host:kill worker.
        let mut content = column![].spacing(12)
            .height(Length::Fill).width(Length::Fill);
        if self.external_adb_server_active {
            let warning_label = self.catalog("lpm.adb.title");
            let warning_message = self.catalog("lpm.adb.conflict");
            let action_label = self.catalog(if self.external_adb_kill_in_flight {
                "lpm.adb.killing"
            } else {
                "lpm.adb.kill"
            });
            let enabled = !self.busy && !self.external_adb_kill_in_flight;
            let action = button(
                text(action_label).size(12)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
            )
            .height(Length::Fixed(42.0))
            .padding([8.0, 16.0])
            .style(shell_warning_button_style);
            let action: Element<'_, Message> = if enabled {
                action.on_press(Message::KillExternalAdbServer).into()
            } else {
                action.into()
            };
            let copy = column![
                text(warning_label).size(14).font(lpm_bold_font())
                    .style(shell_warning_text_style)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
                text(warning_message).size(12)
                    .style(shell_warning_text_style)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
                    .width(Length::Fill)
            ].spacing(5).width(Length::Fill);
            let warning_row = row![
                text("⚠").size(21).style(shell_warning_text_style),
                copy,
                action
            ].spacing(13).width(Length::Fill)
                .align_y(iced::Alignment::Center);
            // LTBox has a thicker amber left border which curves around the
            // box. Nesting two containers reproduces it without a square strip.
            let warning_card = container(
                container(warning_row)
                    .width(Length::Fill)
                    .padding([12.0, 16.0])
                    .style(shell_warning_inner_style)
            )
            .width(Length::Fill)
            .padding(iced::Padding {
                top: 1.0, right: 1.0, bottom: 1.0, left: 3.0
            })
            .style(shell_warning_outer_style);
            content = content.push(warning_card);
        }
        content = content.push(self.gbst_device_card());
        content = content.push(container(cards).width(Length::Fill).height(Length::Fill));
        container(content).width(Length::Fill).height(Length::Fill).padding(20).into()
    }

    fn gbst_footer(&self) -> Element<'_, Message> {
        let connected = self.dashboard_info.connected;
        let status = if self.busy {
            self.catalog("lpm.footer.busy")
        } else if connected {
            format!("{}  ·  {}",self.catalog("ui.footer.connected"),
                    self.dashboard_info.model_name)
        } else {
            self.catalog("ui.footer.disconnected")
        };
        container(row![
            text("●").size(11).style(move |theme: &Theme| iced::widget::text::Style {
                color: Some(if connected {
                    shell_colors(theme).success
                } else { shell_colors(theme).muted })
            }),
            shell_muted(status, 11).width(Length::Fill),
            shell_muted(format!("{APP_DISPLAY_VERSION}"), 11),
        ].spacing(8).align_y(iced::Alignment::Center))
            .width(Length::Fill).padding([8.0, 15.0])
            .style(shell_footer_style).into()
    }

    // Retain the exact worker-reported percentage from the most recent SPFT
    // stage; never infer progress from elapsed time or the seven-step wizard.
    fn latest_spft_progress(&self) -> Option<(String, u8)> {
        self.progress_line_indices.iter()
            .filter_map(|(key, index)| {
                let message = self.log_lines.get(*index)?.message.as_str();
                let (_, percent) = message.rsplit_once(" (")?;
                let percent = percent.strip_suffix("%)")?.parse::<u8>().ok()?.min(100);
                Some((*index, key.strip_prefix("spft:")?.to_string(), percent))
            })
            .max_by_key(|(index, _, _)| *index)
            .map(|(_, stage, percent)| (stage, percent))
    }

    fn gbst_log_panel(&self) -> Element<'_, Message> {
        let header = row![
            text(self.catalog("ui.exec.log_title")).size(14)
                .font(lpm_bold_font()).width(Length::Fill),
            button(text(self.catalog("ui.save_log")).size(12))
                .on_press(Message::ExportLog).style(shell_text_button_style),
            button(text(self.catalog("lpm.logs.clear")).size(12))
                .on_press(Message::ClearLog).style(shell_text_button_style),
        ].spacing(8).align_y(iced::Alignment::Center);
        let start = self.log_lines.len().saturating_sub(300);
        let content = self.log_lines.iter().skip(start)
            .map(|entry| format!("[{}] {}",entry.timestamp,
                lpm_translate_owned(entry.message.clone())))
            .collect::<Vec<_>>().join("\n");
        // Use glyph-safe wrapping for non-spaced file paths and keep a
        // dedicated inner gutter clear of the scroll rail. Do NOT alter
        // stored/exported log lines.
        let body = scrollable(
            container(raw_iced_text(content)
                .size(LOG_FONT)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
                .width(Length::Fill))
                .width(Length::Fill)
                .padding(iced::Padding {
                    top: 7.0, right: 48.0, bottom: 7.0, left: 8.0
                })
        ).direction(iced::widget::scrollable::Direction::Vertical(
            iced::widget::scrollable::Scrollbar::new()
                .width(8.0).scroller_width(6).spacing(16.0),
        )).id(iced::widget::Id::new("lpmbox-rom-work-history"))
         .auto_scroll(true).anchor_top().anchor_left()
         .width(Length::Fill).height(Length::Fill);
        let mut layout = column![header].spacing(8);
        if let Some((stage, percent)) = self.latest_spft_progress() {
            let progress = iced::widget::progress_bar(0.0..=100.0, percent as f32)
                .girth(6)
                .style(|theme: &Theme| {
                    let c = shell_colors(theme);
                    iced::widget::progress_bar::Style {
                        background: c.row.into(),
                        bar: c.accent.into(),
                        border: iced::Border { radius: 8.0.into(), ..Default::default() },
                    }
                });
            let now = row![
                text(stage).size(12).width(Length::Fill),
                text(format!("{percent}%")).size(12).font(lpm_bold_font()),
            ].spacing(12).align_y(iced::Alignment::Center);
            layout = layout.push(container(column![now, progress].spacing(8))
                .width(Length::Fill).padding([8, 4]));
        }
        container(layout.push(body))
            .width(Length::Fill).height(Length::Fill)
            .padding(15).style(shell_panel_style).into()
    }

    fn gbst_rom_current_step(&self) -> Option<usize> {
        // Only count worker milestones from the CURRENT task, not old logs.
        // Keep the highest reached step: nested backup routines may emit their
        // own "[1단계]" labels after a parent ROM flow reached "[2단계]".
        let mut reached: Option<usize> = None;
        for log in self.log_lines.iter().skip(self.runtime_log_start_index) {
            let msg = log.message.as_str();
            for index in (1..=7).rev() {
                if msg.contains(&format!("[{index}단계]")) {
                    reached = Some(reached.unwrap_or(0).max(index - 1));
                    break;
                }
            }
        }
        reached
    }

    fn gbst_execution_elapsed(&self) -> String {
        let seconds = self.runtime_finished_elapsed
            .or_else(|| self.runtime_started_at.map(|start| start.elapsed()))
            .map(|duration| duration.as_secs()).unwrap_or(0);
        format!("{:02}:{:02}",seconds / 60,seconds % 60)
    }

    fn gbst_execution_overall_progress(&self) -> f32 {
        // A ROM stage is a verified milestone, not a time/byte estimate.
        // Never show 100% until the worker emits its explicit success message.
        if self.runtime_last_outcome == Some(true) { return 100.0; }
        let reached = self.gbst_rom_current_step().unwrap_or(0).min(6);
        (reached as f32 * 100.0 / 7.0).min(99.0)
    }

    fn gbst_execution_summary(&self) -> Element<'_, Message> {
        let phase = self.gbst_rom_current_step().unwrap_or(0);
        let title = self.catalog(&format!("lpm.rom.phase.{phase}"));
        let overall = self.gbst_execution_overall_progress();
        let status = if self.busy {
            self.catalog("lpm.exec.status.running")
        } else if self.runtime_last_outcome == Some(true) {
            self.catalog("lpm.exec.status.completed")
        } else if self.runtime_last_outcome == Some(false) {
            self.catalog("lpm.exec.status.failed")
        } else {
            self.catalog("lpm.exec.status.ready")
        };
        let connection = if self.log_lines.iter().skip(self.runtime_log_start_index)
            .any(|row| row.message.contains("PreLoader 감지:"))
        {
            self.catalog("lpm.exec.connection.preloader")
        } else {
            self.catalog("lpm.exec.connection.adb")
        };
        let stats = row![
            shell_detail(self.catalog("ui.exec.connection"),connection),
            shell_detail(self.catalog("lpm.exec.milestones"),
                format!("{}/7",if self.runtime_last_outcome == Some(true) {7}else{phase})),
            shell_detail(self.catalog("ui.exec.elapsed"),self.gbst_execution_elapsed()),
        ].spacing(2).width(Length::Fill);
        let progress = iced::widget::progress_bar(0.0..=100.0,overall)
            .length(Length::Fill).girth(Length::Fixed(7.0))
            .style(|theme: &Theme| {
                let c = shell_colors(theme);
                iced::widget::progress_bar::Style {
                    background: c.row.into(),
                    bar: c.accent.into(),
                    border: iced::Border {radius: 8.0.into(), ..Default::default()},
                }
            });
        let mut body = column![
            row![
                text(title).size(16).font(lpm_bold_font()).width(Length::Fill)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
                text(format!("{overall:.1}%")).size(16).font(lpm_bold_font()),
            ].align_y(iced::Alignment::Center).spacing(10),
            shell_muted(status,12),
            progress,
            container(stats).width(Length::Fill).padding([5.0,1.0])
                .style(shell_detail_row_style),
        ].spacing(8);
        if let Some((stage,percent)) = self.latest_spft_progress() {
            let progress = iced::widget::progress_bar(0.0..=100.0,percent as f32)
                .length(Length::Fill).girth(Length::Fixed(5.0))
                .style(|theme: &Theme| {
                    let c = shell_colors(theme);
                    iced::widget::progress_bar::Style {
                        background: c.row.into(), bar: c.accent.into(),
                        border: iced::Border {radius: 8.0.into(),..Default::default()},
                    }
                });
            body = body.push(
                column![
                    row![
                        shell_muted(format!("{}: {}",self.catalog("lpm.exec.stage_progress"),stage),11)
                            .width(Length::Fill),
                        text(format!("{percent}%")).size(11).font(lpm_bold_font()),
                    ].align_y(iced::Alignment::Center),
                    progress,
                ].spacing(5)
            );
        }
        container(body).width(Length::Fill).padding(15)
            .style(shell_panel_style).into()
    }

    fn gbst_execution_timeline(&self, compact: bool) -> Element<'_, Message> {
        let phase = self.gbst_rom_current_step().unwrap_or(0);
        let success = self.runtime_last_outcome == Some(true);
        let failed = self.runtime_last_outcome == Some(false);
        let mut lines = column![].spacing(5);
        for idx in 0..7 {
            let label = self.catalog(&format!("lpm.rom.phase.{idx}"));
            let current = !success && phase == idx;
            let finished = success || phase > idx;
            let symbol = if finished {"✓".to_owned()} else {format!("{}",idx+1)};
            let circle = container(text(symbol).size(12))
                .width(Length::Fixed(24.0)).height(Length::Fixed(24.0))
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .style(move |theme: &Theme| {
                    let c = shell_colors(theme);
                    let selected = if failed && current {
                        Color::from_rgb8(193,52,58)
                    } else {c.accent};
                    container::Style {
                        background: Some(Background::Color(
                            if current || finished {selected} else {c.row}
                        )),
                        text_color: Some(if current || finished {Color::WHITE} else {c.muted}),
                        border: iced::Border {radius: 13.0.into(), ..Default::default()},
                        ..Default::default()
                    }
                });
            lines = lines.push(
                row![circle,text(label).size(12).width(Length::Fill)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph)]
                .spacing(9).align_y(iced::Alignment::Center)
                .width(Length::Fill).padding([6.0,4.0])
            );
        }
        container(lines).padding(13)
            .width(if compact {Length::Fill} else {Length::Fixed(270.0)})
            .height(if compact {Length::Shrink} else {Length::Fill})
            .style(shell_panel_style).into()
    }

    fn gbst_rom_execution_panel(&self) -> Element<'_, Message> {
        let compact = self.window_is_compact();
        let logs: Element<'_, Message> = if compact {
            column![
                self.gbst_execution_timeline(true),
                self.gbst_log_panel(),
            ].spacing(10).width(Length::Fill).height(Length::Fill).into()
        } else {
            row![self.gbst_execution_timeline(false),self.gbst_log_panel()]
                .spacing(14).width(Length::Fill).height(Length::Fill).into()
        };
        container(column![self.gbst_execution_summary(),logs]
            .spacing(12).height(Length::Fill))
            .width(Length::Fill).height(Length::Fill).padding(16).into()
    }

}


impl App {
    fn gbst_settings_panel(&self) -> Element<'_, Message> {
        let language = pick_list(
            LANGUAGE_OPTIONS,
            Some(self.settings_language),
            Message::SettingsLanguageSelected
        ).text_size(13)
         .style(lpm_nav_language_pick_list_style)
         .menu_style(lpm_nav_language_pick_list_menu_style)
         .width(Length::Fixed(176.0));
        let theme = shell_segment_group(
            ThemeChoice::ALL.iter().copied()
                .map(|choice| (
                    self.catalog(&format!("ui.theme.{}",choice.code())),
                    Message::SettingsThemeSelected(choice),
                    self.appearance.theme == choice
                )).collect()
        );
        let accent = shell_segment_group(
            AccentChoice::ALL.iter().copied()
                .map(|choice| (
                    self.catalog(&format!("ui.accent.{}",choice.code())),
                    Message::SettingsAccentSelected(choice),
                    self.appearance.accent == choice
                )).collect()
        );
        let appearance = shell_settings_card(self.catalog("ui.settings.appearance"),vec![
            shell_setting_row(
                self.catalog("ui.settings.language"),String::new(),language.into()),
            shell_setting_row(
                self.catalog("ui.settings.theme"),String::new(),theme),
            shell_setting_row(
                self.catalog("ui.settings.accent"),
                self.catalog("ui.settings.accent_desc"),accent),
        ]);
        let connection = shell_settings_card(self.catalog("ui.settings.connection"),vec![
            shell_setting_row(
                self.catalog("lpm.settings.driver"),
                self.catalog("lpm.settings.driver_desc"),
                shell_action(self.catalog("ui.settings.open"),
                    Message::SelectNav(NavPage::Driver),true,false))
        ]);
        let files = shell_settings_card(self.catalog("ui.settings.files"),vec![
            shell_setting_row(
                self.catalog("lpm.settings.downloadfolder"),
                self.catalog("lpm.settings.downloadfolder_desc"),
                shell_settings_path(
                    lpmbox_core::app_paths::tool_download_dir()
                        .display().to_string(),
                    self.catalog("ui.settings.open"),
                    Message::OpenDownloadsFolder)),
            shell_setting_row(
                self.catalog("ui.settings.logfolder"),
                self.catalog("ui.settings.logs_desc"),
                shell_settings_path(
                    lpmbox_core::app_paths::log_dir().display().to_string(),
                    self.catalog("ui.settings.open"),
                    Message::OpenLogFolder)),
            shell_setting_row(
                self.catalog("cache.title"),
                self.catalog("cache.description"),
                shell_action(
                    self.catalog("cache.clean"),
                    Message::RequestCacheCleanup,!self.busy,false)),
            shell_setting_row(
                self.catalog("logs.cleanup"),
                self.catalog("logs.cleanup.description"),
                shell_action(
                    self.catalog("logs.cleanup.confirm"),
                    Message::RequestSavedLogCleanup,!self.busy,false))
        ]);
        let program = shell_settings_card(self.catalog("ui.settings.general"),vec![
            shell_setting_row(
                self.catalog("ui.settings.updates"),
                format!("{APP_DISPLAY_VERSION}"),
                shell_action(self.catalog("ui.settings.check"),
                    Message::CheckProgramUpdate,!self.program_update_checking,false)),
            shell_setting_row(
                self.catalog("ui.settings.feedback"),
                String::new(),
                shell_action(self.catalog("ui.settings.open"),
                    Message::OpenFeedback,true,false)),
            shell_setting_row(
                self.catalog("ui.about.license"),
                self.catalog("about.license"),
                shell_action(self.catalog("ui.about.license"),
                    Message::OpenLicenseDialog,true,false)),
        ]);
        let content = column![appearance,connection,files,program]
            .spacing(18).width(Length::Fill);
        let centered = container(content)
            .width(Length::Fill).max_width(840.0);
        container(scrollable(
            container(centered).width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .padding([24.0,22.0])
        ).height(Length::Fill))
            .width(Length::Fill).height(Length::Fill).into()
    }

    fn gbst_about_panel(&self) -> Element<'_, Message> {
        let icon = iced::widget::image(LPMBOX_ABOUT_APP_ICON.clone())
            .width(Length::Fixed(78.0)).height(Length::Fixed(78.0));
        let links = row![
            shell_about_icon_link(
                ABOUT_GBST_REPO_ICON.clone(),
                self.catalog("ui.about.source"),
                Message::OpenSourceRepository),
            shell_about_icon_link(
                ABOUT_ISSUE_ICON.clone(),
                self.catalog("ui.about.issues"),
                Message::OpenFeedback),
            shell_about_icon_link(
                ABOUT_LTBOX_ICON.clone(),
                self.catalog("ui.about.ltbox_github"),
                Message::OpenLTBoxRepository),
        ].spacing(12).align_y(iced::Alignment::Center);
        let licenses = button(text(self.catalog("ui.about.license")).size(13))
            .on_press(Message::OpenLicenseDialog)
            .padding([12.0,17.0])
            .style(shell_text_button_style);
        let about = column![
            icon,
            text("LPMBox").size(26).font(lpm_bold_font()),
            shell_muted(self.catalog("lpm.about.subtitle"),13),
            shell_muted(self.catalog("lpm.about.tagline"),12)
                .align_x(iced::alignment::Horizontal::Center),
            shell_muted(self.catalog("ui.about.ltbox_credit"),12)
                .align_x(iced::alignment::Horizontal::Center),
            shell_muted(format!("{APP_DISPLAY_VERSION}"),12),
            links,
            licenses,
        ].spacing(13).align_x(iced::Alignment::Center);
        container(about).width(Length::Fill).height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .padding(22).into()
    }

    fn gbst_license_popup(&self) -> Element<'_, Message> {
        // Display LPMBox's GNU GPLv3 license text. Historical and
        // third-party license documents remain separately preserved in
        // THIRD_PARTY_LICENSES and are not relicensed by this dialog.
        const OWN_LICENSE: &str = include_str!("../../../LICENSE");
        let license_row = |name: &'static str, license: String| {
            column![
                text(name).size(13).font(lpm_bold_font()).width(Length::Fill),
                shell_muted(license,12).width(Length::Fill)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
            ].spacing(3).width(Length::Fill)
        };
        let index = column![
            text(self.catalog("ui.about.licenses_section"))
                .size(13).font(lpm_bold_font()),
            license_row("LPMBox","GNU GPL v3.0 (GPL-3.0-only)".to_owned()),
            shell_muted(self.catalog("lpm.about.third_party_notice"),11)
                .width(Length::Fill)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
        ].spacing(10).width(Length::Fill);
        let licenses = scrollable(
            column![
                index,
                container(Space::new()).height(Length::Fixed(1.0))
                    .width(Length::Fill).style(shell_divider_style),
                text("LPMBox · GNU GPLv3").size(13).font(lpm_bold_font()),
                raw_iced_text(OWN_LICENSE).size(11).width(Length::Fill)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
            ].spacing(16).width(Length::Fill)
                .padding(iced::Padding {top:12.0,right:16.0,bottom:12.0,left:12.0})
        ).direction(iced::widget::scrollable::Direction::Vertical(
            iced::widget::scrollable::Scrollbar::new()
                .width(8.0).scroller_width(6).spacing(12.0),
        )).width(Length::Fill).height(Length::Fill);
        let body = column![
            text(self.catalog("ui.about.license")).size(22).font(lpm_bold_font()),
            container(Space::new()).width(Length::Fill)
                .height(Length::Fixed(1.0)).style(shell_divider_style),
            licenses,
            container(Space::new()).width(Length::Fill)
                .height(Length::Fixed(1.0)).style(shell_divider_style),
            row![
                Space::new().width(Length::Fill),
                shell_action(self.catalog("ui.about.license_close"),
                    Message::CloseLicenseDialog,true,false),
            ],
        ].spacing(12).height(Length::Fill);
        let popup = container(body)
            .width(Length::Fixed(520.0)).height(Length::Fixed(590.0))
            .padding(22).style(|theme: &Theme| {
                let mut s = shell_panel_style(theme);
                s.border.radius = 26.0.into();
                s.shadow = iced::Shadow{
                    color: Color::from_rgba(0.0,0.0,0.0,0.25),
                    offset: iced::Vector::new(0.0,8.0),
                    blur_radius:24.0
                };
                s
            });
        let scrim = container(Space::new())
            .width(Length::Fill).height(Length::Fill)
            .style(|_: &Theme| container::Style{
                background: Some(Background::Color(Color::from_rgba(0.0,0.0,0.0,0.35))),
                ..Default::default()
            });
        let center = container(popup)
            .width(Length::Fill).height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);
        iced::widget::opaque(
            iced::widget::Stack::with_children(vec![scrim.into(),center.into()])
                .width(Length::Fill).height(Length::Fill)
        ).into()
    }

    fn gbst_confirm_cleanup_popup(&self, log_cleanup: bool) -> Element<'_, Message> {
        let title = if log_cleanup {self.catalog("logs.cleanup")}
                    else {self.catalog("cache.title")};
        let warning = if log_cleanup {self.catalog("logs.cleanup.warning")}
                      else {self.catalog("cache.warning")};
        let affirmative = if log_cleanup {self.catalog("logs.cleanup.confirm")}
                          else {self.catalog("cache.clean")};
        let cancel = if log_cleanup {
            Message::CancelSavedLogCleanup
        } else { Message::CancelCacheCleanup };
        let proceed = if log_cleanup {
            Message::ConfirmSavedLogCleanup
        } else { Message::ConfirmCacheCleanup };
        let content=column![
            text(title).size(19).font(lpm_bold_font()),
            shell_muted(warning,13)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
            row![
                Space::new().width(Length::Fill),
                shell_action(self.catalog("common.cancel"),cancel,true,false),
                shell_action(affirmative,proceed,true,true),
            ].spacing(12).align_y(iced::Alignment::Center),
        ].spacing(18);
        let dialog=container(content).padding(24).width(Length::Fixed(520.0))
            .style(shell_panel_style);
        let scrim=container(Space::new()).width(Length::Fill).height(Length::Fill)
            .style(|_:&Theme|container::Style{
                background:Some(Background::Color(Color::from_rgba(0.0,0.0,0.0,0.35))),
                ..Default::default()
            });
        let center=container(dialog).width(Length::Fill).height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);
        iced::widget::opaque(
            iced::widget::Stack::with_children(vec![scrim.into(),center.into()])
                .width(Length::Fill).height(Length::Fill)).into()
    }
}

// GBST v2.0.0 navigation icon recoloring, preserving alpha and geometry.
fn light_nav_png_handle(
    bytes: &'static [u8],
    width: u32,
    height: u32,
) -> iced::widget::image::Handle {
    let Ok(decoded) = image_crate::load_from_memory(bytes) else {
        return iced::widget::image::Handle::from_bytes(bytes.to_vec());
    };
    let rgba = decoded.to_rgba8();
    let mut resized = image_crate::imageops::resize(
        &rgba,
        width,
        height,
        image_crate::imageops::FilterType::Lanczos3,
    );
    for pixel in resized.pixels_mut() {
        // Preserve transparent background and anti-aliased alpha edges.
        if pixel[3] > 0 {
            pixel[0] = 222;
            pixel[1] = 222;
            pixel[2] = 235;
        }
    }
    iced::widget::image::Handle::from_rgba(width, height, resized.into_raw())
}

#[cfg(test)]
mod gbst_gui_port_tests {
    use super::*;

    #[test]
    fn gbts_original_light_and_dark_palette_values_are_preserved() {
        let light = shell_colors(&Theme::Light);
        let dark = shell_colors(&Theme::Dark);
        assert_eq!(light.page, Color::from_rgb8(251,248,253));
        assert_eq!(light.panel, Color::from_rgb8(239,236,241));
        assert_eq!(dark.page, Color::from_rgb8(19,19,24));
        assert_eq!(dark.sidebar, Color::from_rgb8(27,27,33));
        assert_ne!(light.muted, dark.muted);
    }
    #[test]
    fn gbts_sidebar_responsive_rule_is_exact() {
        assert_eq!(sidebar_width_for_window(820.0,0.0),80.0);
        assert_eq!(sidebar_width_for_window(820.0,1.0),232.0);
        assert!(is_compact_window(1079.0));
        assert!(!is_compact_window(1080.0));
        assert_eq!(sidebar_width_for_window(1080.0,0.0),232.0);
    }
    #[test]
    fn all_navigation_entries_fit_normal_720px_window() {
        let items = 8.0 * 56.0;
        let group = 48.0;
        let padding = 24.0;
        assert!(items + group + padding < WINDOW_HEIGHT - 32.0);
    }
    #[test]
    fn localizations_have_rom_specific_labels_not_google_installers() {
        for locale in ["en","ko","ja","zh-TW","ru","vi","es","el","hi","ka","nl","ar"] {
            for key in [
                "lpm.nav.rom", "lpm.nav.driver", "lpm.rom.phase.5",
                "lpm.settings.downloadfolder", "lpm.about.third_party_notice"
            ] {
                let result=lpmbox_core::language_catalog::translate(locale,key);
                assert_ne!(result,key,"{locale} missing {key}");
                assert!(!result.trim().is_empty(),"{locale} empty {key}");
            }
        }
    }
}

// Some LPMBox nav_log.png pixels are opaque WHITE within the document glyph.
// GBST's normal dark recolor would turn those background pixels into a solid
// white rectangle. Preserve the outline and make white interior pixels transparent.
fn dark_nav_png_handle_with_white_knockout(
    bytes: &'static [u8], width: u32, height: u32
) -> iced::widget::image::Handle {
    let Ok(decoded) = image_crate::load_from_memory(bytes) else {
        return iced::widget::image::Handle::from_bytes(bytes.to_vec());
    };
    let mut resized = image_crate::imageops::resize(
        &decoded.to_rgba8(),width,height,
        image_crate::imageops::FilterType::Lanczos3,
    );
    for pixel in resized.pixels_mut() {
        let luminosity = (u16::from(pixel[0])+u16::from(pixel[1])+u16::from(pixel[2]))/3;
        if luminosity >= 240 {
            pixel[3] = 0;
        } else if luminosity > 175 {
            let opacity_scale = f32::from(240-luminosity)/65.0;
            pixel[3] = (f32::from(pixel[3])*opacity_scale) as u8;
        }
        pixel[0]=222; pixel[1]=222; pixel[2]=235;
    }
    iced::widget::image::Handle::from_rgba(width,height,resized.into_raw())
}
