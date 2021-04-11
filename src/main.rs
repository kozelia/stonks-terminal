mod clients;
mod config;
mod session;
mod stonks_error;
mod network;
mod store;
mod ui;
mod utils;
mod app;
use config::{ClientConfig, UserConfig};
use store::AuthInMemoryStore;
use crate::clients::{Etrade};
use crate::session::{Credentials, Mode, Session};
use crate::store::{Store};
use crate::network::{Network, IoEvent};
use std::sync::mpsc::Receiver;
use stonks_error::RuntimeError;
use clap::{App as ClapApp, Arg};
use crossterm::{
  cursor::MoveTo,
  event::{DisableMouseCapture, EnableMouseCapture},
  execute,
  style::Print,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
  ExecutableCommand,
};
use std::{
    sync::Arc,
    io::{stdout},
    time::{SystemTime},
};
use log::debug;
use tokio::sync::Mutex;
use ui::{event::Events, event::Event, key::Key};
use tui::{
  backend::{Backend, CrosstermBackend},
  Terminal,
};
use app::{ActiveBlock, App};

#[tokio::main]
async fn main() -> Result<(), RuntimeError> {
    let matches = ClapApp::new("Stonksss")
        .version("1.0")
        .author("Scott N. <snewcomer24@gmail.com>")
        .about("Trade easy")
        .after_help(
          "Your etrade Client ID and Client Secret are stored in $HOME/.config/stonks-terminal/client.yml",
        )
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .help("Specify configuration file path.")
            .takes_value(true))
        .arg(Arg::with_name("mode")
            .short("m")
            .long("mode")
            .help("Specify either --sandbox or --live. Default is --live.")
            .takes_value(true))
        .get_matches();

    match matches.value_of("mode") {
        Some("sandbox") => run(Mode::Sandbox).await,
        _ => run(Mode::Live).await,
    }
}

async fn run(mode: Mode) -> Result<(), RuntimeError> {
    let mut client_config = ClientConfig::new();
    // ask user for configuration details
    let config_paths = client_config.load_config()?;
    let user_config = UserConfig::new();

    env_logger::init();

    debug!("Request token in flight for {:?} mode", mode);

    let mut session = Session::new(mode, AuthInMemoryStore::new(), config_paths);

    // check if we have a non expired access token
    let etrade_token_expiry = utils::midnight_eastern(1);
    if session.expired_access_token() {
        // get consumer and access tokens
        session.full_access_flow(client_config.clone()).await?;
    }

    let (sync_io_tx, sync_io_rx) = std::sync::mpsc::channel::<IoEvent>();

    // network APIs interface
    let etrade = Etrade::new(client_config.clone());

    // Initialise app state
    let app = Arc::new(Mutex::new(App::new(
        sync_io_tx,
        user_config.clone(),
        etrade_token_expiry,
    )));

    let cloned_app = Arc::clone(&app);

    std::thread::spawn(move || {
        let mut network = Network::new(etrade, session, client_config, &app);
        start_tokio(sync_io_rx, &mut network);
    });

    start_ui(&cloned_app).await?;

    Ok(())
}

#[tokio::main]
async fn start_tokio<T>(rx: Receiver<IoEvent>, network: &mut Network<T>)
where T: Store
{
    // the sender from App will respond to user key events and receive here...
    while let Ok(event) = rx.recv() {
        network.handle_network_event(event).await;
    }
}

async fn start_ui(app: &Arc<Mutex<App>>) -> Result<(), RuntimeError> {
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();
    let mut is_first_render = true;

    loop {
        let mut app = app.lock().await;
        let current_route = app.get_current_route();
        if current_route.active_block == ActiveBlock::Input {
            terminal.show_cursor()?;

            // Put the cursor back inside the input box
            terminal.backend_mut().execute(MoveTo(
              2 + app.input_cursor_position,
              2,
            ))?;
        } else {
            terminal.hide_cursor()?;
        }

        terminal.draw(|mut f| ui::draw_main(&mut f, &app))?;

        let mut midnight = utils::midnight_eastern(1);
        // Handle authentication refresh
        if midnight < app.etrade_token_expiry {
            // reset to tomorrow
            midnight = utils::midnight_eastern(1);
            app.etrade_token_expiry = midnight;
            app.dispatch(IoEvent::RefreshAuthentication);
        }

        match events.next()? {
            Event::Input(key) => {
                if key == Key::Ctrl('c') {
                    break;
                }

                let current_active_block = app.get_current_route().active_block;
                if current_active_block == ActiveBlock::Input {
                    ui::handlers::input_handler(key, &mut app);
                } else {
                    ui::handlers::handle_app(key, &mut app);
                }
            }
            Event::Tick => {}
        }

        if is_first_render {
          app.dispatch(IoEvent::GetPortfolio);
          is_first_render = false;
        }

    }

    terminal.show_cursor()?;
    close_application()?;

    Ok(())
}

fn close_application() -> Result<(), RuntimeError> {
  disable_raw_mode()?;

  let mut stdout = stdout();
  execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
  Ok(())
}

