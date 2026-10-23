use std::ops::{Deref, DerefMut};

use color_eyre::eyre::Result;
use crossterm::cursor;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEventKind,
};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

use crate::tui::event::Event;

pub(crate) mod event;

#[derive(Debug)]
pub struct Tui {
    pub terminal: ratatui::Terminal<CrosstermBackend<std::io::Stderr>>,
    pub cancellation_token: CancellationToken,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_tx: UnboundedSender<Event>,
    pub frame_rate: f64,
    pub mouse: bool,
}

#[allow(dead_code)]
impl Tui {
    pub fn new() -> Result<Self> {
        let frame_rate = 60.0;
        let terminal = ratatui::Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let mouse = false;
        Ok(Self {
            terminal,
            cancellation_token,
            event_rx,
            event_tx,
            frame_rate,
            mouse,
        })
    }

    pub fn frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    pub fn mouse(mut self, mouse: bool) -> Self {
        self.mouse = mouse;
        self
    }

    pub fn start(&mut self) {
        let render_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);
        self.cancel();
        self.cancellation_token = CancellationToken::new();
        let _cancellation_token = self.cancellation_token.clone();
        let _event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut render_interval = tokio::time::interval(render_delay);
            _event_tx.send(Event::Init).unwrap();

            loop {
                let render_tick = render_interval.tick();
                let crossterm_fuse = reader.next().fuse();
                tokio::select! {
                    _ = _cancellation_token.cancelled() => {
                        break;
                    }
                    crossterm_event = crossterm_fuse => {
                        match crossterm_event {
                            Some(Ok(evt)) => {
                                match evt {
                                    CrosstermEvent::Key(key) => {
                                        if key.kind == KeyEventKind::Press {
                                            _event_tx.send(Event::Key(key)).unwrap();
                                        }
                                    },
                                    CrosstermEvent::Mouse(mouse) => {
                                        _event_tx.send(Event::Mouse(mouse)).unwrap();
                                    },
                                    CrosstermEvent::Resize(x, y) => {
                                        _event_tx.send(Event::Resize(x, y)).unwrap();
                                    },
                                    CrosstermEvent::FocusLost => {
                                        _event_tx.send(Event::FocusLost).unwrap();
                                    },
                                    CrosstermEvent::FocusGained => {
                                        _event_tx.send(Event::FocusGained).unwrap();
                                    },
                                    CrosstermEvent::Paste(_) => {
                                        /* ignored by this app */
                                    },
                                }
                            }
                            Some(Err(_)) => {
                                _event_tx.send(Event::Error).unwrap();
                            }
                            None => {},
                        }
                    },
                    _ = render_tick => {
                        _event_tx.send(Event::Render).unwrap();
                    },
                }
            }
        });
    }

    pub fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stderr(), EnterAlternateScreen, cursor::Hide)?;

        if self.mouse {
            crossterm::execute!(std::io::stderr(), EnableMouseCapture)?;
        }

        self.start();
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        if self.mouse {
            crossterm::execute!(std::io::stderr(), DisableMouseCapture)?;
        }
        if crossterm::terminal::is_raw_mode_enabled()? {
            self.flush()?;
            crossterm::terminal::disable_raw_mode()?;
        }
        crossterm::execute!(std::io::stderr(), LeaveAlternateScreen, cursor::Show)?;
        Ok(())
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    pub fn resume(&mut self) -> Result<()> {
        self.enter()?;
        Ok(())
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.event_rx
            .recv()
            .await
            .ok_or(color_eyre::eyre::eyre!("Unable to get event"))
    }
}

impl Deref for Tui {
    type Target = ratatui::Terminal<CrosstermBackend<std::io::Stderr>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}
