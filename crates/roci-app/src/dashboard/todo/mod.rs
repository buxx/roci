use gitlab::{api::AsyncQuery, AsyncGitlab};
use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    label::Label,
    v_flex, Icon, IconName, StyledExt,
};
use roci_app_components::{error::WithButtonModalError, with_button_error, LoadState};
use tracing_unwrap::ResultExt;

use crate::{
    dashboard::{error::GitlabError, todo::endpoint::MyTodos},
    state::{gitlab::todo::Todo, AppState},
};

mod endpoint;

pub struct Todos {
    inner: Entity<LoadState<TodosInner, WithButtonModalError<GitlabError>>>,
    host: String,
}

impl Todos {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        config: crate::config::gitlab_::Gitlab,
    ) -> Self {
        let host = config.host.clone();
        let host_ = config.host.clone();
        let gitlabs = AppState::global(cx).gitlabs();

        let config_ = config.clone();
        cx.spawn_in(window, async move |this, cx| {
            match gitlabs.get(&config_).await {
                Ok(gitlab) => match get_todos(gitlab).await {
                    Ok(todos_) => {
                        let _ = this.update_in(cx, |this, _window, cx| {
                            let todos = cx.new(|_cx| TodosInner(todos_));
                            this.inner = cx.new(|_cx| LoadState::Ready(todos));
                        });
                    }
                    Err(error) => {
                        let _ = this.update_in(cx, |this, _window, cx| {
                            this.inner = cx.new(|cx| {
                                with_button_error!(
                                    cx,
                                    "Load error".into(),
                                    format!("Error during load todos of {}", host),
                                    error.into()
                                )
                            });
                        });
                    }
                },
                Err(error) => {
                    let _ = this.update_in(cx, |this, _window, cx| {
                        this.inner = cx.new(|cx| {
                            with_button_error!(
                                cx,
                                "Load error".into(),
                                format!("Error during connect gitlab {}", host),
                                error.into()
                            )
                        });
                    });
                }
            };
        })
        .detach();

        let config_ = config.clone();
        let interval = AppState::global(cx).config().refresh_every.duration();
        cx.spawn_in(window, async move |this, cx| {
            Timer::after(interval).await;
            let _ = this.update_in(cx, |this, window, cx| {
                *this = Self::new(window, cx, config_.clone());
                cx.notify();
            });
        })
        .detach();

        Self {
            inner: cx.new(|_cx| LoadState::Loading),
            host: host_,
        }
    }
}

impl Render for Todos {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let host = self.host.clone();

        div()
            .child(
                h_flex()
                    .child(Icon::new(IconName::ArrowRight))
                    .child(Label::new(format!("{} todos", host)).text_xl()),
            )
            .child(self.inner.clone())
    }
}

pub struct TodosInner(Vec<Todo>);

impl Render for TodosInner {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .paddings(Edges::all(px(5.)))
            .when(!self.0.is_empty(), |element| {
                element.children(self.0.iter().map(|todo| {
                    let target_url = todo.target_url.clone();

                    h_flex().id(ElementId::Integer(todo.id)).child(
                        Button::new("id-link")
                            .link()
                            .label(todo.body.to_string())
                            .on_click(move |_, _, _| {
                                open::that(target_url.clone()).unwrap_or_log();
                            }),
                    )
                }))
            })
            .when(self.0.is_empty(), |element| {
                element.child("n/a".to_string())
            })
    }
}

async fn get_todos(gitlab: AsyncGitlab) -> Result<Vec<Todo>, GitlabError> {
    Ok(MyTodos.query_async(&gitlab).await?)
}
