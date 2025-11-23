use std::rc::Rc;

use gpui::*;
use gpui_component::Theme;
use gpui_component::*;

use crate::{config::Config, logging::configure_logging};

mod assets;
mod config;
mod dashboard;
mod logging;
mod runtime;
mod state;

fn main() -> Result<(), anyhow::Error> {
    configure_logging();
    prepare_runtime!();

    let app = Application::new().with_assets(assets::Assets);
    let (config, info) = Config::from_env()?;
    let (theme, theme_error) = config::theme::load_theme(config.theme_mode)?;

    app.run(move |cx| {
        gpui_component::init(cx);
        state::AppState::init(cx, config);
        Theme::global_mut(cx).apply_config(&Rc::new(theme));

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(800.), px(600.)), cx)),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let mut notifications = vec![];
                if let Some(info) = info {
                    notifications.push(info.into_notification());
                }
                if let Some(theme_error) = theme_error {
                    notifications.push(theme_error.into_notification());
                }

                let view = cx.new(|cx| {
                    dashboard::Dashboard::new(window, cx).with_notifications(notifications)
                });
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });

    Ok(())
}
