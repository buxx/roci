use gpui::*;
use gpui_component::v_flex;

#[derive(Clone)]
pub struct List<T: Render>(pub Vec<Entity<T>>);

impl<T: Render + 'static> Render for List<T> {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().children(self.0.iter().map(|item| item.clone()))
    }
}
