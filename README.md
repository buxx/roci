# 🚀 Roci (Rocinante)

## Install

### Install from sources

    cargo install --path crates/roci-app

Then run with `roci`.

## TODO

- [x] Multi gitlab access token for permit gitlab ce projects
- [x] re-implement auto refresh of pipelines and merge_requests
- [x] Ui to modify config
- [ ] Ui to modify displayed number of latest pipelines
- [ ] Ui to modify merge requests "only mine" or "all"
- [x] Title bar with menu (config, refresh, about, etc)
- [ ] Resume of lasts nightly evals
- [x] Resume of assigned issues
- [x] bug: need restart after edit gitlab instance config
- [ ] When applied "refresh_every" config, hot apply on components which use it
