// @TODO: uncomment once https://github.com/rust-lang/rust/issues/54726 stable
//#![rustfmt::skip::macros(class)]

#![allow(
    clippy::used_underscore_binding,
    clippy::non_ascii_literal,
    clippy::enum_glob_use,
    clippy::must_use_candidate
)]

mod generated;
mod guide;
mod page;

use generated::css_classes::C;
use guide::Guide;
use page::partial::blender;
use seed::{
    app, div, document, nodes, prelude::*, wasm_bindgen, web_sys, window, C,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use Visibility::*;

const SEED_VERSION: &str = "0.6.0";
const SEED_VERSION_DATE: &str = "Feb 1, 2020";
const TITLE_SUFFIX: &str = "Seed";
const STORAGE_KEY: &str = "seed";
const USER_AGENT_FOR_PRERENDERING: &str = "ReactSnap";

// ------ ------
//    Common
// ------ ------

// ------ Visibility  ------

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Visibility {
    Visible,
    Hidden,
}

impl Visibility {
    pub fn toggle(&mut self) {
        *self = match self {
            Visible => Hidden,
            Hidden => Visible,
        }
    }
}

// ------ Config ------

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    mode: Mode,
}

// ------ ------
//     Init
// ------ ------

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders
        .send_msg(Msg::RouteChanged(url.clone().into()))
        .send_msg(Msg::UpdatePageTitle)
        .subscribe(|app::subs::UrlChanged(url)| -> Msg {
            Msg::RouteChanged(url.into())
        });

    let guides = guide::guides();
    Model {
        page: Page::from_route_and_replace_history(&url.into(), &guides),
        guide_list_visibility: Hidden,
        menu_visibility: Hidden,
        in_prerendering: is_in_prerendering(),
        guides,
        search_query: "".to_string(),
        matched_guides: vec![],
        mode: load_config().mode,
    }
}

fn load_config() -> Config {
    LocalStorage::get(STORAGE_KEY).unwrap_or_default()
}

fn is_in_prerendering() -> bool {
    let user_agent = window().navigator().user_agent().expect("get user agent");
    user_agent == USER_AGENT_FOR_PRERENDERING
}

// ------ ------
//     Model
// ------ ------

pub struct Model {
    pub page: Page,
    pub guide_list_visibility: Visibility,
    pub menu_visibility: Visibility,
    pub in_prerendering: bool,
    pub guides: Vec<Guide>,
    pub search_query: String,
    pub matched_guides: Vec<Guide>,
    pub mode: Mode,
}

// ------ Mode  ------

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Light,
    Dark,
}

impl Mode {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::Light
    }
}

// ------ Page ------

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Guide {
        guide: Guide,
        show_intro: bool,
    },
    NotFound,
}

impl Page {
    pub fn to_href(self) -> String {
        match self {
            Self::Guide {
                guide,
                ..
            } => format!("/guide/{}", guide.slug),
            Self::NotFound => "/404".into(),
        }
    }

    pub fn from_route_and_replace_history(
        route: &Route,
        guides: &[Guide],
    ) -> Self {
        match route {
            Route::Root => match guides.first() {
                Some(guide) => Self::Guide {
                    guide: *guide,
                    show_intro: true,
                },
                None => Self::NotFound,
            },
            Route::Guide(slug) => {
                match guides.iter().find(|guide| guide.slug == slug) {
                    Some(guide) => Self::Guide {
                        guide: *guide,
                        show_intro: false,
                    },
                    None => Self::NotFound,
                }
            },
            Route::Unknown => Self::NotFound,
        }
    }
}

// ------ ------
//    Routes
// ------ ------

pub fn routes(url: Url) -> Option<Msg> {
    Some(Msg::RouteChanged(url.into()))
}

#[derive(Clone)]
pub enum Route {
    Root,
    Guide(String),
    Unknown,
}

impl From<Url> for Route {
    fn from(mut url: Url) -> Self {
        match url.remaining_path_parts().as_slice() {
            [] => Self::Root,
            ["guide", page] => Self::Guide((*page).to_string()),
            _ => Self::Unknown,
        }
    }
}

impl Route {
    pub fn path(&self) -> Vec<&str> {
        match self {
            Self::Root => vec![],
            Self::Guide(slug) => vec!["guide", slug.as_str()],
            Self::Unknown => vec!["404"],
        }
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "/{}", self.path().join("/"))
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
pub enum Msg {
    RouteChanged(Route),
    UpdatePageTitle,
    ScrollToTop,
    ToggleGuideList,
    HideGuideList,
    ToggleMenu,
    HideMenu,
    SearchQueryChanged(String),
    ToggleMode,
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::RouteChanged(route) => {
            model.page =
                Page::from_route_and_replace_history(&route, &model.guides);
            orders.send_msg(Msg::ScrollToTop);
            orders.send_msg(Msg::UpdatePageTitle);
        },
        Msg::UpdatePageTitle => {
            let title = match model.page {
                Page::Guide {
                    guide,
                    ..
                } => format!("{} - {}", guide.menu_title, TITLE_SUFFIX),
                Page::NotFound => format!("404 - {}", TITLE_SUFFIX),
            };
            document().set_title(&title);
        },
        Msg::ScrollToTop => window().scroll_to_with_scroll_to_options(
            web_sys::ScrollToOptions::new().top(0.),
        ),
        Msg::ToggleGuideList => model.guide_list_visibility.toggle(),
        Msg::HideGuideList => {
            model.guide_list_visibility = Hidden;
        },
        Msg::ToggleMenu => model.menu_visibility.toggle(),
        Msg::HideMenu => {
            model.menu_visibility = Hidden;
        },
        Msg::SearchQueryChanged(query) => {
            model.matched_guides = search(&model.guides, &query);
            model.search_query = query;
        },
        Msg::ToggleMode => {
            model.mode.toggle();

            let config = Config {
                mode: model.mode,
            };
            LocalStorage::insert(STORAGE_KEY, &config)
                .expect("insert to local storage");
        },
    }
}

fn search(guides: &[Guide], query: &str) -> Vec<Guide> {
    if query.is_empty() {
        return Vec::new();
    }
    let query = query.to_lowercase();
    guides
        .iter()
        .filter_map(|guide| {
            if guide.lowercase_text.contains(&query) {
                Some(*guide)
            } else {
                None
            }
        })
        .collect()
}

// ------ ------
//     View
// ------ ------

pub fn view(model: &Model) -> impl IntoNodes<Msg> {
    nodes![
        div![
            C![C.min_h_screen, C.bg_white,],
            match model.page {
                Page::Guide {
                    guide,
                    show_intro,
                } => page::guide::view(&guide, model, show_intro),
                Page::NotFound => page::not_found::view(),
            },
            page::partial::header::view(model),
        ],
        blender::view_for_content(model.mode),
    ]
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
