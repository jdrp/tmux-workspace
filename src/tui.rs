use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    widgets::{Block, Borders, Paragraph},
};

pub fn run() -> Result<(), String> {
    let mut terminal =
        ratatui::try_init().map_err(|error| format!("failed to initialize TUI: {error}"))?;

    let result = run_app(&mut terminal);

    ratatui::restore();

    result
}

fn run_app(terminal: &mut DefaultTerminal) -> Result<(), String> {
    loop {
        terminal
            .draw(render)
            .map_err(|error| format!("failed to draw TUI: {error}"))?;

        let event = event::read().map_err(|error| format!("failed to read event: {error}"))?;

        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                _ => {}
            }
        }
    }
}

fn render(frame: &mut Frame) {
    let area = frame.area();

    let widget = Paragraph::new("tmux-workspace TUI\n\nq / Esc: quit")
        .block(Block::default().title("tw").borders(Borders::ALL));

    frame.render_widget(widget, area);
}
