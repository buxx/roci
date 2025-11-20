use gpui::*;

use crate::{
    config::{Config, ConfigError},
    state::gitlab::Gitlabs,
};

pub mod gitlab;

pub struct AppState {
    config: Config,
    gitlabs: Gitlabs,
}

impl AppState {
    pub fn init(cx: &mut App, config: Config) {
        let state = Self {
            config,
            gitlabs: Gitlabs::default(),
        };

        cx.set_global::<AppState>(state);
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn gitlabs(&self) -> Gitlabs {
        self.gitlabs.clone()
    }

    pub fn replace_config(&mut self, new: Config) -> std::result::Result<(), ConfigError> {
        new.persist()?;
        self.config = new;

        Ok(())
    }
}

impl Global for AppState {}
