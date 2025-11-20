use gitlab::api::{self, projects, AsyncQuery};
use gitlab::AsyncGitlab;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::label::Label;
use gpui_component::{h_flex, ActiveTheme, Icon, IconName};
use roci_app_components::error::WithButtonModalError;
use roci_app_components::{with_button_error, LoadState};
use tracing_unwrap::ResultExt;

use crate::dashboard::error::GitlabError;
use crate::state::gitlab::project::pipeline::{Pipeline, PipelineStatus};

pub const PIPELINES_COUNT: usize = 10;

pub struct Pipelines(Entity<LoadState<PipelinesInner, WithButtonModalError<GitlabError>>>);

impl Pipelines {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        gitlab: AsyncGitlab,
        project_id: u64,
    ) -> Self {
        cx.spawn_in(window, async move |pipelines, cx| {
            match get_pipelines(gitlab, project_id).await {
                Ok(pipelines_) => {
                    let _ = pipelines.update_in(cx, |pipelines, _window, cx| {
                        let pipelines_ = LoadState::Ready(cx.new(|_cx| PipelinesInner(pipelines_)));
                        let pipelines_ = cx.new(|_cx| pipelines_);
                        pipelines.0 = pipelines_;
                    });
                }
                Err(error) => {
                    let _ = pipelines.update_in(cx, |project, _window, cx| {
                        project.0 = cx.new(|cx| {
                            with_button_error!(
                                cx,
                                "Load error".into(),
                                format!("Error during load pipelines"),
                                error.into()
                            )
                        });
                    });
                }
            }
        })
        .detach();

        Self(cx.new(|_cx| LoadState::Loading))
    }
}

impl Render for Pipelines {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child(self.0.clone())
    }
}

pub struct PipelinesInner(Vec<Pipeline>);

impl Render for PipelinesInner {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .when(self.0.is_empty(), |_cx| div().child(Label::new("n/a")))
            .id("pipelines")
            .children(self.0.iter().map(|pipeline| {
                let (icon, color): (IconName, Hsla) = match pipeline.status {
                    PipelineStatus::Running
                    | PipelineStatus::Pending
                    | PipelineStatus::Created
                    | PipelineStatus::Preparing
                    | PipelineStatus::Manual
                    | PipelineStatus::Scheduled
                    | PipelineStatus::WaitingForResource => (IconName::Ellipsis, cx.theme().cyan),
                    PipelineStatus::Success => (IconName::CircleCheck, cx.theme().green),
                    PipelineStatus::Failed | PipelineStatus::Canceled | PipelineStatus::Skipped => {
                        (IconName::CircleX, cx.theme().red)
                    }
                };
                let web_url = pipeline.web_url.clone();

                div().id(ElementId::Integer(pipeline.id)).child(
                    div().child(
                        Button::new("pipeline-icon")
                            .icon(Icon::new(icon).text_color(color))
                            .link()
                            .tooltip(format!(
                                "{} ({}), {}",
                                pipeline.id, pipeline.iid, pipeline.status
                            ))
                            .on_click(move |_, _, _| {
                                open::that(web_url.clone()).unwrap_or_log();
                            }),
                    ),
                )
            }))
    }
}

async fn get_pipelines(gitlab: AsyncGitlab, project_id: u64) -> Result<Vec<Pipeline>, GitlabError> {
    let project_: crate::state::gitlab::project::Project = projects::Project::builder()
        .project(project_id)
        .build()?
        .query_async(&gitlab)
        .await?;

    let pipelines_endpoint = projects::pipelines::Pipelines::builder()
        .project(project_id)
        .source(projects::pipelines::PipelineSource::Push)
        .ref_(&project_.default_branch)
        .build()?;
    let mut pipelines: Vec<crate::state::gitlab::project::pipeline::Pipeline> =
        api::paged(pipelines_endpoint, api::Pagination::Limit(PIPELINES_COUNT))
            .query_async(&gitlab)
            .await?;
    pipelines.reverse();

    Ok(pipelines)
}
