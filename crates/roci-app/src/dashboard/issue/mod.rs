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
    dashboard::{error::GitlabError, issue::endpoint::MyIssues},
    state::{gitlab::issue::Issue, AppState},
};

mod endpoint;

pub struct Issues {
    inner: Entity<LoadState<IssuesInner, WithButtonModalError<GitlabError>>>,
    host: String,
}

impl Issues {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        config: crate::config::gitlab_::Gitlab,
    ) -> Self {
        let host = config.host.clone();
        let host_ = config.host.clone();
        let gitlabs = AppState::global(cx).gitlabs();

        let config_ = config.clone();
        cx.spawn_in(window, async move |issues, cx| {
            match gitlabs.get(&config_).await {
                Ok(gitlab) => match get_issues(gitlab).await {
                    Ok(issues_) => {
                        let _ = issues.update_in(cx, |issues, _window, cx| {
                            let issues_ = cx.new(|_cx| IssuesInner(issues_));
                            issues.inner = cx.new(|_cx| LoadState::Ready(issues_));
                        });
                    }
                    Err(error) => {
                        let _ = issues.update_in(cx, |issues, _window, cx| {
                            issues.inner = cx.new(|cx| {
                                with_button_error!(
                                    cx,
                                    "Load error".into(),
                                    format!("Error during load issues of {}", host),
                                    error.into()
                                )
                            });
                        });
                    }
                },
                Err(error) => {
                    let _ = issues.update_in(cx, |issues, _window, cx| {
                        issues.inner = cx.new(|cx| {
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
        cx.spawn_in(window, async move |issues, cx| {
            Timer::after(interval).await;
            let _ = issues.update_in(cx, |issues, window, cx| {
                *issues = Self::new(window, cx, config_.clone());
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

impl Render for Issues {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let host = self.host.clone();

        div()
            .child(
                h_flex()
                    .child(Icon::new(IconName::ArrowRight))
                    .child(Label::new(format!("{} issues", host)).text_xl()),
            )
            .child(self.inner.clone())
    }
}

pub struct IssuesInner(Vec<Issue>);

impl Render for IssuesInner {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .paddings(Edges::all(px(5.)))
            .when(!self.0.is_empty(), |element| {
                element.children(self.0.iter().map(|issue| {
                    let web_url = issue.web_url.clone();

                    h_flex()
                        .id(ElementId::Integer(issue.id))
                        .child(
                            Button::new("id-link")
                                .link()
                                .label(format!("#{}", issue.iid))
                                .on_click(move |_, _, _| {
                                    open::that(web_url.clone()).unwrap_or_log();
                                }),
                        )
                        .child(" ".to_string())
                        .child(issue.title.clone())
                }))
            })
            .when(self.0.is_empty(), |element| {
                element.child("n/a".to_string())
            })
    }
}

async fn get_issues(
    gitlab: AsyncGitlab,
) -> Result<Vec<crate::state::gitlab::issue::Issue>, GitlabError> {
    Ok(MyIssues::new("assigned_to_me", Some("opened"))
        .query_async(&gitlab)
        .await?)
}
