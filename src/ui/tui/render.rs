// UI rendering functions for the TUI

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

// Use our defined PastelColor enum and Theme
use crate::ui::colors::{PastelColor, Theme};
use crate::core::system::DistroFamily; // Re-add the import
use super::state::AppState;

/// Convert our custom PastelColor to ratatui Color
fn pastel_to_ratatui_color(color: PastelColor) -> Color {
    match color {
        PastelColor::Pink => Color::Rgb(255, 182, 193),       // Light pink
        PastelColor::Lavender => Color::Rgb(204, 169, 221),   // Light purple
        PastelColor::Mint => Color::Rgb(176, 224, 183),       // Mint green
        PastelColor::SkyBlue => Color::Rgb(173, 216, 230),    // Light sky blue
        PastelColor::Peach => Color::Rgb(255, 218, 185),      // Peach
        PastelColor::LightYellow => Color::Rgb(255, 255, 224),// Light yellow
        PastelColor::White => Color::White,
        PastelColor::Gray => Color::Rgb(169, 169, 169),       // Light gray
    }
}

/// Main UI render function (Make explicitly public)
pub fn ui(f: &mut Frame, app: &AppState) {
    // Get theme (using default for now, could be part of AppState later)
    let theme = Theme::default();

    // Create main layout
    let size = f.size();

    // Create main UI layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Title bar
            Constraint::Min(10),     // Main content
            Constraint::Length(8),   // Console log
            Constraint::Length(3),   // Footer
        ])
        .split(size);

    // Create and render title bar
    render_title(f, app, chunks[0], &theme);

    // Render different content based on state
    if app.show_gpu_details && app.gpus.as_ref().map_or(0, |g| g.len()) > 0 {
        render_gpu_details(f, app, chunks[1], &theme);
    } else {
        render_dashboard(f, app, chunks[1], &theme);
    }

    // Render console log
    render_console(f, app, chunks[2], &theme);

    // Create and render footer
    render_footer(f, app, chunks[3], &theme);

    // Render loading overlay if needed
    if let Some(message) = app.loading_message.as_ref().or(app.current_action.as_ref()) {
        render_loading_overlay(f, message, &theme);
    }
} // Closing brace for ui function

/// Render a loading overlay
fn render_loading_overlay(f: &mut Frame, message: &str, theme: &Theme) {
    let area = f.size();
    let overlay_height = 3;
    let overlay_width = message.len() as u16 + 10;

    // Calculate overlay position
    let x = area.width.saturating_sub(overlay_width) / 2;
    let y = area.height.saturating_sub(overlay_height) / 2;

    // Create overlay area
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    // Create overlay block
    let overlay = Paragraph::new(Line::from(vec![
        Span::styled(
            message,
            Style::default()
                .fg(pastel_to_ratatui_color(theme.primary)) // Lavender
                .add_modifier(Modifier::BOLD),
        )
    ]))
    .block(Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(pastel_to_ratatui_color(theme.accent)))) // Pink
    .alignment(Alignment::Center);

    f.render_widget(overlay, overlay_area);
}

/// Render the title bar
fn render_title(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    let title = format!(" ✨ {} ✨ ", app.title);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            title,
            Style::default()
                .fg(pastel_to_ratatui_color(theme.primary)) // Lavender
                .add_modifier(Modifier::BOLD)
        ))
        .border_style(Style::default().fg(pastel_to_ratatui_color(theme.primary))); // Lavender

    f.render_widget(title_block, area);

    // Show subtitle
    let subtitle = "GPU Passthrough Automation Command Center"; // Updated subtitle
    let subtitle_len = subtitle.len() as u16;
    let center_x = area.x + (area.width.saturating_sub(subtitle_len)) / 2;

    // Create paragraph for subtitle
    let subtitle_area = Rect::new(
        center_x,
        area.y + 1,
        subtitle_len,
        1
    );

    let subtitle_text = Paragraph::new(Line::from(vec![
        Span::styled(
            subtitle,
            Style::default()
                .fg(pastel_to_ratatui_color(theme.accent)) // Pink
                .add_modifier(Modifier::BOLD),
        )
    ]));

    f.render_widget(subtitle_text, subtitle_area);
}

/// Render the main dashboard with system and GPU information
fn render_dashboard(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    // Split area into system info and GPU info
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // System info
            Constraint::Percentage(50), // GPU info
        ])
        .split(area);

    // Render system info
    render_system_info(f, app, chunks[0], theme);

    // Render GPU summary
    render_gpu_summary(f, app, chunks[1], theme);
}

/// Render system information panel
fn render_system_info(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(" System Information ", Style::default().fg(pastel_to_ratatui_color(theme.secondary)).add_modifier(Modifier::BOLD))) // SkyBlue Title
        .border_style(Style::default().fg(pastel_to_ratatui_color(theme.secondary))); // SkyBlue Border

    f.render_widget(block, area);

    // Create inner area for content
    let inner_area = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    // Show system info if available
    if let Some(system_info) = &app.system_info {
        // Convert system info to friendly display format
        let mut info_lines = Vec::new();
        let label_style = Style::default().fg(pastel_to_ratatui_color(theme.text)).add_modifier(Modifier::BOLD);
        let value_style = Style::default().fg(pastel_to_ratatui_color(theme.text));

        // Add kernel info with a pretty symbol
        info_lines.push(Line::from(vec![
            Span::styled("🖥️  Kernel: ", label_style),
            Span::styled(&system_info.kernel_version.full_version, value_style),
        ]));

        // Add bootloader info
        info_lines.push(Line::from(vec![
            Span::styled("⚙️  Bootloader: ", label_style),
            Span::styled(format!("{:?}", system_info.bootloader), value_style),
        ]));

        // Add CPU info
        info_lines.push(Line::from(vec![
            Span::styled("💻 CPU Vendor: ", label_style),
            Span::styled(format!("{:?}", system_info.cpu_vendor), value_style),
        ]));

        // Add virtualization status
        info_lines.push(Line::from(vec![
            Span::styled("🔄 Virtualization: ", label_style),
            Span::styled(
                if system_info.virtualization_enabled { "Enabled" } else { "Disabled" },
                if system_info.virtualization_enabled {
                    Style::default().fg(pastel_to_ratatui_color(theme.success)) // Mint (success)
                } else {
                    Style::default().fg(pastel_to_ratatui_color(theme.error)) // Pink (warning)
                }
            ),
        ]));

        // Add distribution info if available
        if let Some(ref distro) = system_info.distribution {
             let family_str = match &distro.family {
                 Some(family) => {
                     // Explicitly match on DistroFamily variants to mark the type as used
                     let family_name = match family {
                         DistroFamily::Arch => "Arch",
                         DistroFamily::Debian => "Debian",
                         DistroFamily::Fedora => "Fedora",
                         DistroFamily::Suse => "Suse",
                         DistroFamily::Gentoo => "Gentoo",
                         DistroFamily::Other(name) => name.as_str(), // Use the name from Other variant
                     };
                     format!(" ({}-based)", family_name) // Use the matched name
                 },
                 None => String::new(),
             };
             info_lines.push(Line::from(vec![
                 Span::styled("🐧 Distribution: ", label_style),
                 Span::styled(format!("{} {}{}", distro.name, distro.version, family_str), value_style), // Include family in output
             ]));
        }

        // Add init system
        info_lines.push(Line::from(vec![
            Span::styled("🔧 Init System: ", label_style),
            Span::styled(format!("{:?}", system_info.init_system), value_style),
        ]));

        // Add initramfs system
        info_lines.push(Line::from(vec![
            Span::styled("📦 Initramfs: ", label_style),
            Span::styled(format!("{:?}", system_info.initramfs_system), value_style),
        ]));

        // Add secure boot status
        if let Some(secure_boot) = system_info.secure_boot_enabled {
            info_lines.push(Line::from(vec![
                Span::styled("🔒 Secure Boot: ", label_style),
                Span::styled(
                    if secure_boot { "Enabled" } else { "Disabled" },
                    if secure_boot {
                        Style::default().fg(pastel_to_ratatui_color(theme.error)) // Pink (warning)
                    } else {
                        Style::default().fg(pastel_to_ratatui_color(theme.success)) // Mint (ok)
                    }
                ),
            ]));
        }

        // Create paragraph
        let paragraph = Paragraph::new(info_lines)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, inner_area);
    } else {
        // Show loading message
        let text = vec![
            Line::from(vec![
                Span::styled(
                    "System information not available",
                    Style::default().fg(pastel_to_ratatui_color(theme.primary)), // Lavender
                )
            ]),
            Line::from(""),
            Line::from("Press 'r' to detect system information."),
        ];

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, inner_area);
    }
}

/// Render GPU summary panel
fn render_gpu_summary(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    // Determine title style based on GPU detection status
    let title_style = Style::default()
        .fg(pastel_to_ratatui_color(theme.accent)) // Pink
        .add_modifier(Modifier::BOLD);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(" GPU Information ", title_style))
        .border_style(Style::default().fg(pastel_to_ratatui_color(theme.accent))); // Pink

    f.render_widget(block, area);

    // Create inner area for content
    let inner_area = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    // Show GPU info if available
    if let Some(gpus) = &app.gpus {
        if gpus.is_empty() {
            // No GPUs found
            let text = vec![
                Line::from(vec![
                    Span::styled(
                        "No GPUs detected",
                        Style::default().fg(pastel_to_ratatui_color(theme.error)), // Peach -> Changed to Error Pink
                    )
                ]),
                Line::from(""),
                Line::from("This may indicate a detection issue or that there are no GPUs in the system."),
            ];

            let paragraph = Paragraph::new(text)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, inner_area);
        } else {
            // Show a summary of each GPU with stylish formatting
            let mut info_lines = vec![
                Line::from(vec![
                    Span::styled(
                        format!("🔍 Found {} GPU(s)", gpus.len()),
                        Style::default()
                            .fg(pastel_to_ratatui_color(theme.primary)) // Lavender
                            .add_modifier(Modifier::BOLD),
                    )
                ]),
                Line::from(vec![
                    Span::styled("Press 'g' to view detailed GPU information", Style::default().fg(pastel_to_ratatui_color(theme.text)))
                ]),
                Line::from(""),
            ];

            // Define styles used in this function
            let label_style = Style::default().fg(pastel_to_ratatui_color(theme.text)).add_modifier(Modifier::BOLD);
            let value_style = Style::default().fg(pastel_to_ratatui_color(theme.text));

            // Add a summary for each GPU
            for (i, gpu) in gpus.iter().enumerate() {
                // Highlight selected GPU for passthrough
                let header_style = if app.selected_passthrough_gpu_index == Some(i) {
                    Style::default().add_modifier(Modifier::BOLD).fg(pastel_to_ratatui_color(theme.success)) // Mint for selected
                } else {
                    Style::default().add_modifier(Modifier::BOLD).fg(pastel_to_ratatui_color(theme.accent)) // Pink otherwise
                };
                let text_style = if app.selected_passthrough_gpu_index == Some(i) {
                    Style::default().fg(pastel_to_ratatui_color(theme.success))
                } else {
                    Style::default().fg(pastel_to_ratatui_color(theme.text))
                };

                // Create a stylized header for each GPU
                info_lines.push(Line::from(vec![
                    Span::styled(format!("GPU {}: ", i+1), header_style),
                    Span::styled(gpu.model_name(), text_style.add_modifier(Modifier::BOLD)),
                    Span::styled(" (", text_style),
                    Span::styled(format!("{}", gpu.vendor()), text_style),
                    Span::styled(")", text_style),
                    if app.selected_passthrough_gpu_index == Some(i) {
                        Span::styled(" [Selected for Passthrough]", Style::default().fg(pastel_to_ratatui_color(theme.success)).add_modifier(Modifier::ITALIC))
                    } else {
                        Span::raw("")
                    }
                ]));

                // Add basic information about the GPU
                info_lines.push(Line::from(vec![
                    Span::styled("  • BDF: ", label_style),
                    Span::styled(gpu.bdf(), value_style),
                ]));

                // Show driver information
                info_lines.push(Line::from(vec![
                    Span::styled("  • Driver: ", label_style),
                    Span::styled(gpu.driver.as_deref().unwrap_or("None"), value_style),
                ]));

                // Show key capabilities with colored status indicators
                let reset_style = if gpu.capabilities.supports_reset {
                    Style::default().fg(pastel_to_ratatui_color(theme.success)) // Mint (good)
                } else {
                    Style::default().fg(pastel_to_ratatui_color(theme.error)) // Pink (bad)
                };

                let reset_bug_style = if gpu.capabilities.has_reset_bug {
                    Style::default().fg(pastel_to_ratatui_color(theme.error)) // Pink (bad)
                } else {
                    Style::default().fg(pastel_to_ratatui_color(theme.success)) // Mint (good)
                };

                info_lines.push(Line::from(vec![
                    Span::styled("  • Reset: ", label_style),
                    Span::styled(
                        if gpu.capabilities.supports_reset { "✓" } else { "✗" },
                        reset_style,
                    ),
                    Span::styled("  Reset Bug: ", label_style),
                    Span::styled(
                        if gpu.capabilities.has_reset_bug { "✓" } else { "✗" },
                        reset_bug_style,
                    ),
                ]));

                // Add separator between GPUs
                if i < gpus.len() - 1 {
                    info_lines.push(Line::from(""));
                }
            }

            // Create scrollable paragraph
            let paragraph = Paragraph::new(info_lines)
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, inner_area);
        }
    } else {
        // Show info not available message
        let text = vec![
            Line::from(vec![
                Span::styled(
                    "GPU information not available",
                    Style::default().fg(pastel_to_ratatui_color(theme.primary)), // Lavender
                )
            ]),
            Line::from(""),
            Line::from("Press 'r' to detect GPUs."),
        ];

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, inner_area);
    }
}

/// Render detailed information for a specific GPU
fn render_gpu_details(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    if let Some(gpus) = &app.gpus {
        if app.selected_gpu_index < gpus.len() {
            let gpu = &gpus[app.selected_gpu_index];

            // Create header with navigation help
            let header_text = format!(" GPU {} of {} - {} ",
                app.selected_gpu_index + 1,
                gpus.len(),
                gpu.model_name()
            );
            let selection_indicator = if Some(app.selected_gpu_index) == app.selected_passthrough_gpu_index {
                " [Selected]"
            } else {
                ""
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    format!("{}{}", header_text, selection_indicator), // Add selection indicator
                    Style::default()
                        .fg(pastel_to_ratatui_color(theme.accent)) // Pink
                        .add_modifier(Modifier::BOLD)
                ))
                .border_style(Style::default().fg(pastel_to_ratatui_color(theme.accent))); // Pink

            f.render_widget(block, area);

            // Create inner area for content
            let inner_area = Rect {
                x: area.x + 2,
                y: area.y + 1,
                width: area.width.saturating_sub(4),
                height: area.height.saturating_sub(2),
            };

            // Build detailed information
            let mut info_lines = Vec::new();
            let label_style = Style::default().fg(pastel_to_ratatui_color(theme.text)).add_modifier(Modifier::BOLD);
            let value_style = Style::default().fg(pastel_to_ratatui_color(theme.text));

            // Basic information
            info_lines.push(Line::from(vec![
                Span::styled("Model: ", label_style),
                Span::styled(gpu.model_name(), value_style),
            ]));

            info_lines.push(Line::from(vec![
                Span::styled("Vendor: ", label_style),
                Span::styled(format!("{}", gpu.vendor()), value_style),
            ]));

            info_lines.push(Line::from(vec![
                Span::styled("BDF Address: ", label_style),
                Span::styled(gpu.bdf(), value_style),
            ]));

            info_lines.push(Line::from(vec![
                Span::styled("Vendor ID: ", label_style),
                Span::styled(&gpu.vendor_id, value_style),
            ]));

            info_lines.push(Line::from(vec![
                Span::styled("Device ID: ", label_style),
                Span::styled(&gpu.device_id, value_style),
            ]));

            info_lines.push(Line::from(vec![
                Span::styled("Driver: ", label_style),
                Span::styled(gpu.driver.as_deref().unwrap_or("None"), value_style),
            ]));

            info_lines.push(Line::from(vec![
                Span::styled("Integrated: ", label_style),
                Span::styled(if gpu.is_integrated { "Yes" } else { "No" }, value_style),
            ]));

            info_lines.push(Line::from(""));

            // GPU capabilities with detailed explanations
            info_lines.push(Line::from(vec![
                Span::styled("Capabilities:",
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(pastel_to_ratatui_color(theme.primary)) // Lavender
                )
            ]));

            // Reset support
            info_lines.push(Line::from(vec![
                Span::styled("  Reset Support: ", label_style),
                Span::styled(
                    if gpu.capabilities.supports_reset { "Yes" } else { "No" },
                    if gpu.capabilities.supports_reset {
                        Style::default().fg(pastel_to_ratatui_color(theme.success)) // Mint (good)
                    } else {
                        Style::default().fg(pastel_to_ratatui_color(theme.error)) // Pink (bad)
                    }
                ),
            ]));

            info_lines.push(Line::from(vec![
                Span::styled("    Can the GPU be reset without rebooting the host system", Style::default().fg(pastel_to_ratatui_color(PastelColor::Gray)))
            ]));

            // Reset bug
            info_lines.push(Line::from(vec![
                Span::styled("  Reset Bug: ", label_style),
                Span::styled(
                    if gpu.capabilities.has_reset_bug { "Yes (affected)" } else { "No" },
                    if gpu.capabilities.has_reset_bug {
                        Style::default().fg(pastel_to_ratatui_color(theme.error)) // Pink (bad)
                    } else {
                        Style::default().fg(pastel_to_ratatui_color(theme.success)) // Mint (good)
                    }
                ),
            ]));

            info_lines.push(Line::from(vec![
                Span::styled("    AMD GPUs that hang after VM shutdown due to firmware issues", Style::default().fg(pastel_to_ratatui_color(PastelColor::Gray)))
            ]));

            // Code 43
            info_lines.push(Line::from(vec![
                Span::styled("  Code 43 Workaround: ", label_style),
                Span::styled(
                    if gpu.capabilities.needs_code_43_workaround { "Required" } else { "Not needed" },
                    if gpu.capabilities.needs_code_43_workaround {
                        Style::default().fg(pastel_to_ratatui_color(theme.error)) // Peach (warning) -> Changed to Error Pink
                    } else {
                        Style::default().fg(pastel_to_ratatui_color(theme.success)) // Mint (good)
                    }
                ),
            ]));

            info_lines.push(Line::from(vec![
                Span::styled("    NVIDIA driver detection countermeasures needed for Windows guests", Style::default().fg(pastel_to_ratatui_color(PastelColor::Gray)))
            ]));

            // GVT-g support
            info_lines.push(Line::from(vec![
                Span::styled("  GVT-g Support: ", label_style),
                Span::styled(if gpu.capabilities.supports_gvt { "Yes" } else { "No" }, value_style),
            ]));

            info_lines.push(Line::from(vec![
                Span::styled("    Intel GPU virtualization technology for sharing the GPU", Style::default().fg(pastel_to_ratatui_color(PastelColor::Gray)))
            ]));

            // VBIOS loading
            info_lines.push(Line::from(vec![
                Span::styled("  VBIOS Loading: ", label_style),
                Span::styled(if gpu.capabilities.supports_vbios_loading { "Supported" } else { "Not Supported" }, value_style),
            ]));

            info_lines.push(Line::from(vec![
                Span::styled("    Custom video BIOS can be loaded for better compatibility", Style::default().fg(pastel_to_ratatui_color(PastelColor::Gray)))
            ]));

            // Add action prompt
            info_lines.push(Line::from(""));
            if Some(app.selected_gpu_index) == app.selected_passthrough_gpu_index {
                 info_lines.push(Line::from(vec![
                     Span::styled("✓ This GPU is selected for passthrough.", Style::default().fg(pastel_to_ratatui_color(theme.success)))
                 ]));
            } else {
                 info_lines.push(Line::from(vec![
                     Span::styled("Press 's' to select this GPU for passthrough", Style::default().fg(pastel_to_ratatui_color(theme.accent)).add_modifier(Modifier::BOLD))
                 ]));
            }


            // Create paragraph with the detailed information
            let paragraph = Paragraph::new(info_lines)
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, inner_area);
        }
    }
}

/// Render console log panel
fn render_console(f: &mut Frame, app: &AppState, area: Rect, _theme: &Theme) { // Mark theme as unused
    // Draw console block
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Console ")
        .border_style(Style::default().fg(pastel_to_ratatui_color(PastelColor::Gray))); // Gray

    f.render_widget(block, area);

    // Create inner area for log messages
    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    // Create list of log messages
    let log_items: Vec<ListItem> = app.log_messages
        .iter()
        .map(|msg| {
            let color = msg.level.color();
            let time_style = Style::default().fg(pastel_to_ratatui_color(PastelColor::Gray));

            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", msg.timestamp), time_style),
                Span::styled(&msg.text, Style::default().fg(color)),
            ]))
        })
        .collect();

    // Create the list widget
    let log_view = List::new(log_items);

    f.render_widget(log_view, inner_area);
}

/// Render the footer
fn render_footer(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    let mut help_text = vec![
        Span::styled("r", Style::default().fg(pastel_to_ratatui_color(theme.accent)).add_modifier(Modifier::BOLD)),
        Span::styled("efresh | ", Style::default().fg(pastel_to_ratatui_color(theme.text))),
    ];

    // Show different help text based on current view
    if app.show_gpu_details {
        help_text.extend(vec![
            Span::styled("↑/↓", Style::default().fg(pastel_to_ratatui_color(theme.accent)).add_modifier(Modifier::BOLD)),
            Span::styled(" change GPU | ", Style::default().fg(pastel_to_ratatui_color(theme.text))),
            Span::styled("s", Style::default().fg(pastel_to_ratatui_color(theme.accent)).add_modifier(Modifier::BOLD)), // Select key
            Span::styled("elect GPU | ", Style::default().fg(pastel_to_ratatui_color(theme.text))),
            Span::styled("Esc", Style::default().fg(pastel_to_ratatui_color(theme.accent)).add_modifier(Modifier::BOLD)),
            Span::styled(" back | ", Style::default().fg(pastel_to_ratatui_color(theme.text))),
        ]);
    } else {
        if app.gpus.as_ref().map_or(0, |g| g.len()) > 0 {
            help_text.extend(vec![
                Span::styled("g", Style::default().fg(pastel_to_ratatui_color(theme.accent)).add_modifier(Modifier::BOLD)),
                Span::styled(" GPU details | ", Style::default().fg(pastel_to_ratatui_color(theme.text))),
            ]);
        }
        if app.selected_passthrough_gpu_index.is_some() {
             help_text.extend(vec![
                Span::styled("c", Style::default().fg(pastel_to_ratatui_color(theme.accent)).add_modifier(Modifier::BOLD)),
                Span::styled("onfigure | ", Style::default().fg(pastel_to_ratatui_color(theme.text))),
            ]);
        }
    }

    help_text.extend(vec![
        Span::styled("q", Style::default().fg(pastel_to_ratatui_color(theme.accent)).add_modifier(Modifier::BOLD)),
        Span::styled("uit", Style::default().fg(pastel_to_ratatui_color(theme.text))),
    ]);

    let text = vec![Line::from(help_text)];

    let paragraph = Paragraph::new(text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(pastel_to_ratatui_color(PastelColor::Gray)))) // Gray
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}