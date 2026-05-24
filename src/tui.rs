use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::workspace::Workspace;
use crate::storage::list_workspaces;

struct App {
    workspaces: Vec<Workspace>,
    selected: usize,
    list_state: ListState,
}

impl App {
    fn new() -> Result<Self, String> {
        let workspaces = list_workspaces()?;
        let selected = 0;
        let mut list_state = ListState::default();

        if !workspaces.is_empty() {
            list_state.select(Some(selected));
        }

        Ok(Self {
            workspaces,
            selected,
            list_state,
        })
    }

    fn selected_workspace(&self) -> Option<&Workspace> {
        self.workspaces.get(self.selected)
    }

    fn next(&mut self) {
        if self.workspaces.is_empty() {
            return;
        }

        self.selected = (self.selected + 1) % self.workspaces.len();
        self.sync_list_state();
    }

    fn previous(&mut self) {
        if self.workspaces.is_empty() {
            return;
        }

        self.selected = (self.selected + self.workspaces.len() - 1) % self.workspaces.len();
        self.sync_list_state();
    }

    fn sync_list_state(&mut self) {
        if self.workspaces.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(self.selected));
        }
    }
}

pub fn run() -> Result<Option<String>, String> {
    let mut terminal =
        ratatui::try_init().map_err(|error| format!("failed to initialize TUI: {error}"))?;

    let mut app = App::new()?;
    let result = run_app(&mut terminal, &mut app);

    ratatui::restore();

    result
}

fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> Result<Option<String>, String> {
    loop {
        terminal
            .draw(|frame| render(frame, app))
            .map_err(|error| format!("failed to draw TUI: {error}"))?;

        let event = event::read().map_err(|error| format!("failed to read event: {error}"))?;

        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Char('q') => return Ok(None),
                KeyCode::Char('j') | KeyCode::Down => app.next(),
                KeyCode::Char('k') | KeyCode::Up => app.previous(),
                KeyCode::Enter => {
                    if let Some(workspace) = app.selected_workspace() {
                        return Ok(Some(workspace.name.clone()));
                    }
                }
                _ => {}
            }
        }
    }
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
    render_footer(frame, vertical_chunks[1]);
}

fn render_workspace_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = if app.workspaces.is_empty() {
        vec![ListItem::new("No workspaces found")]
    } else {
        app.workspaces
            .iter()
            .map(|workspace| {
                ListItem::new(format!("{:<20} {}", workspace.name, workspace.template))
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
        Some(workspace) => workspace_details_text(workspace),
        None => String::from(
            "No workspaces found.\n\nCreate one with:\n\n  tw init my-project --template rust --root .",
        ),
    };

    let details = Paragraph::new(text)
        .block(Block::default().title("Details").borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(details, area);
}

fn workspace_details_text(workspace: &Workspace) -> String {
    let mut text = String::new();

    text.push_str(&format!("name: {}\n", workspace.name));
    text.push_str(&format!("template: {}\n", workspace.template));
    text.push_str(&format!("root: {}\n", workspace.root));
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

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from("↑/↓ j/k move   Enter start   q quit"))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}
