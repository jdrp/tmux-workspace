use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::collections::HashSet;

use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::tmux::tmux_session_exists;
use crate::workspace::Workspace;
use crate::storage::{delete_workspace_file, list_workspaces};

pub enum TuiAction {
    Start(String),
    Edit(String),
    Quit,
}

struct App {
    workspaces: Vec<Workspace>,
    filtered_indices: Vec<usize>,
    selected: usize,
    list_state: ListState,
    search: String,
    search_mode: bool,
    pending_delete: Option<String>,
    message: Option<String>,
    running_sessions: HashSet<String>,
}

impl App {
    fn new() -> Result<Self, String> {
        let workspaces = list_workspaces()?;
        let filtered_indices = (0..workspaces.len()).collect();
        let selected = 0;
        let mut list_state = ListState::default();

        if !workspaces.is_empty() {
            list_state.select(Some(selected));
        }

        let mut app = Self {
            workspaces,
            filtered_indices,
            selected,
            list_state,
            search: String::new(),
            search_mode: false,
            pending_delete: None,
            message: None,
            running_sessions: HashSet::new(),
        };

        app.refresh_running_sessions();
        Ok(app)
    }

    fn selected_workspace(&self) -> Option<&Workspace> {
        let workspace_index = *self.filtered_indices.get(self.selected)?;
        self.workspaces.get(workspace_index)
    }

    fn next(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }

        self.selected = (self.selected + 1) % self.filtered_indices.len();
        self.sync_list_state();
    }

    fn previous(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }

        self.selected =
            (self.selected + self.filtered_indices.len() - 1) % self.filtered_indices.len();
        self.sync_list_state();
    }

    fn sync_list_state(&mut self) {
        if self.filtered_indices.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(self.selected));
        }
    }

    fn refresh_running_sessions(&mut self) {
        self.running_sessions.clear();

        for workspace in &self.workspaces {
            match tmux_session_exists(&workspace.name) {
                Ok(true) => {
                    self.running_sessions.insert(workspace.name.clone());
                }
                Ok(false) => {}
                Err(_) => {}
            }
        }
    }

    fn is_running(&self, workspace: &Workspace) -> bool {
        self.running_sessions.contains(&workspace.name)
    }

    fn refresh(&mut self) -> Result<(), String> {
        self.workspaces = list_workspaces()?;
        self.refresh_running_sessions();
        self.apply_filter();

        Ok(())
    }

    fn apply_filter(&mut self) {
        let query = self.search.to_lowercase();

        self.filtered_indices = self
            .workspaces
            .iter()
            .enumerate()
            .filter_map(|(index, workspace)| {
                let matches = query.is_empty()
                    || workspace.name.to_lowercase().contains(&query)
                    || workspace.template.to_lowercase().contains(&query)
                    || workspace.root.to_lowercase().contains(&query);

                matches.then_some(index)
            })
            .collect();

        if self.filtered_indices.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.filtered_indices.len() {
            self.selected = self.filtered_indices.len() - 1;
        }

        self.sync_list_state();
    }

    fn request_delete_selected(&mut self) {
        if let Some(workspace) = self.selected_workspace() {
            self.pending_delete = Some(workspace.name.clone());
        }
    }

    fn confirm_delete(&mut self) -> Result<(), String> {
        let Some(name) = self.pending_delete.take() else {
            return Ok(());
        };

        let path = delete_workspace_file(&name)?;
        self.message = Some(format!("Deleted {}", path.display()));
        self.refresh()?;

        Ok(())
    }

    fn cancel_delete(&mut self) {
        self.pending_delete = None;
    }
}

pub fn run() -> Result<TuiAction, String> {
    let mut terminal =
        ratatui::try_init().map_err(|error| format!("failed to initialize TUI: {error}"))?;

    let mut app = App::new()?;
    let result = run_app(&mut terminal, &mut app);

    ratatui::restore();

    result
}

fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> Result<TuiAction, String> {
    loop {
        terminal
            .draw(|frame| render(frame, app))
            .map_err(|error| format!("failed to draw TUI: {error}"))?;

        let event = event::read().map_err(|error| format!("failed to read event: {error}"))?;

        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if app.pending_delete.is_some() {
                match key.code {
                    KeyCode::Char('y') => {
                        app.confirm_delete()?;
                    }
                    KeyCode::Char('n') | KeyCode::Esc => {
                        app.cancel_delete();
                    }
                    _ => {}
                }

                continue;
            }

            if app.search_mode {
                match key.code {
                    KeyCode::Esc => {
                        app.search_mode = false;
                        app.search.clear();
                        app.apply_filter();
                    }
                    KeyCode::Enter => {
                        app.search_mode = false;
                    }
                    KeyCode::Backspace => {
                        app.search.pop();
                        app.apply_filter();
                    }
                    KeyCode::Char(character) => {
                        app.search.push(character);
                        app.apply_filter();
                    }
                    _ => {}
                }

                continue;
            }

            match key.code {
                KeyCode::Char('q') => return Ok(TuiAction::Quit),
                KeyCode::Char('j') | KeyCode::Down => app.next(),
                KeyCode::Char('k') | KeyCode::Up => app.previous(),
                KeyCode::Char('r') => app.refresh()?,
                KeyCode::Char('d') => app.request_delete_selected(),
                KeyCode::Char('/') => app.search_mode = true,
                KeyCode::Esc => {
                    app.search_mode = false;
                    app.search.clear();
                    app.apply_filter();
                }
                KeyCode::Enter => {
                    if let Some(workspace) = app.selected_workspace() {
                        return Ok(TuiAction::Start(workspace.name.clone()));
                    }
                }
                KeyCode::Char('e') => {
                    if let Some(workspace) = app.selected_workspace() {
                        return Ok(TuiAction::Edit(workspace.name.clone()));
                    }
                }
                _ => {}
            }
        }
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width.saturating_sub(4));
    let height = height.min(area.height.saturating_sub(4));

    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;

    Rect {
        x,
        y,
        width,
        height,
    }
}

fn render_delete_popup(frame: &mut Frame, app: &App, area: Rect) {
    let Some(name) = &app.pending_delete else {
        return;
    };

    let popup_area = centered_rect(58, 9, area);

    let text = format!(
        "Delete workspace '{name}'?\n\nThis will remove its TOML file.\nThe tmux session will not be killed.\n\n y confirm   n/Esc cancel"
    );

    let popup = Paragraph::new(text)
        .block(
            Block::default()
                .title("Delete workspace")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(vertical_chunks[0]);

    render_workspace_list(frame, app, main_chunks[0]);
    render_workspace_details(frame, app, main_chunks[1]);
    render_footer(frame, app, vertical_chunks[1]);
    render_delete_popup(frame, app, area);
}

fn render_workspace_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = if app.workspaces.is_empty() {
        vec![ListItem::new("No workspaces found")]
    } else if app.filtered_indices.is_empty() {
        vec![ListItem::new("No matches")]
    } else {
        app.filtered_indices
            .iter()
            .filter_map(|&workspace_index| app.workspaces.get(workspace_index))
            .map(|workspace| {
                let status = if app.is_running(workspace) {
                    "●"
                } else {
                    "○"
                };
                ListItem::new(format!(
                    "{:<20} {:<10} {}",
                    workspace.name, workspace.template, status
                ))
            })
            .collect()
    };

    let list = List::new(items)
        .block(Block::default().title("Workspaces").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_workspace_details(frame: &mut Frame, app: &App, area: Rect) {
    let text = match app.selected_workspace() {
        Some(workspace) => {
            let running = app.is_running(workspace);
            workspace_details_text(workspace, running)
        }
        None => String::from(
            "No workspaces found.\n\nCreate one with:\n\n  tw init my-project --template rust --root .",
        ),
    };

    let details = Paragraph::new(text)
        .block(Block::default().title("Details").borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(details, area);
}

fn workspace_details_text(workspace: &Workspace, running: bool) -> String {
    let mut text = String::new();
    let session = if running { "running" } else { "stopped" };

    text.push_str(&format!("name: {}\n\n", workspace.name));
    text.push_str(&format!("template: {}\n\n", workspace.template));
    text.push_str(&format!("root: {}\n\n", workspace.root));
    text.push_str(&format!("session: {}\n\n", session));
    text.push_str("windows:\n");

    for window in &workspace.windows {
        match &window.command {
            Some(command) => {
                text.push_str(&format!("  {}: {}\n", window.name, command));
            }
            None => {
                text.push_str(&format!("  {}:\n", window.name));
            }
        }

        if let Some(layout) = window.layout {
            text.push_str(&format!("    layout: {}\n", layout.tmux_name()));
        }

        for pane in &window.panes {
            text.push_str(&format!("    pane: {}\n", pane.command));
        }
    }

    text
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(area);

    frame.render_widget(block, area);

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(inner);

    let search_text = if app.search_mode {
        format!(" Search: {}", app.search)
    } else if app.search.is_empty() {
        String::new()
    } else {
        format!(" Filter: {}", app.search)
    };

    let keybinds = if app.search_mode {
        "Esc clear  "
    } else if app.search.is_empty() {
        "↑/↓ j/k move   Enter start   e edit   d delete   r refresh   / search   q quit  "
    } else {
        "Esc clear   ↑/↓ j/k move   Enter start   e edit   d delete   r refresh   / search   q quit  "
    };

    let left_footer = Paragraph::new(Line::from(search_text));
    let right_footer = Paragraph::new(Line::from(keybinds)).right_aligned();

    frame.render_widget(left_footer, footer_chunks[0]);
    frame.render_widget(right_footer, footer_chunks[1]);
}
