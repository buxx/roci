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

        let mut window_size = size(px(1600.0), px(1200.0));
        if let Some(display) = cx.primary_display() {
            let display_size = display.bounds().size;
            window_size.width = window_size.width.min(display_size.width * 0.85);
            window_size.height = window_size.height.min(display_size.height * 0.85);
        }
        let window_bounds = Bounds::centered(None, window_size, cx);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(window_bounds)),
            titlebar: Some(TitlebarOptions {
                title: None,
                appears_transparent: true,
                traffic_light_position: Some(point(px(9.0), px(9.0))),
            }),
            window_min_size: Some(gpui::Size {
                width: px(640.),
                height: px(480.),
            }),
            kind: WindowKind::Normal,
            #[cfg(target_os = "linux")]
            window_background: gpui::WindowBackgroundAppearance::Transparent,
            #[cfg(target_os = "linux")]
            window_decorations: Some(gpui::WindowDecorations::Client),
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
