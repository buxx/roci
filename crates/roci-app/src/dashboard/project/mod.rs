use gitlab::{api::AsyncQuery, AsyncGitlab};
use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    form::{field, v_form},
    h_flex,
    input::{Input, InputState},
    label::Label,
    notification::NotificationType,
    Icon, IconName, WindowExt,
};
use roci_app_components::{error::WithButtonModalError, list::List, with_button_error, LoadState};

use crate::{
    dashboard::{
        error::GitlabError,
        project::{merge_request::MergeRequests, pipeline::Pipelines},
    },
    state::AppState,
};

mod merge_request;
mod pipeline;

pub struct Projects {
    inner: Entity<LoadState<List<Project>, WithButtonModalError<GitlabError>>>,
    config: crate::config::gitlab_::Gitlab,
    new_project_ids: Entity<InputState>,
}

impl Projects {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        config: crate::config::gitlab_::Gitlab,
    ) -> Self {
        let project_ids = config
            .project_ids
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(",");

        Self::init(window, cx, config.clone());

        Self {
            inner: cx.new(|_cx| LoadState::Loading),
            config,
            new_project_ids: cx.new(|cx| {
                let mut value = InputState::new(window, cx);
                value.set_value(SharedString::new(project_ids), window, cx);
                value
            }),
        }
    }

    fn init(window: &mut Window, cx: &mut Context<Self>, config: crate::config::gitlab_::Gitlab) {
        let host = config.host.clone();
        let gitlabs = AppState::global(cx).gitlabs();

        cx.spawn_in(window, async move |projects, cx| {
            match gitlabs.get(&config).await {
                Ok(gitlab) => {
                    let _ = projects.update_in(cx, |projects, window, cx| {
                        let projects_ = Self::projects(window, cx, config, gitlab);
                        let projects_ = cx.new(|_cx| projects_);
                        projects.inner = cx.new(|_cx| LoadState::Ready(projects_));

                        cx.notify();
                    });
                }
                Err(error) => {
                    let _ = projects.update_in(cx, |projects, _window, cx| {
                        projects.inner = cx.new(|cx| {
                            with_button_error!(
                                cx,
                                "Load error".into(),
                                format!("Error during load gitlab {}", host),
                                error.into()
                            )
                        });

                        cx.notify();
                    });
                }
            };
        })
        .detach();
    }

    fn projects(
        window: &mut Window,
        cx: &mut Context<Self>,
        config: crate::config::gitlab_::Gitlab,
        gitlab: AsyncGitlab,
    ) -> List<Project> {
        List(
            config
                .project_ids
                .iter()
                .map(|project_id| {
                    cx.new(|cx| Project::new(window, cx, gitlab.clone(), *project_id))
                })
                .collect::<Vec<Entity<Project>>>(),
        )
    }

    fn show_project_ids_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.entity().clone();
        let project_ids = self.new_project_ids.clone();

        window.open_dialog(cx, move |dialog, _window, _cx| {
            dialog
                .title("Configure projects")
                .child(
                    v_form().child(
                        field()
                            .label("Fill projects ids separated by comma (eg. `1,42,756`)")
                            .child(Input::new(&project_ids)),
                    ),
                )
                .footer({
                    let view = view.clone();
                    let project_ids = project_ids.clone();

                    move |_, _, _, _cx| {
                        vec![
                            Button::new("confirm").primary().label("Confirm").on_click({
                                let view = view.clone();
                                let project_ids = project_ids.clone();

                                move |_, window, cx| {
                                    view.update(cx, |view, cx| {
                                        let mut config = AppState::global(cx).config().clone();

                                        if let Some(gitlab) = config
                                            .gitlabs
                                            .iter_mut()
                                            .find(|gitlab| gitlab.host == view.config.host)
                                        {
                                            gitlab.project_ids = project_ids
                                                .read(cx)
                                                .value()
                                                .split(",")
                                                .into_iter()
                                                .filter_map(|raw| raw.parse::<u64>().ok())
                                                .collect::<Vec<u64>>();
                                            let gitlab_ = gitlab.clone();

                                            if let Err(error) =
                                                AppState::global_mut(cx).replace_config(config)
                                            {
                                                window.push_notification(
                                                    (
                                                        NotificationType::Error,
                                                        SharedString::new(format!(
                                                            "Error during project ids list save: {}",
                                                            error.to_string()
                                                        )),
                                                    ),
                                                    cx,
                                                );
                                            } else {
                                                window.push_notification(
                                                    SharedString::new("Project ids list saved"),
                                                    cx,
                                                );

                                                view.inner = cx.new(|_cx| LoadState::Loading);
                                                Self::init(window, cx, gitlab_);
                                            }
                                        }
                                    });

                                    window.close_dialog(cx);
                                }
                            }),
                            Button::new("cancel")
                                .label("Cancel")
                                .on_click(move |_, window, cx| {
                                    window.close_dialog(cx);
                                }),
                        ]
                    }
                })
        })
    }
}

impl Render for Projects {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let host = self.config.host.clone();

        div()
            .child(
                h_flex()
                    .child(Icon::new(IconName::ArrowRight))
                    .child(Label::new(format!("{} projects", host)).text_xl())
                    .child(" ".to_string())
                    .child(
                        Button::new("configure-projects")
                            .icon(IconName::Settings)
                            .link()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.show_project_ids_dialog(window, cx)
                            })),
                    ),
            )
            .child(self.inner.clone())
    }
}

pub struct Project(Entity<LoadState<ProjectInner, WithButtonModalError<GitlabError>>>);

impl Project {
    fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        gitlab: AsyncGitlab,
        project_id: u64,
    ) -> Self {
        let gitlab_ = gitlab.clone();
        cx.spawn_in(window, async move |project, cx| {
            match get_project(gitlab_.clone(), project_id).await {
                Ok(project_) => {
                    let _ = project.update_in(cx, |project, window, cx| {
                        let name = project_.name.into();
                        let pipelines =
                            cx.new(|cx| Pipelines::new(window, cx, gitlab_.clone(), project_id));
                        let merge_requests = cx
                            .new(|cx| MergeRequests::new(window, cx, gitlab_.clone(), project_id));

                        project.0 = cx.new(|cx| {
                            LoadState::Ready(cx.new(|_cx| ProjectInner {
                                name,
                                pipelines,
                                merge_requests,
                            }))
                        });
                    });
                }
                Err(error) => {
                    let _ = project.update_in(cx, |project, _window, cx| {
                        project.0 = cx.new(|cx| {
                            with_button_error!(
                                cx,
                                "Load error".into(),
                                format!("Error during load project {}", project_id),
                                error.into()
                            )
                        });
                    });
                }
            };
        })
        .detach();

        let gitlab_ = gitlab.clone();
        let interval = AppState::global(cx).config().refresh_every.duration();
        cx.spawn_in(window, async move |project, cx| {
            Timer::after(interval).await;
            let _ = project.update_in(cx, |project, window, cx| {
                *project = Self::new(window, cx, gitlab_.clone(), project_id);
                cx.notify();
            });
        })
        .detach();

        Self(cx.new(|_cx| LoadState::Loading))
    }
}

impl Render for Project {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child(self.0.clone())
    }
}

pub struct ProjectInner {
    name: SharedString,
    pipelines: Entity<Pipelines>,
    merge_requests: Entity<MergeRequests>,
}

impl Render for ProjectInner {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .child(self.name.clone())
            .child(" | ".to_string())
            .child(self.pipelines.clone())
            .child(" | ".to_string())
            .child(self.merge_requests.clone())
    }
}

async fn get_project(
    gitlab: AsyncGitlab,
    project_id: u64,
) -> Result<crate::state::gitlab::project::Project, GitlabError> {
    Ok(gitlab::api::projects::Project::builder()
        .project(project_id)
        .build()?
        .query_async(&gitlab)
        .await?)
}
