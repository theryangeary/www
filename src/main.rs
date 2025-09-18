use std::str::FromStr;

use axum::extract::Path;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{Router, routing::get};
use maud::DOCTYPE;
use maud::{Markup, html};
use rust_embed::Embed;
use strum::{EnumIter, EnumString, IntoEnumIterator};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct Link {
    href: &'static str,
    title: &'static str,
    target: Option<&'static str>,
}

const HOMEPAGE_BUTTONS: [Link; 4] = [
    Link {
        href: "/projects",
        title: "Projects",
        target: None,
    },
    Link {
        href: "/posts",
        title: "Posts",
        target: None,
    },
    Link {
        href: "https://github.com/theryangeary",
        title: "GitHub",
        target: Some("_blank"),
    },
    Link {
        href: "https://www.linkedin.com/in/theryangeary/",
        title: "LinkedIn",
        target: Some("_blank"),
    },
];

fn head() -> Markup {
    html! {
        head {
            (DOCTYPE)
            meta charset="UTF-8" {};
            meta name="viewport" content="width=device-width, initial-scale=1.0" {};
            link rel="stylesheet" href="/static/output.css";
            script src="/static/htmx.min.js" {};
        }
    }
}

fn navbar() -> Markup {
    html! {
        nav class="m-4" {
            div class="flex justify-center divide-x-1 divide-purple-300 divide-solid " {
                a class="text-xl text-purple-900/50 dark:text-violet-300 hover:underline decoration-3 pr-3 pl-3 md:text-3xl flex-1 " href="/" { "Ryan Geary" }
                a class="text-xl text-purple-900/50 dark:text-violet-300 hover:underline decoration-3 pr-3 pl-3 flex-none " href="/projects" { "Projects" }
                a class="text-xl text-purple-900/50 dark:text-violet-300 hover:underline decoration-3 pr-3 pl-3 flex-none " href="/posts" { "Posts" }
            }
        }
    }
}

struct Project {
    id: String,
    title: String,
    description: String,
    tech_stack: Vec<String>,
    github_url: Option<String>,
    try_it_url: Option<String>,
    category: ProjectCategory,
}

#[derive(EnumIter, EnumString, PartialEq, Eq, Hash, strum::Display)]
#[strum(serialize_all = "snake_case")]
enum ProjectCategory {
    Production,
    Toy,
}

impl Default for ProjectCategory {
    fn default() -> Self {
        ProjectCategory::Production
    }
}

impl ProjectCategory {
    fn title(&self) -> &str {
        match self {
            ProjectCategory::Production => "Production Projects",
            ProjectCategory::Toy => "Toy Projects",
        }
    }

    fn current_projects(&self) -> Vec<Project> {
        match self {
            ProjectCategory::Production => vec![Project {
                id: "choose".to_string(),
                title: "choose".to_string(),
                description: "A human-friendly and fast alternative to cut (and sometimes awk).".to_string(),
                tech_stack: vec!["Rust".to_string()],
                github_url: Some("https://github.com/theryangeary/choose".to_string()),
                try_it_url: Some("https://github.com/theryangeary/choose?tab=readme-ov-file#installing-from-source".to_string()),
                category: ProjectCategory::Production,
            }],
            ProjectCategory::Toy => vec![],
        }
    }
}

async fn project_tabs(Path(tab): Path<String>) -> Markup {
    project_tabs_markup(ProjectCategory::from_str(&tab).unwrap_or(Default::default()))
}

fn id(s: &str) -> String {
    format!("#{}", s)
}

fn project_tabs_markup(active: ProjectCategory) -> Markup {
    let all_tab_styles = "px-6 py-3 border-1 border-purple-300 font-medium transition-colors ";
    let inactive_tab_styles = "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600";
    let active_tab_styles = "bg-purple-900/75 text-amber-200 ";
    let target_id = "project_tabs";
    html! {
        div id=(target_id) class="space-y-6 divide-solid divide-purple-300 divide-y-1" {
            div class="flex justify-center" {
                @for tab in ProjectCategory::iter() {
                    @let classes = if tab == active {
                        all_tab_styles.to_owned() + active_tab_styles
                    } else {
                        all_tab_styles.to_owned() + inactive_tab_styles
                    };

                    button class=(classes) hx-get=(format!("/projects/{}", tab.to_string())) hx-target=(id(target_id)){
                        (tab.title())
                    }
                }
            }

            (project_grid(active.current_projects()))
        }
    }
}

fn project_grid(projects: Vec<Project>) -> Markup {
    html! {
        div class="mt-8" {
            div class="space-y-8" {
                section {
                    div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3" {
                        @for project in projects {
                            (project_card(project))
                        }
                    }
                }
            }
        }
    }
}

fn project_card(project: Project) -> Markup {
    html! {
        div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow" {
            header class="mb-4" {
                div class="flex items-start justify-between mb-2" {
                    h3 class="text-xl font-semibold text-primary" {
                        (project.title)
                    }
                }
            }

            p class="text-gray-700 dark:text-gray-300 mb-4" {
                (project.description)
            }

            div className="mb-4" {
                h4 className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-2" {
                "Tech Stack: "
                }
                div className="flex flex-wrap gap-2" {
                    @for tech in project.tech_stack {
                        span className="bg-violet-100 dark:bg-violet-900/30 text-violet-800 dark:text-violet-300 px-2 py-1 rounded text-xs" {
                            (tech)
                        }
                    }
                }
            }
        }
    }
}

async fn projects() -> Markup {
    html! {
        (head())
        body {
            div {
                div class="container mx-auto px-4 py-4" {
                    (navbar())
                    div class="mt-8" {
                        (project_tabs_markup(Default::default()))
                    }
                }
            }
        }
    }
}

async fn index() -> Markup {
    html! {
        (head())
        body {
            div class="container mx-auto px-4 flex h-screen" {
                div class="m-auto" {
                    h1 class="font-bold underline p-4 text-primary" {
                        span class="text-4xl md:text-6xl lg:text-8xl" {
                            "Ryan Geary"
                        }
                    }

                    p class="flex justify-center text-secondary" {
                        "Software Developer @Lyft"
                    }
                    p class="flex justify-center text-secondary" {
                        "FOSS Developer"
                    }

                    div class="flex justify-center" {
                        img src="/static/headshot.jpg" alt="Ryan Geary's headshot" class="w-3xs rounded-full p-10" {};
                    }

                    div class="grid grid-cols-2 grid-rows-2 gap-4" {
                        @for b in HOMEPAGE_BUTTONS {
                            a href=(b.href) target=(b.target.unwrap_or("_self")) class="
                                flex
                                justify-center
                                text-amber-200
                                hover:text-amber-50
                                bg-purple-900/75
                                p-4
                                border-4
                                border-purple-300
                                rounded-none
                                active:translate-1
                                active:shadow-none
                                shadow-[8px_8px_0_rgba(0,0,0,0.25)]
                                "
                            {
                                (b.title)
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Embed)]
#[folder = "$OUT_DIR/static"]
struct Assets;

async fn static_file(Path(path): Path<String>) -> impl IntoResponse {
    tracing::info!("static");
    match Assets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            println!(
                "{} not found in {:?}",
                path,
                Assets::iter().collect::<Vec<_>>()
            );
            return not_found().await;
        }
    }
}

async fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "404").into_response()
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "grocery_list_backend=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Build our application
    let app = Router::new()
        .route("/static/{file}", get(static_file))
        .route("/", get(index))
        .route("/projects", get(projects))
        .route("/projects/{tab}", get(project_tabs))
        .layer(TraceLayer::new_for_http());

    // Run it on localhost:3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
