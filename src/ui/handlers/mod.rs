mod common_key_events;
mod dialog;
mod empty;
mod portfolio;
mod input;
mod order_form;
mod watch_list;
mod account_list;
mod search_results;
mod ticker_detail;

use crate::app::{ActiveBlock, App, RouteId, OrderFormState};
use super::key::Key;

pub use input::handler as input_handler;

pub fn handle_app(key: Key, app: &mut App) {
    // First handle any global event and then move to block event
    match key {
        Key::Esc => {
            handle_escape(app);
        }
        // _ if key == app.user_config.keys.help => {
        //   app.set_current_route_state(Some(ActiveBlock::HelpMenu), None);
        // }
        _ if key == app.user_config.keys.search => {
          app.set_current_route_state(Some(ActiveBlock::Input), Some(ActiveBlock::Input));
        }
        _ => handle_block_events(key, app),
    }
}

// Handle event for the current active block
fn handle_block_events(key: Key, app: &mut App) {
    let current_route = app.get_current_route();
    match current_route.active_block {
        ActiveBlock::Input => {
            input::handler(key, app);
        }
        // ActiveBlock::HelpMenu => {
        //     help_menu::handler(key, app);
        // }
        // ActiveBlock::Error => {
        //     error_screen::handler(key, app);
        // }
        ActiveBlock::Dialog(_) => {
            dialog::handler(key, app);
        }
        ActiveBlock::SearchResults => {
            search_results::handler(key, app);
        }
        // ActiveBlock::Home => {
        //     home::handler(key, app);
        // }
        ActiveBlock::WatchList => {
            watch_list::handler(key, app);
        }
        ActiveBlock::TickerDetail => {
            ticker_detail::handler(key, app);
        }
        ActiveBlock::Portfolio => {
            portfolio::handler(key, app);
        }
        ActiveBlock::AccountList => {
            account_list::handler(key, app);
        }
        ActiveBlock::OrderForm => {
            order_form::handler(key, app);
        }
        ActiveBlock::Empty => {
            empty::handler(key, app);
        }
    //     ActiveBlock::RecentlyPlayed => {
    //         recently_played::handler(key, app);
    //     }
    //     ActiveBlock::Dialog(_) => {
    //         dialog::handler(key, app);
    //     }
         _ => {}
    }
}

fn handle_escape(app: &mut App) {
    match app.get_current_route().active_block {
        ActiveBlock::SearchResults => {
            app.search_results.tickers = None;
            app.search_results.selected_ticker_index = None;
            app.pop_navigation_stack();
        }
        ActiveBlock::TickerDetail => {
            if let (Some(_tickers), Some(_selected_ticker_index)) =
                (&app.search_results.tickers, &app.search_results.selected_ticker_index)
                {
                    app.push_navigation_stack(RouteId::Search, ActiveBlock::SearchResults);
                    app.selected_ticker = None;
                } else {
                    app.pop_navigation_stack();
                }
        }
        ActiveBlock::OrderForm => {
            match app.order_form_state {
                OrderFormState::Submit => {
                    app.order_form_state = OrderFormState::Quantity;
                    app.set_current_route_state(Some(ActiveBlock::Input), Some(ActiveBlock::Input));
                }
                _ => ()
            }
            app.pop_navigation_stack();
        }
        ActiveBlock::Dialog(_) => {
            app.pop_navigation_stack();
        }
        ActiveBlock::Error => {
            app.pop_navigation_stack();
        }
        _ => {
            // if let OrderFormState::Quantity = app.order_form_state {
            //     app.order_form_state = OrderFormState::Initial;
            //     app.cancel_preview_order();
            //     app.push_navigation_stack(RouteId::TickerDetail, ActiveBlock::TickerDetail);
            // // if RouteId::OrderForm == app.get_current_route().id {
            // //     app.order_form_state = OrderFormState::Quantity;
            // //     app.pop_navigation_stack();
            // } else {
            app.set_current_route_state(Some(ActiveBlock::Empty), None);
            // }
        }
    }
}

