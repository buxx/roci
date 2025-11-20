use gpui::*;
use gpui_component::spinner::Spinner;

pub mod bool;
pub mod error;
pub mod list;

pub enum LoadState<T: Render, E: Render> {
    Loading,
    Ready(Entity<T>),
    Error(Entity<E>),
}

impl<T: Render, E: Render> Render for LoadState<T, E> {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match self {
            LoadState::Loading => div().child(Spinner::new()),
            LoadState::Ready(content) => div().child(content.clone()),
            LoadState::Error(details) => div().child(details.clone()),
        }
    }
}

#[macro_export]
macro_rules! with_button_error {
    ($cx:expr, $label:expr, $message:expr, $error:expr) => {{
        let details =
            $cx.new(|_cx| roci_app_components::error::ErrorDetails::new($message, Some($error)));
        let error_ =
            $cx.new(|_cx| roci_app_components::error::WithButtonModalError::new($label, details));
        LoadState::Error(error_)
    }};
}
