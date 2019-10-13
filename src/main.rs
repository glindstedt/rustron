use std::sync::mpsc;
use std::{error, io};

use termion::raw::IntoRawMode;
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::widgets::{Block, Borders, List, SelectableList, Text, Widget};
use tui::{Frame, Terminal};

use rustron_lib::parser::neutron_message;

use crate::app::App;
use crate::events::Events;

mod app;
mod events;
mod midi;

// Used for primitive scrolling logic
fn bottom_slice<T>(array: &[T], max_size: usize) -> &[T] {
    let array_size = array.len();
    let start_index = if array_size < max_size {
        0
    } else {
        array_size - max_size
    };
    &array[start_index..]
}

fn render_command_history<B>(frame: &mut Frame<B>, rectangle: Rect, app: &App)
where
    B: Backend,
{
    let command_history = bottom_slice(app.command_history.as_slice(), rectangle.height as usize)
        .iter()
        .map(|event| Text::raw(event.to_string()));
    List::new(command_history)
        .block(
            Block::default()
                .title("Command History")
                .borders(Borders::ALL),
        )
        .render(frame, rectangle);
}

fn render_options_menu<B>(frame: &mut Frame<B>, rectangle: Rect, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(rectangle);

    // Old menu
    SelectableList::default()
        .block(Block::default())
        .items(&app.basic_menu.items)
        .select(Some(app.basic_menu.selection))
        .highlight_symbol(">>")
        .render(frame, chunks[0]);

    // Prototype new menu
    //TODO
}

fn render_midi_stream<B>(frame: &mut Frame<B>, rectangle: Rect, app: &App)
where
    B: Backend,
{
    let midi_messages = bottom_slice(app.midi_in_messages.as_slice(), rectangle.height as usize)
        .iter()
        .map(|event| match neutron_message(event.as_slice()) {
            Ok((_, msg)) => Text::raw(msg.to_string()),
            Err(_) => Text::raw(hex::encode(event)),
        });
    List::new(midi_messages)
        .block(
            Block::default()
                .title("MIDI Sysex Input")
                .borders(Borders::ALL),
        )
        .render(frame, rectangle);
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let key_events = Events::new();

    let (midi_in_sender, midi_in_receiver) = mpsc::channel();

    let app = &mut App::new();
    if let Err(error) = app.connection.register_midi_in_channel(midi_in_sender) {
        app.command_history.push(format!("{}", error))
    };

    while !app.should_quit {
        match midi_in_receiver.try_recv() {
            Ok(msg) => app.midi_in_messages.push(msg.into()),
            Err(_) => {}
        }
        terminal.draw(|mut frame| {
            let size = frame.size();
            let vertical_split = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(size);
            Block::default()
                .title("Rustron")
                .borders(Borders::ALL)
                .render(&mut frame, size);

            {
                // Left half
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(vertical_split[0]);

                render_options_menu(&mut frame, chunks[0], app);
                render_command_history(&mut frame, chunks[1], app);
            }

            render_midi_stream(&mut frame, vertical_split[1], app);
        })?;

        app.handle_event(key_events.next()?);
    }
    Ok(())
}
