//! Static site generator for www.ryangeary.dev
//!
//! Compiles to a single executable that pre-renders all pages into the `output/` directory.
//!
//! General structure:
//! 1. const definitions
//! 2. domain model definitions
//! 3. `Markup` generating functions (components)
//! 4. page renderers (return `String`)
//! 5. static site builder (`main` + helpers)
//! 6. utility functions

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::NaiveDate;
use lazy_static::lazy_static;
use maud::{DOCTYPE, PreEscaped};
use maud::{Markup, html};
use pulldown_cmark::{Options, Parser, html as cmark_html};
use strum::{EnumIter, EnumString, IntoEnumIterator};

// static resources

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

lazy_static! {
    static ref POSTS: Vec<Post> = vec![
        Post {
            title: "Working with Multiple Files in Vim",
            date: NaiveDate::from_ymd_opt(2019, 01, 02).unwrap(),
            tags: vec!["vim"],
            excerpt: "I was recently asked by another member of udellug about my \"top three tips for working with multiple files/large projects in vim\". Three quickly turned into six.",
            content: include_str!("../posts/2019-vim-tips.md"),
            id: "2019-vim-tips",
        },
        Post {
            title: "Making FZF Completion Automatic in ZSH",
            date: NaiveDate::from_ymd_opt(2025, 07, 25).unwrap(),
            tags: vec!["zsh", "fzf"],
            excerpt: "I'm forcing myself to use FZF by triggering it on spacebar with commands that can benefit from it.",
            content: include_str!("../posts/2025-zsh-zle-fzf.md"),
            id: "2025-zsh-zle-fzf",
        },
        Post {
            title: "Why Oh Why Am I Starting a Homelab",
            date: NaiveDate::from_ymd_opt(2025, 08, 10).unwrap(),
            tags: vec!["homelab", "fly.io"],
            excerpt: "After evaluating a handful of options for free-tier and cheap cloud hosting, I'm foraying into the wacky world of self-hosting.",
            content: include_str!("../posts/2025-homelab-1.md"),
            id: "2025-homelab-1",
        },
        Post {
            title: "Why I'm Making My Own Grocery List",
            date: NaiveDate::from_ymd_opt(2025, 09, 10).unwrap(),
            tags: vec!["rust", "react", "gl", "homelab"],
            excerpt: "Typing is lame. Pressing buttons is cool 😎. I buy the same things from the grocery store ALL the time. You probably do too.",
            content: include_str!("../posts/2025-why-gl.md"),
            id: "2025-why-gl",
        },
        Post {
            title: "Grocery List Demo is Now Live",
            date: NaiveDate::from_ymd_opt(2025, 10, 14).unwrap(),
            tags: vec!["gl", "homelab"],
            excerpt: "Take it for a spin!",
            content: include_str!("../posts/2025-gl-demo.md"),
            id: "2025-gl-demo",
        },
        Post {
            title: "I Caused a Security Vulnerability Today",
            date: NaiveDate::from_ymd_opt(2025, 11, 05).unwrap(),
            tags: vec![tag::DOCKER, tag::HOMELAB, tag::MTA_DISPLAY, tag::CADDY],
            excerpt: "It doesn't help that I have absolutely no automated observability yet.",
            content: include_str!("../posts/2025-homelab-sec-vuln.md"),
            id: "2025-homelab-sec-vuln",
        },
        Post {
            title: "Experimenting with Local Inference",
            date: NaiveDate::from_ymd_opt(2026, 05, 26).unwrap(),
            tags: vec![tag::RUST, tag::MAUD, tag::LLMS],
            excerpt: "At what point do the models become smart and efficient enough that I can just run them on my commodity hardware rather than shelling out to someone else hosting it publicly?",
            content: include_str!("../posts/2026-local-inference.md"),
            id: "2026-local-inference",
        }

    ];

    static ref PROJECTS: Vec<Project> = vec![
        Project {
            id: "choose".to_string(),
            title: "choose".to_string(),
            description: "A human-friendly and fast alternative to cut (and sometimes awk).".to_string(),
            tech_stack: vec![tag::RUST],
            github_url: Some("https://github.com/theryangeary/choose".to_string()),
            try_it_url: Some("https://github.com/theryangeary/choose?tab=readme-ov-file#installing-from-source".to_string()),
            category: ProjectCategory::Production,
        },
        Project {
            id: "personal-website".to_string(),
            title: "Personal Website".to_string(),
            description: "This site! A static site generator built with Rust (maud) and Tailwind CSS. Compiles to a single binary.".to_string(),
            tech_stack: vec![tag::RUST, tag::TAILWIND, tag::MAUD],
            github_url: Some("https://github.com/theryangeary/www".to_string()),
            try_it_url: Some("https://www.ryangeary.dev".to_string()),
            category: ProjectCategory::Production,
        },
        Project {
            id: "homelab".to_string(),
            title: "Homelab".to_string(),
            description: "My personal infrastructure, hosted on a Raspberry Pi running docker swarm in my router closet.".to_string(),
            tech_stack: vec![tag::DOCKER, tag::CLOUDFLARE_TUNNELS, tag::CADDY],
            github_url: Some("https://github.com/theryangeary/homelab".to_string()),
            try_it_url: Some("https://www.ryangeary.dev".to_string()),
            category: ProjectCategory::Production,
        },
        Project {
            id: "fib-o1".to_string(),
            title: "fib-o1: Constant Time Fibonacci Sequence Values".to_string(),
            description: "Abusing the Rust build system to provide O(1) fib(n) at runtime.".to_string(),
            tech_stack: vec![tag::RUST, tag::BIGINT],
            github_url: Some("https://github.com/theryangeary/fib-o1".to_string()),
            try_it_url: Some("https://crates.io/crates/fib-o1".to_string()),
            category: ProjectCategory::Toy,
        },
        Project {
            id: "pathfinder".to_string(),
            title: "Pathfinder.prof".to_string(),
            description: "A daily word puzzle combining points-based tiles with grid-based word finding.".to_string(),
            tech_stack: vec![tag::FLY_IO, tag::CLOUDFLARE_PAGES, tag::POSTGRES, tag::RUST, tag::REACT, tag::TYPESCRIPT],
            github_url: Some("https://github.com/theryangeary/pathfinder".to_string()),
            try_it_url: Some("https://pathfinder.prof".to_string()),
            category: ProjectCategory::Production,
        },
        Project {
            id: "gl".to_string(),
            title: "gl".to_string(),
            description: "A personal-software grocery list featuring multi-player, autocomplete, and smart categorization".to_string(),
            tech_stack: vec![tag::HOMELAB, tag::SQLITE, tag::RUST, tag::REACT, tag::TYPESCRIPT],
            github_url: Some("https://github.com/theryangeary/gl".to_string()),
            try_it_url: Some("https://gldemo.ryangeary.dev".to_string()),
            category: ProjectCategory::Production,
        },
        Project {
            id: "ginh".to_string(),
            title: "Ginh Is Not a Histogram".to_string(),
            description: "A shell-based visual representation of a user's shell history.".to_string(),
            tech_stack: vec![tag::BASH, "That's it it's pure bash script"],
            github_url: Some("https://github.com/crclark96/ginh".to_string()),
            try_it_url: Some("https://github.com/crclark96/ginh?tab=readme-ov-file#installation".to_string()),
            category: ProjectCategory::Toy,
        },
        Project {
            id: "photo".to_string(),
            title: "Photography Gallery Website".to_string(),
            description: "Photo gallery website made with pure vanilla javascript components.".to_string(),
            tech_stack: vec![tag::BASH, tag::JAVASCRIPT, tag::EXIFTOOL],
            github_url: Some("https://github.com/theryangeary/photo".to_string()),
            try_it_url: Some("https://theryangeary.github.io/photo".to_string()),
            category: ProjectCategory::Production,
        },
        Project {
            id: "mta-display".to_string(),
            title: "MTA Subway Train Display".to_string(),
            description: "A simulation of MTA displays with user-specified messages, complete with guestbook.".to_string(),
            tech_stack: vec![tag::RUST, tag::HTMX, tag::TAILWIND, tag::MAUD, tag::AXUM, tag::SQLITE],
            github_url: Some("https://github.com/theryangeary/mta-display".to_string()),
            try_it_url: Some("https://mtadisplay.ryangeary.dev".to_string()),
            category: ProjectCategory::Production,
        }
    ];
}

// domain models

struct Link {
    href: &'static str,
    title: &'static str,
    target: Option<&'static str>,
}

struct Post {
    /// a unique id for this post; is also used in URLs, making it a very brief description of the post
    id: &'static str,
    /// title of post; displayed in page and html.head.title
    title: &'static str,
    /// data of publication; displayed in page
    date: chrono::NaiveDate,
    /// relevant subjects or technologies; displayed in page
    tags: Vec<tag::Tag>,
    /// excerpt from content; not displayed in post page, but displayed in previews to posts
    excerpt: &'static str,
    /// markdown content of document
    content: &'static str,
}

impl Post {
    fn content(&self) -> Markup {
        PreEscaped(markdown_to_html(self.content))
    }

    fn formatted_date(&self) -> String {
        self.date.to_string()
    }
}

mod tag {
    pub type Tag = &'static str;

    pub const RUST: Tag = "Rust";
    pub const AXUM: Tag = "Axum";
    pub const MAUD: Tag = "Maud";
    pub const HTMX: Tag = "htmx";
    pub const TAILWIND: Tag = "Tailwind CSS";
    pub const DOCKER: Tag = "Docker";
    pub const CLOUDFLARE_TUNNELS: Tag = "Cloudflare Tunnels";
    pub const CADDY: Tag = "Caddy";
    pub const BIGINT: Tag = "BigInt";
    pub const FLY_IO: Tag = "Fly.io";
    pub const CLOUDFLARE_PAGES: Tag = "Cloudflare Pages";
    pub const POSTGRES: Tag = "PostgreSQL";
    pub const REACT: Tag = "React";
    pub const TYPESCRIPT: Tag = "TypeScript";
    pub const HOMELAB: Tag = "Homelab";
    pub const SQLITE: Tag = "SQLite";
    pub const BASH: Tag = "Bash";
    pub const JAVASCRIPT: Tag = "JavaScript";
    pub const EXIFTOOL: Tag = "ExifTool";
    pub const MTA_DISPLAY: Tag = "MTA Display";
    pub const LLMS: Tag = "LLMs";
}

struct Project {
    #[allow(dead_code)]
    id: String,
    title: String,
    description: String,
    tech_stack: Vec<tag::Tag>,
    github_url: Option<String>,
    try_it_url: Option<String>,
    category: ProjectCategory,
}

#[derive(EnumIter, EnumString, PartialEq, Eq, Hash, strum::Display, Copy, Clone)]
#[strum(serialize_all = "snake_case")]
#[derive(Default)]
enum ProjectCategory {
    #[default]
    Production,
    Toy,
}

impl ProjectCategory {
    fn title(&self) -> &str {
        match self {
            ProjectCategory::Production => "Production Projects",
            ProjectCategory::Toy => "Toy Projects",
        }
    }

    fn current_projects(&self) -> impl Iterator<Item = &Project> {
        PROJECTS.iter().filter(|p| p.category == *self)
    }
}

// markup generation (components — return Markup)

fn head(title: &str) -> Markup {
    html! {
        head {
            (DOCTYPE)
            meta charset="UTF-8" {};
            meta name="viewport" content="width=device-width, initial-scale=1.0" {};
            link rel="stylesheet" href="/static/output.css";
            title { (title) }
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

fn project_tabs_markup(active: ProjectCategory) -> Markup {
    let all_tab_styles = "px-6 py-3 border-1 border-purple-300 font-medium transition-colors ";
    let inactive_tab_styles = "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600";
    let active_tab_styles = "bg-purple-900/75 text-amber-200 ";

    html! {
        div id="project_tabs" class="space-y-6 divide-solid divide-purple-300 divide-y-1" {
            div class="flex justify-center" {
                @for tab in ProjectCategory::iter() {
                    @let classes = if tab == active {
                        all_tab_styles.to_owned() + active_tab_styles
                    } else {
                        all_tab_styles.to_owned() + inactive_tab_styles
                    };
                    @let href = tab_href(tab, active);

                    @if let Some(h) = href {
                        a class=(classes) href=(h) {
                            (tab.title())
                        }
                    } @else {
                        span class=(classes) {
                            (tab.title())
                        }
                    }
                }
            }

            (project_grid_markup(active.current_projects()))
        }
    }
}

fn tab_href(tab: ProjectCategory, active: ProjectCategory) -> Option<&'static str> {
    if tab == active {
        None // active tab — render as <span>
    } else {
        match tab {
            ProjectCategory::Production => Some("../"),
            ProjectCategory::Toy => Some("toy/"),
        }
    }
}

fn project_grid_markup<'a>(projects: impl Iterator<Item = &'a Project>) -> Markup {
    html! {
        div class="mt-8" {
            div class="space-y-8" {
                section {
                    div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3" {
                        @for project in projects {
                            (project_card_markup(project))
                        }
                    }
                }
            }
        }
    }
}

fn project_card_markup(project: &Project) -> Markup {
    html! {
        div class="
            bg-white
            dark:bg-gray-800
            rounded-lg
            shadow-md
            p-6
            hover:shadow-lg
            transition-shadow
            flex
            flex-col
            justify-between
            h-full"
        {
            div {
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

                div class="mb-4" {
                    h4 class="text-sm font-medium text-gray-900 dark:text-gray-100 mb-2" {
                    "Tech Stack: "
                    }
                    div class="flex flex-wrap gap-2" {
                        @for tech in &project.tech_stack {
                            span class="bg-violet-100 dark:bg-violet-900/30 text-violet-800 dark:text-violet-300 px-2 py-1 rounded text-xs" {
                                (tech)
                            }
                        }
                    }
                }
            }

            div class="flex gap-3" {
                @if let Some(github_url) = &project.github_url {
                    a href=(github_url) target="_blank" rel="noopener noreferrer" class="text-violet-600 dark:text-violet-400 hover:underline font-medium text-sm" {
                        "View Code →"
                    }
                }

                @if let Some(try_it_url) = &project.try_it_url {
                    a href=(try_it_url) target="_blank" rel="noopener noreferrer" class="text-violet-600 dark:text-violet-400 hover:underline font-medium text-sm" {
                        "Try It →"
                    }
                }
            }
        }
    }
}

fn post_article_markup(p: &Post) -> Markup {
    html! {
        article class="max-w-4xl mx-auto px-4 py-8" {
            header class="mb-8" {
                h1 class="text-3xl md:text-4xl font-bold text-violet-900/50 dark:text-violet-300 mb-4" {
                    (p.title)
                }
                div class="flex flex-wrap items-center gap-4 text-sm text-gray-600 dark:text-gray-400" {
                    time dateTime=(p.date) {
                        (p.formatted_date())
                    }
                    div class="flex gap-2" {
                        @for tag in &p.tags {
                            span class="bg-violet-100 dark:bg-violet-900/30 text-violet-800 dark:text-violet-300 px-2 py-1 rounded text-xs" {
                                (format!("#{tag}"))
                            }
                        }
                    }
                }
            }

            div class="prose prose-lg prose-footnotes:flex prose-footnotes:items-start prose-footnotes:inline-flex dark:prose-invert max-w-none" {
                (p.content())
            }
        }
    }
}

fn post_linked_list_markup(post: &Post) -> Markup {
    let previous_sequence_number = POSTS
        .iter()
        .enumerate()
        .find(|(_, p)| p.id == post.id)
        .map(|i| i.0)
        .map(|d| d.checked_sub(1))
        .flatten();

    let next_sequence_number = POSTS
        .iter()
        .enumerate()
        .find(|(_, p)| p.id == post.id)
        .map(|i| i.0)
        .filter(|d| *d < POSTS.len() - 1)
        .map(|d| d.checked_add(1))
        .flatten();

    let prev_post_opt = previous_sequence_number.map(|i| &POSTS[i]);
    let next_post_opt = next_sequence_number.map(|i| &POSTS[i]);

    let card_classes = "flex-none
    max-w-2/5 overflow-hidden
    p-4 py-3.5
    bg-black/5 hover:bg-black/10
    dark:bg-white/5 hover:dark:bg-white/10
    text-violet-600 dark:text-violet-400
    border border-2 rounded-md
    border-violet-300 dark:border-violet-700";

    let card_direction_classes = "text-sm text-gray-700 dark:text-gray-300";
    let text_right = " text-right";

    let card_title_classes = "text-sm md:text-md lg:text-lg ";

    let arrow_classes = "text-sm text-xl";

    html! {
        div class="flex max-w-full pb-4" {
            @if let Some(prev_index) = previous_sequence_number && let Some(prev_post) = prev_post_opt {
                a href=(&format!("/posts/{}/{}", prev_index, prev_post.id)) class=(card_classes) {
                    div class="flex pr-2 items-center h-full" {
                        div class="flex-1 hidden md:flex items-center pr-4"{
                            p class=(arrow_classes) { "←" }
                        }
                        div class="flex-grow" {
                            p class=(card_direction_classes) { span class="md:hidden" { "← "} "Previous Post"  }
                            p class=(card_title_classes){ (prev_post.title) }
                        }
                    }
                }
            }

            div class="flex-grow"{}

            @if let Some(next_index) = next_sequence_number && let Some(next_post) = next_post_opt {
                a href=(&format!("/posts/{}/{}", next_index, next_post.id)) class=(card_classes) {
                    div class="flex pl-2 items-center h-full" {
                        div class="flex-grow" {
                            p class=(card_direction_classes.to_owned()+text_right) { "Next Post" span class="md:hidden" { " →"} }
                            p class=(card_title_classes.to_owned()+text_right) { (next_post.title) }
                        }
                        div class="flex-1 hidden md:flex items-center pl-4 "{
                            p class=(arrow_classes) { "→" }
                        }
                    }

                }
            }
        }
    }
}

fn posts_list_markup(ps: &[Post]) -> Markup {
    html! {
        div class="grid gap-6 md:gap-8" {
            @for (index, p) in ps.iter().enumerate().rev() {
                (post_card_markup(index, p))
            }
        }
    }
}

fn post_card_markup(index: usize, p: &Post) -> Markup {
    html! {
        article class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 hover:shadow-lg transition-shadow" {
            header class="mb-4" {
                h2 class="text-xl font-semibold text-violet-900/50 dark:text-violet-300 mb-2" {
                    a href=(&format!("/posts/{}/{}", index, p.id)) class="hover:underline" {
                        (p.title)
                    }
                }
                div class="flex flex-wrap items-center gap-4 text-sm text-gray-600 dark:text-gray-400" {
                    time dateTime=(p.date){(p.formatted_date())}
                    div class="flex gap-2" {
                        @for tag in &p.tags {
                            span class="bg-violet-100 dark:bg-violet-900/30 text-violet-800 dark:text-violet-300 px-2 py-1 rounded text-xs" {
                                (id(tag))
                            }
                        }
                    }
                }
            }

            p class="text-gray-700 dark:text-gray-300 mb-4" {
                (p.excerpt)
            }

            a href=(&format!("/posts/{}/{}", index, p.id)) class="text-violet-600 dark:text-violet-400 hover:underline font-medium" {
                "Read more →"
            }
        }
    }
}

// page renderers (return String)

fn index_page() -> String {
    html! {
        html {
            (head("Ryan Geary"))
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
    }.into_string()
}

fn projects_page(category: ProjectCategory) -> String {
    html! {
        html {
            (head("Projects"))
            body {
                div {
                    div class="container mx-auto px-4 py-4" {
                        (navbar())
                        div class="mt-8" {
                            (project_tabs_markup(category))
                        }
                    }
                }
            }
        }
    }
    .into_string()
}

fn posts_page() -> String {
    html! {
        html {
            (head("Posts"))
            body {
                div {
                    div class="container mx-auto px-4 py-4" {
                        (navbar())
                        div class="mt-8" {
                            (posts_list_markup(&POSTS))
                        }
                    }
                }
            }
        }
    }
    .into_string()
}

fn post_page(post: &Post) -> String {
    html! {
        html {
            (head(post.title))
            body {
                div {
                    div class="container mx-auto px-4 py-4" {
                        (navbar())
                    }

                    (post_article_markup(post))

                    div class="container mx-auto px-4 pb-8" {
                        (post_linked_list_markup(post))

                        a href="/posts" class="text-violet-600 dark:text-violet-400 hover:underline" {
                            "← Back to Posts"
                        }
                    }
                }
            }
        }
    }.into_string()
}

// static site builder

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output_dir = manifest_dir.join("output");

    cleanup_output(&output_dir);
    build_css(manifest_dir, &output_dir);
    copy_static_assets(manifest_dir, &output_dir);
    write_index(&output_dir);
    write_posts(&output_dir);
    write_projects(&output_dir);

    let file_count = count_files(&output_dir);
    println!("Done. {} files in output/", file_count);
}

fn cleanup_output(output_dir: &Path) {
    if output_dir.exists() {
        fs::remove_dir_all(output_dir).expect("failed to remove output/");
    }
    fs::create_dir_all(output_dir).expect("failed to create output/");
}

fn build_css(manifest_dir: &Path, output_dir: &Path) {
    fs::create_dir_all(output_dir.join("static")).unwrap();
    let tailwind_bin = manifest_dir.join("tailwindcss");
    Command::new(&tailwind_bin)
        .arg("-i")
        .arg(manifest_dir.join("input.css"))
        .arg("-o")
        .arg(output_dir.join("static/output.css"))
        .arg("--minify")
        .current_dir(manifest_dir)
        .status()
        .expect("failed to compile tailwind styles — ensure ./tailwindcss binary exists");
}

fn copy_static_assets(manifest_dir: &Path, output_dir: &Path) {
    let static_src = manifest_dir.join("static");
    let static_dst = output_dir.join("static");
    fs::create_dir_all(&static_dst).unwrap();

    for entry in fs::read_dir(&static_src).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        if file_name == "htmx.min.js" {
            continue;
        }
        fs::copy(entry.path(), static_dst.join(&file_name)).unwrap();
    }
}

fn write_index(output_dir: &Path) {
    fs::write(output_dir.join("index.html"), index_page()).expect("failed to write index.html");
}

fn write_posts(output_dir: &Path) {
    // Posts listing page
    let posts_dir = output_dir.join("posts");
    fs::create_dir_all(&posts_dir).unwrap();
    fs::write(posts_dir.join("index.html"), posts_page())
        .expect("failed to write posts/index.html");

    // Individual posts
    for (i, post) in POSTS.iter().enumerate() {
        // Canonical post page: output/posts/{i}/{id}/index.html
        let post_dir = output_dir.join(format!("posts/{}/{}", i, post.id));
        fs::create_dir_all(&post_dir).unwrap();
        fs::write(post_dir.join("index.html"), post_page(post)).expect(&format!(
            "failed to write posts/{}/{}index.html",
            i, post.id
        ));

        // Meta-refresh redirect: output/posts/{i}/index.html
        let redirect_dir = output_dir.join(format!("posts/{}", i));
        fs::create_dir_all(&redirect_dir).unwrap();
        fs::write(
            redirect_dir.join("index.html"),
            meta_refresh(&format!("/posts/{}/{}", i, post.id)),
        )
        .expect(&format!("failed to write posts/{}/index.html", i));
    }
}

fn write_projects(output_dir: &Path) {
    // Production projects: output/projects/index.html
    let projects_dir = output_dir.join("projects");
    fs::create_dir_all(&projects_dir).unwrap();
    fs::write(
        projects_dir.join("index.html"),
        projects_page(ProjectCategory::Production),
    )
    .expect("failed to write projects/index.html");

    // Toy projects: output/projects/toy/index.html
    let toy_dir = projects_dir.join("toy");
    fs::create_dir_all(&toy_dir).unwrap();
    fs::write(
        toy_dir.join("index.html"),
        projects_page(ProjectCategory::Toy),
    )
    .expect("failed to write projects/toy/index.html");
}

fn count_files(dir: &Path) -> usize {
    let mut count = 0;
    for _entry in walkdir(dir) {
        count += 1;
    }
    count
}

fn walkdir(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if dir.is_file() {
        files.push(dir.to_path_buf());
    } else if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            files.extend(walkdir(&entry.path()));
        }
    }
    files
}

// utility functions

fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, Options::all());
    let mut html_output = String::new();
    cmark_html::push_html(&mut html_output, parser);
    html_output
}

fn id(s: &str) -> String {
    format!("#{s}")
}

fn meta_refresh(url: &str) -> String {
    format!(
        r#"<!DOCTYPE html><html><head><meta http-equiv="refresh" content="0;url={}"></head></html>"#,
        url
    )
}
