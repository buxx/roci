use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    form::{field, h_form},
    h_flex,
    input::Input,
    label::Label,
    notification::NotificationType,
    select::Select,
    switch::Switch,
    v_flex, IconName, WindowExt,
};

use crate::{
    config::{gitlab_::Gitlab, set_password},
    dashboard::Dashboard,
    state::AppState,
};

impl Dashboard {
    pub fn show_configure_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_new_instance(window, cx);

        let view = cx.entity().clone();
        let new_protocol = self.new_protocol.clone();
        let new_host = self.new_host.clone();
        let new_auth_key = self.new_auth_key.clone();
        let new_cert_insecure = self.new_cert_insecure.clone();
        let new_is_editing = self.new_is_editing.clone();

        window.open_dialog(cx, move |dialog, _window, cx| {
            let view = view.clone();
            let new_cert_insecure_ = new_cert_insecure.read(cx);
            let new_is_editing = new_is_editing.read(cx);

            dialog
                .title("Configure access")
                .child(Label::new("Add gitlab instance").text_lg())
                .child(
                    h_form()
                        .child(field().label("Protocol").child(Select::new(&new_protocol)))
                        .child(
                            field()
                                .label("Host")
                                .child(Input::new(&new_host).disabled(new_is_editing.get()))
                                .required(true),
                        )
                        .child(
                            field()
                                .label("Access token")
                                .child(Input::new(&new_auth_key))
                                .required(true),
                        )
                        .child(
                            field().label("Insecure certificate").child(
                                Switch::new("insecure")
                                    .checked(new_cert_insecure_.get())
                                    .on_click({
                                        let new_cert_insecure = new_cert_insecure.clone();
                                        move |_, _window, cx| {
                                            new_cert_insecure.update(cx, |this, _cx| {
                                                this.toggle();
                                            });
                                        }
                                    }),
                            ),
                        ),
                )
                .child(field().label_indent(false).child({
                    let label = if !new_is_editing.get() {
                        "Add"
                    } else {
                        "Update"
                    };
                    Button::new("submit").primary().child(label).on_click({
                        let view = view.clone();

                        move |_, window, cx| {
                            view.update(cx, |this, cx| {
                                this.validate_new_host(window, cx);
                            });
                        }
                    })
                }))
                .child(
                    v_flex()
                        .child(Label::new("Existing gitlab instances").text_lg())
                        .children(AppState::global(cx).config().gitlabs.iter().map(|gitlab| {
                            let host = gitlab.host.clone();

                            h_flex()
                                .child(
                                    Button::new(ElementId::Name(SharedString::new(format!(
                                        "delete-host-{}",
                                        host.clone()
                                    ))))
                                    .icon(IconName::Close)
                                    .link()
                                    .on_click({
                                        let view = view.clone();
                                        let host = host.clone();

                                        move |_, window, cx| {
                                            view.update(cx, |this, cx| {
                                                this.delete_host(window, cx, host.to_string());
                                            });
                                        }
                                    }),
                                )
                                .child(
                                    Button::new(ElementId::Name(SharedString::new(format!(
                                        "edit-host-{}",
                                        host.clone()
                                    ))))
                                    .icon(IconName::Inspector)
                                    .link()
                                    .on_click({
                                        let view = view.clone();
                                        let host = host.clone();

                                        move |_, window, cx| {
                                            view.update(cx, |this, cx| {
                                                this.prepare_edit_host(
                                                    window,
                                                    cx,
                                                    host.to_string(),
                                                );
                                            });
                                        }
                                    }),
                                )
                                .child(" ".to_string())
                                .child(Label::new(host.to_string()))
                        })),
                )
        })
    }

    fn delete_host(&mut self, window: &mut Window, cx: &mut Context<Self>, host: String) {
        let state = AppState::global_mut(cx);
        let mut new_config = state.config().clone();
        new_config.gitlabs.retain(|g| g.host != host);

        if let Err(error) = state.replace_config(new_config) {
            window.push_notification(
                (
                    NotificationType::Error,
                    SharedString::new(format!("Failed to write config on disk: {:#}", error)),
                ),
                cx,
            );
            return;
        }

        window.push_notification(format!("Gitlab instance {} deleted", host), cx);
    }

    fn prepare_edit_host(&mut self, window: &mut Window, cx: &mut Context<Self>, host: String) {
        let state = AppState::global_mut(cx);
        let config = state.config().clone();

        if let Some(gitlab) = config.gitlabs.iter().find(|g| g.host == host) {
            self.reset_new_instance(window, cx);

            self.new_cert_insecure
                .update(cx, |this, _cx| this.set(gitlab.cert_insecure));
            self.new_protocol.update(cx, |this, cx| {
                this.set_selected_value(&gitlab.protocol(), window, cx);
            });
            self.new_host.update(cx, |this, cx| {
                this.set_value(SharedString::new(gitlab.host.clone()), window, cx);
            });
            self.new_is_editing.update(cx, |this, _cx| this.set(true));
        }
    }

    fn validate_new_host(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let protocol = self.new_protocol.read(cx).selected_value();
        let host = self.new_host.read(cx).value();
        let token = self.new_auth_key.read(cx).value();
        let cert_insecure = self.new_cert_insecure.read(cx).get();
        let is_update = self.new_is_editing.read(cx).get();

        if !is_update && (host.is_empty() || token.is_empty()) {
            window.push_notification("Please fill both fields.", cx);
            return;
        }

        if !token.is_empty() {
            if let Err(error) = set_password(&host, &token) {
                window.push_notification(
                    (
                        NotificationType::Error,
                        SharedString::new(format!("Can't save token as secret: {}", error)),
                    ),
                    cx,
                );
                return;
            }
        }

        let insecure = protocol == Some(&"http://".to_string());
        let state = AppState::global_mut(cx);

        let mut new_config = state.config().clone();
        if is_update {
            if let Some(gitlab) = new_config.gitlabs.iter_mut().find(|g| g.host == host) {
                gitlab.insecure = insecure;
                gitlab.cert_insecure = cert_insecure;
                state.gitlabs().invalidate(&host);
            }
        } else {
            let gitlab_ = Gitlab::empty(host.clone().into(), insecure, cert_insecure);
            new_config.gitlabs.push(gitlab_.clone());
        };

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
        window.push_notification(format!("Gitlab instance {} added", host), cx);

        self.reset_new_instance(window, cx);
    }

    fn reset_new_instance(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.new_host.update(cx, |this, cx| {
            this.set_value(SharedString::new(""), window, cx);
        });
        self.new_auth_key.update(cx, |this, cx| {
            this.set_value(SharedString::new(""), window, cx);
        });
        self.new_cert_insecure
            .update(cx, |this, _cx| this.set(false));
        self.new_is_editing.update(cx, |this, _cx| this.set(false));
    }
}
