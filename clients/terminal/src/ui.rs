use crate::app::{App, AppScreen};
use pong_core::{Config, RenderHelper, Side, Status, View};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, List, ListItem, Paragraph, Widget},
};

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create layout with title and content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5)])
            .split(area);

        // Render title
        let title = Paragraph::new("🏓 Pong Terminal Client 🏓")
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title("Pong")
                    .title_alignment(Alignment::Center),
            );
        title.render(chunks[0], buf);

        // Render screen-specific content
        match self.screen {
            AppScreen::Start => self.render_start_screen(chunks[1], buf),
            AppScreen::Host => self.render_host_screen(chunks[1], buf),
            AppScreen::Join => self.render_join_screen(chunks[1], buf),
            AppScreen::Local => self.render_local_screen(chunks[1], buf),
            AppScreen::Game => self.render_game_screen(chunks[1], buf),
        }
    }
}

impl App {
    fn render_start_screen(&self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .get_start_menu_items()
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.menu_state.start_selected {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(*item).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title("Main Menu")
                    .title_alignment(Alignment::Center),
            )
            .highlight_symbol("► ");

        list.render(area, buf);
    }

    fn render_host_screen(&self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Offer SDP display
                Constraint::Length(6), // Answer SDP input
                Constraint::Length(3), // Connection status
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        // Display generated Offer SDP
        let offer_widget = Paragraph::new(if self.menu_state.host_state.offer_sdp.is_empty() {
            "Generating offer...".to_string()
        } else {
            format!(
                "Offer SDP (copy this):\n\n{}",
                self.menu_state
                    .host_state
                    .offer_sdp
                    .chars()
                    .take(200)
                    .collect::<String>()
                    + "..."
            )
        })
        .style(Style::default().fg(Color::Green))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("Your Offer SDP")
                .title_alignment(Alignment::Center),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });
        offer_widget.render(chunks[0], buf);

        // Answer SDP input area
        let answer_content = if self.menu_state.host_state.answer_input.is_empty() {
            "Answer SDP from peer:\n\nPaste the Answer SDP here...".to_string()
        } else {
            let truncated = self
                .menu_state
                .host_state
                .answer_input
                .chars()
                .take(100)
                .collect::<String>();
            format!("Answer SDP from peer:\n\n{}", truncated)
        };

        let answer_widget = Paragraph::new(answer_content)
            .style(Style::default().fg(Color::Cyan))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title("Peer's Answer SDP")
                    .title_alignment(Alignment::Center),
            );
        answer_widget.render(chunks[1], buf);

        // Connection status
        let status_widget = Paragraph::new(self.menu_state.host_state.connection_status.clone())
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title("Status")
                    .title_alignment(Alignment::Center),
            );
        status_widget.render(chunks[2], buf);

        // Instructions
        let instructions = "ESC: Back to menu   ENTER: Connect (when Answer SDP is provided)";
        let instructions_widget = Paragraph::new(instructions)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        instructions_widget.render(chunks[3], buf);
    }

    fn render_join_screen(&self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Offer SDP input
                Constraint::Length(8), // Answer SDP display
                Constraint::Length(3), // Connection status
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        // Offer SDP input area
        let offer_content = if self.menu_state.join_state.offer_input.is_empty() {
            "Offer SDP from host:\n\nPaste the Offer SDP here...".to_string()
        } else {
            let truncated = self
                .menu_state
                .join_state
                .offer_input
                .chars()
                .take(100)
                .collect::<String>();
            format!("Offer SDP from host:\n\n{}", truncated)
        };

        let offer_widget = Paragraph::new(offer_content)
            .style(Style::default().fg(Color::Cyan))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title("Host's Offer SDP")
                    .title_alignment(Alignment::Center),
            );
        offer_widget.render(chunks[0], buf);

        // Display generated Answer SDP
        let answer_widget = Paragraph::new(if self.menu_state.join_state.answer_sdp.is_empty() {
            "Answer SDP will appear here after processing Offer...".to_string()
        } else {
            format!(
                "Answer SDP (copy this):\n\n{}",
                self.menu_state
                    .join_state
                    .answer_sdp
                    .chars()
                    .take(200)
                    .collect::<String>()
                    + "..."
            )
        })
        .style(Style::default().fg(Color::Green))
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("Your Answer SDP")
                .title_alignment(Alignment::Center),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });
        answer_widget.render(chunks[1], buf);

        // Connection status
        let status_widget = Paragraph::new(self.menu_state.join_state.connection_status.clone())
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title("Status")
                    .title_alignment(Alignment::Center),
            );
        status_widget.render(chunks[2], buf);

        // Instructions
        let instructions = "ESC: Back to menu   ENTER: Process Offer SDP";
        let instructions_widget = Paragraph::new(instructions)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        instructions_widget.render(chunks[3], buf);
    }

    fn render_local_screen(&self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .get_local_menu_items()
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.menu_state.local_selected {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(*item).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title("Local Game")
                    .title_alignment(Alignment::Center),
            )
            .highlight_symbol("► ");

        list.render(area, buf);
    }

    fn render_game_screen(&self, area: Rect, buf: &mut Buffer) {
        // Check if terminal is too small
        if self.ui_paused {
            let message = format!(
                "Terminal too small!\n\nMinimum required: {}×{}\nCurrent size: {}×{}\n\nPlease resize your terminal to continue playing.",
                60, // MIN_GAME_WIDTH
                20, // MIN_GAME_HEIGHT
                self.terminal_size.0,
                self.terminal_size.1
            );

            let resize_widget = Paragraph::new(message)
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .title("⚠ Resize Required ⚠")
                        .title_alignment(Alignment::Center),
                );
            resize_widget.render(area, buf);
            return;
        }

        if let Some(ref game) = self.game {
            let view = game.view();

            // Use fixed-size centered game area
            if let Some(game_area) = self.calculate_centered_game_area(area) {
                // Create layout for score + field + controls within centered area
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), // Score
                        Constraint::Min(10),   // Field
                        Constraint::Length(3), // Controls
                    ])
                    .split(game_area);

                // Render score and status
                let status_text = match view.status {
                    Status::Lobby => "Waiting for players to be ready (SPACE)",
                    Status::Countdown(_) => "Get ready...",
                    Status::Playing => "Playing",
                    Status::Scored(_, _) => "Point scored!",
                    Status::GameOver(_) => "Game Over!",
                };

                let score_content = format!(
                    "Score: {} - {}    Status: {}    Tick: {}",
                    view.score[0], view.score[1], status_text, view.tick
                );

                let score_widget = Paragraph::new(score_content)
                    .style(Style::default().fg(Color::Yellow))
                    .alignment(Alignment::Center)
                    .block(
                        Block::bordered()
                            .border_type(BorderType::Rounded)
                            .title("Game Info")
                            .title_alignment(Alignment::Center),
                    );
                score_widget.render(chunks[0], buf);

                // Render fixed-size game field with perfect paddle consistency
                self.render_game_field_with_helper(chunks[1], buf, &view);

                // Render controls
                let controls_text =
                    "P1: W/S (up/down)  P2: ↑/↓ (up/down)  SPACE: Ready  ESC: Menu  Q: Quit";
                let controls_widget = Paragraph::new(controls_text)
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center)
                    .block(
                        Block::bordered()
                            .border_type(BorderType::Rounded)
                            .title("Controls")
                            .title_alignment(Alignment::Center),
                    );
                controls_widget.render(chunks[2], buf);
            }
        } else {
            let content = Paragraph::new("No game running\n\nPress 'ESC' to return to menu")
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center)
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .title("Game")
                        .title_alignment(Alignment::Center),
                );
            content.render(area, buf);
        }
    }

    /// New rendering method with perfect paddle height consistency
    fn render_game_field_with_helper(&self, area: Rect, buf: &mut Buffer, view: &View) {
        // Terminal characters are ~2:1 (width:height), so compensate for aspect ratio
        const CHAR_ASPECT_RATIO: f32 = 0.5; // height/width ratio of terminal chars

        let available_width = area.width.saturating_sub(2);
        let available_height = area.height.saturating_sub(2);

        if available_width == 0 || available_height == 0 {
            return; // Too small to render
        }

        // Calculate field dimensions that appear square to users
        let field_width = available_width as usize;
        let ideal_height = (available_width as f32 * CHAR_ASPECT_RATIO) as usize;
        let field_height = ideal_height.min(available_height as usize);

        // Create config for RenderHelper (reconstruct from view data)
        let config = Config {
            paddle_half_h: view.paddle_half_h,
            paddle_width: view.paddle_width,
            ball_radius: view.ball_radius,
            paddle_x: view.paddle_x_offset,
            ..Config::default()
        };

        // Create RenderHelper for perfect paddle consistency
        let render_helper = RenderHelper::new(field_width, field_height, &config);

        // Create field content using RenderHelper - guarantees perfect consistency!
        let mut field_lines = Vec::new();

        for y in 0..field_height {
            let mut line = vec![' '; field_width];

            // Ball - using RenderHelper
            let (ball_x, ball_y) = render_helper.get_ball_position(view.ball_pos);
            if ball_x < field_width && y == ball_y {
                line[ball_x] = '●';
            }

            // Left paddle - using RenderHelper for perfect consistency
            let left_paddle_rect = render_helper.get_paddle_rect(view.left_paddle_y, Side::Left);
            if y >= left_paddle_rect.top && y <= left_paddle_rect.bottom {
                for x in left_paddle_rect.left..=left_paddle_rect.right {
                    if x < field_width {
                        line[x] = '█';
                    }
                }
            }

            // Right paddle - using RenderHelper for perfect consistency
            let right_paddle_rect = render_helper.get_paddle_rect(view.right_paddle_y, Side::Right);
            if y >= right_paddle_rect.top && y <= right_paddle_rect.bottom {
                for x in right_paddle_rect.left..=right_paddle_rect.right {
                    if x < field_width {
                        line[x] = '█';
                    }
                }
            }

            // Center line
            let center_x = field_width / 2;
            if center_x < field_width && line[center_x] == ' ' {
                line[center_x] = '┊';
            }

            field_lines.push(Line::from(line.iter().collect::<String>()));
        }

        let field_widget = Paragraph::new(field_lines)
            .style(Style::default().fg(Color::White))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title("Field")
                    .title_alignment(Alignment::Center),
            );
        field_widget.render(area, buf);
    }

    /// Legacy rendering method (kept for reference/fallback)
    fn render_game_field(&self, area: Rect, buf: &mut Buffer) {
        // This method is now deprecated - use render_game_field_with_helper instead
        // Keeping for compatibility during transition
        if let Some(ref game) = self.game {
            let view = game.view();
            self.render_game_field_with_helper(area, buf, &view);
        }
    }
}
