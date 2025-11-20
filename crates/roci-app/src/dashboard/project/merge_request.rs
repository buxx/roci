use gitlab::api::{self, AsyncQuery};
use gitlab::AsyncGitlab;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, ActiveTheme, Icon, IconName};
use roci_app_components::error::WithButtonModalError;
use roci_app_components::{with_button_error, LoadState};
use tracing_unwrap::ResultExt;

use crate::config::merge_request::ShowMergeRequest;
use crate::dashboard::error::GitlabError;
use crate::state::gitlab::project::merge_request::MergeRequestContainer;
use crate::state::gitlab::project::merge_request::{MergeRequestState, MERGE_STATUS_MERGEABLE};
use crate::state::gitlab::project::pipeline::Pipeline;
use crate::state::gitlab::user::User;
use crate::state::AppState;

pub struct MergeRequests(Entity<LoadState<MergeRequestsInner, WithButtonModalError<GitlabError>>>);

impl MergeRequests {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        gitlab: AsyncGitlab,
        project_id: u64,
    ) -> Self {
        let show = AppState::global(cx).config().show_merge_request.clone();

        cx.spawn_in(
            window,
            async move |merge_requests, cx| match get_merge_requests(gitlab, project_id, show).await
            {
                Ok(merge_requests_) => {
                    let _ = merge_requests.update_in(cx, |merge_requests, _window, cx| {
                        let merge_requests_ =
                            LoadState::Ready(cx.new(|_cx| MergeRequestsInner(merge_requests_)));
                        let merge_requests_ = cx.new(|_cx| merge_requests_);
                        merge_requests.0 = merge_requests_;
                    });
                }
                Err(error) => {
                    let _ = merge_requests.update_in(cx, |project, _window, cx| {
                        project.0 = cx.new(|cx| {
                            with_button_error!(
                                cx,
                                "Load error".into(),
                                format!("Error during load merge requests"),
                                error.into()
                            )
                        });
                    });
                }
            },
        )
        .detach();

        Self(cx.new(|_cx| LoadState::Loading))
    }
}

impl Render for MergeRequests {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().child(self.0.clone())
    }
}

pub struct MergeRequestsInner(Vec<MergeRequestContainer>);

impl Render for MergeRequestsInner {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .id("merge_requests")
            .children(self.0.iter().filter_map(|merge_request| {
                let forced_color = merge_request
                    .last_pipeline
                    .as_ref()
                    .filter(|p| p.status.is_error())
                    .map(|_| cx.theme().red);
                let icon = match merge_request.state {
                    MergeRequestState::Opened => {
                        if &merge_request.detailed_merge_status == MERGE_STATUS_MERGEABLE {
                            Some((
                                IconName::LayoutDashboard,
                                forced_color.unwrap_or(cx.theme().green),
                            ))
                        } else {
                            Some((
                                IconName::LayoutDashboard,
                                forced_color.unwrap_or(cx.theme().cyan),
                            ))
                        }
                    }
                    _ => None,
                };
                let web_url = merge_request.web_url.clone();

                if let Some((icon, color)) = icon {
                    Some(
                        div().id(ElementId::Integer(merge_request.id)).child(
                            div().child(
                                Button::new("merge_request-icon")
                                    .icon(Icon::new(icon).text_color(color))
                                    .link()
                                    .tooltip(format!(
                                        "{} ({})",
                                        merge_request.title, merge_request.detailed_merge_status
                                    ))
                                    .on_click(move |_, _, _| {
                                        open::that(web_url.clone()).unwrap_or_log();
                                    }),
                            ),
                        ),
                    )
                } else {
                    None
                }
            }))
    }
}

async fn get_merge_requests(
    gitlab: AsyncGitlab,
    project_id: u64,
    show: ShowMergeRequest,
) -> Result<Vec<MergeRequestContainer>, GitlabError> {
    let user: User = api::users::CurrentUser::builder()
        .build()?
        .query_async(&gitlab)
        .await?;

    let mut builder = gitlab::api::projects::merge_requests::MergeRequests::builder();
    let mut endpoint = builder.project(project_id);
    if matches!(show, ShowMergeRequest::OnlyMine) {
        endpoint = endpoint.author(user.id);
    }

    let mut merge_requests: Vec<crate::state::gitlab::project::merge_request::MergeRequest> =
        gitlab::api::paged(endpoint.build()?, api::Pagination::Limit(25))
            .query_async(&gitlab)
            .await?;
    merge_requests.reverse();

    let mut merge_requests_ = vec![];
    for merge_request in merge_requests.into_iter() {
        let last_pipeline = get_last_pipeline(&gitlab, project_id, merge_request.iid).await?;
        merge_requests_.push(MergeRequestContainer::new(merge_request, last_pipeline));
    }

    Ok(merge_requests_)
}

async fn get_last_pipeline(
    gitlab: &AsyncGitlab,
    project_id: u64,
    merge_request_iid: u64,
) -> Result<Option<Pipeline>, GitlabError> {
    let endpoint =
        gitlab::api::projects::merge_requests::pipelines::MergeRequestPipelines::builder()
            .project(project_id)
            .merge_request(merge_request_iid)
            .build()?;
    let pipelines: Vec<Pipeline> = gitlab::api::paged(endpoint, api::Pagination::Limit(1))
        .query_async(gitlab)
        .await?;

    Ok(pipelines.first().cloned())
}
