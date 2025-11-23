use std::rc::Rc;

use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    input::InputState,
    menu::{DropdownMenu as _, PopupMenuItem},
    notification::NotificationType,
    select::{Select, SelectEvent, SelectState},
    *,
};
use roci_app_components::{bool::BooleanState, list::List};
use strum::IntoEnumIterator;

use crate::{
    config::{
        merge_request::ShowMergeRequest,
        refresh::RefreshEvery,
        theme::{load_theme, ThemeMode},
    },
    state::AppState,
};

mod configure;
mod error;
mod issue;
mod project;
mod todo;

pub const CONTAINER_PADDING: Pixels = px(15.);

pub struct Dashboard {
    notifications: Vec<(NotificationType, SharedString)>,
    projects: Entity<List<project::Projects>>,
    issues: Entity<List<issue::Issues>>,
    todos: Entity<List<todo::Todos>>,
    //
    new_protocol: Entity<SelectState<Vec<String>>>,
    new_host: Entity<InputState>,
    new_auth_key: Entity<InputState>,
    new_cert_insecure: Entity<BooleanState>,
    new_is_editing: Entity<BooleanState>,
    //
    refresh_every: Entity<SelectState<Vec<RefreshEvery>>>,
    show_merge_request: Entity<SelectState<Vec<ShowMergeRequest>>>,
    theme_mode: Entity<SelectState<Vec<ThemeMode>>>,
}

impl Dashboard {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = AppState::global(cx).config().clone();
        let gitlabs = config.gitlabs.clone();

        let projects = cx.new(|cx| {
            List(
                gitlabs
                    .iter()
                    .cloned()
                    .map(|config| cx.new(|cx| project::Projects::new(window, cx, config)))
                    .collect(),
            )
        });
        let issues = cx.new(|cx| {
            List(
                gitlabs
                    .iter()
                    .cloned()
                    .map(|config| cx.new(|cx| issue::Issues::new(window, cx, config)))
                    .collect(),
            )
        });
        let todos = cx.new(|cx| {
            List(
                gitlabs
                    .iter()
                    .cloned()
                    .map(|config| cx.new(|cx| todo::Todos::new(window, cx, config)))
                    .collect(),
            )
        });

        let new_protocol = cx.new(|cx| {
            SelectState::new(
                vec!["https://".to_string(), "http://".to_string()],
                Some(IndexPath::default()),
                window,
                cx,
            )
        });
        let new_host = cx.new(|cx| InputState::new(window, cx));
        let new_auth_key = cx.new(|cx| InputState::new(window, cx));
        let new_cert_insecure = cx.new(|_cx| BooleanState::new(false));
        let new_is_editing = cx.new(|_cx| BooleanState::new(false));

        let refresh_every_index = RefreshEvery::iter()
            .collect::<Vec<RefreshEvery>>()
            .iter()
            .position(|v| v == &config.refresh_every);
        let refresh_every = cx.new(|cx| {
            SelectState::new(
                RefreshEvery::iter().collect(),
                refresh_every_index.map(|v| IndexPath::new(v)),
                window,
                cx,
            )
        });
        cx.subscribe_in(&refresh_every, window, Self::on_select_refresh_every)
            .detach();

        let show_merge_request_index = ShowMergeRequest::iter()
            .collect::<Vec<ShowMergeRequest>>()
            .iter()
            .position(|v| v == &config.show_merge_request);
        let show_merge_request = cx.new(|cx| {
            SelectState::new(
                ShowMergeRequest::iter().collect(),
                show_merge_request_index.map(|v| IndexPath::new(v)),
                window,
                cx,
            )
        });
        cx.subscribe_in(
            &show_merge_request,
            window,
            Self::on_select_show_merge_request,
        )
        .detach();

        let theme_mode_index = ThemeMode::iter()
            .collect::<Vec<ThemeMode>>()
            .iter()
            .position(|v| v == &config.theme_mode);
        let theme_mode = cx.new(|cx| {
            SelectState::new(
                ThemeMode::iter().collect(),
                theme_mode_index.map(|v| IndexPath::new(v)),
                window,
                cx,
            )
        });
        cx.subscribe_in(&theme_mode, window, Self::on_select_theme_mode)
            .detach();

        Self {
            notifications: vec![],
            projects,
            issues,
            todos,
            //
            new_protocol,
            new_host,
            new_auth_key,
            new_cert_insecure,
            new_is_editing,
            //
            refresh_every,
            show_merge_request,
            theme_mode,
        }
    }

    pub fn with_notifications(
        mut self,
        notifications: Vec<(NotificationType, SharedString)>,
    ) -> Self {
        self.notifications.extend(notifications);
        self
    }

    fn refresh_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        *self = Self::new(window, cx);
    }

    fn on_select_refresh_every(
        &mut self,
        _: &Entity<SelectState<Vec<RefreshEvery>>>,
        event: &SelectEvent<Vec<RefreshEvery>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(Some(refresh_every)) = event else {
            return;
        };
        let state = AppState::global_mut(cx);
        let mut new_config = state.config().clone();
        new_config.refresh_every = refresh_every.clone();

        if let Err(error) = state.replace_config(new_config.clone()) {
            window.push_notification(
                (
                    NotificationType::Error,
                    SharedString::new(format!("Failed to write config on disk: {:#}", error)),
                ),
                cx,
            );
            return;
        }

        self.refresh_all(window, cx);
    }

    fn on_select_show_merge_request(
        &mut self,
        _: &Entity<SelectState<Vec<ShowMergeRequest>>>,
        event: &SelectEvent<Vec<ShowMergeRequest>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(Some(show_merge_request)) = event else {
            return;
        };
        let state = AppState::global_mut(cx);
        let mut new_config = state.config().clone();
        new_config.show_merge_request = show_merge_request.clone();

        if let Err(error) = state.replace_config(new_config.clone()) {
            window.push_notification(
                (
                    NotificationType::Error,
                    SharedString::new(format!("Failed to write config on disk: {:#}", error)),
                ),
                cx,
            );
            return;
        }

        self.refresh_all(window, cx);
    }

    fn on_select_theme_mode(
        &mut self,
        _: &Entity<SelectState<Vec<ThemeMode>>>,
        event: &SelectEvent<Vec<ThemeMode>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(Some(theme_mode)) = event else {
            return;
        };
        let state = AppState::global_mut(cx);
        let mut new_config = state.config().clone();
        new_config.theme_mode = theme_mode.clone();
        let (theme, theme_error) = match load_theme(*theme_mode) {
            Ok(value) => value,
            Err(error) => {
                window.push_notification(
                    (
                        NotificationType::Error,
                        SharedString::new(format!("Failed to apply theme: {:#}", error)),
                    ),
                    cx,
                );
                return;
            }
        };
        Theme::global_mut(cx).apply_config(&Rc::new(theme));

        if let Some(error) = theme_error {
            window.push_notification(error.into_notification(), cx);
            return;
        }

        let state = AppState::global_mut(cx);
        if let Err(error) = state.replace_config(new_config.clone()) {
            window.push_notification(
                (
                    NotificationType::Error,
                    SharedString::new(format!("Failed to write config on disk: {:#}", error)),
                ),
                cx,
            );
            return;
        }
    }
}

impl Render for Dashboard {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        let refresh_every = self.refresh_every.clone();

        while let Some((notification_type, notification_message)) = self.notifications.pop() {
            window.push_notification(
                (notification_type, SharedString::new(notification_message)),
                cx,
            );
        }

        div()
            .child(
                v_flex()
                    .child(
                        h_flex()
                            .child(
                                Button::new("title-bar-edit")
                                    .outline()
                                    .label("Edit")
                                    .ghost()
                                    .dropdown_menu({
                                        let view = view.clone();

                                        move |this, window, _cx| {
                                            this.item(
                                                PopupMenuItem::new("Configure access").on_click(
                                                    window.listener_for(
                                                        &view,
                                                        |this, _event, window, cx| {
                                                            this.show_configure_dialog(window, cx);
                                                        },
                                                    ),
                                                ),
                                            )
                                        }
                                    }),
                            )
                            .child(
                                Button::new("Refresh now")
                                    .icon(IconName::Loader)
                                    .on_click({
                                        let view = view.clone();

                                        window.listener_for(&view, |this, _, window, cx| {
                                            this.refresh_all(window, cx);
                                        })
                                    })
                                    .tooltip("Refresh now"),
                            )
                            .child(Select::new(&refresh_every))
                            .child(Select::new(&self.show_merge_request))
                            .child(Select::new(&self.theme_mode)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .paddings(Edges::all(CONTAINER_PADDING))
                            .child(self.todos.clone())
                            .child(self.projects.clone())
                            .child(self.issues.clone()),
                    ),
            )
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
            .scrollable(Axis::Vertical)
    }
}
