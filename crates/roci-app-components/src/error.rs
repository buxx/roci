use derive_more::Constructor;
use gpui::*;

use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    text::TextView,
    v_flex, ActiveTheme, Icon, IconName, WindowExt,
};

#[derive(Constructor)]
pub struct WithButtonModalError<E: std::fmt::Debug + 'static> {
    label: String,
    details: Entity<ErrorDetails<E>>,
}

impl<E: std::fmt::Debug> Render for WithButtonModalError<E> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().gap_5().child(
            h_flex()
                .child(Icon::new(IconName::CircleX).text_color(cx.theme().red))
                .child(" ")
                .child(self.label.clone())
                .child(" ")
                .child(
                    Button::new("error-details-button")
                        .link()
                        .label("?")
                        .on_click(cx.listener(|this, _, window, cx| {
                            let label = this.label.clone();
                            let details = this.details.clone();

                            window.open_dialog(cx, move |modal, _window, _cx| {
                                modal.title(label.clone()).child(details.clone())
                            })
                        })),
                ),
        )
    }
}

#[derive(Constructor)]
pub struct ErrorDetails<E: std::fmt::Debug> {
    pub message: String,
    pub error: Option<E>,
}

impl<E: std::fmt::Debug + 'static> Render for ErrorDetails<E> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child(self.message.clone()).child(TextView::markdown(
            "error-detail",
            format!("```\n{:#?}\n```", self.error),
            window,
            cx,
        ))
    }
}
