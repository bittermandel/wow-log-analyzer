#![allow(non_snake_case)]
use std::io::Read;

// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;
use dioxus_router::prelude::*;

mod parser;

fn main() {
    // launch the dioxus app in a webview
    dioxus_desktop::launch(App);
}

#[derive(Routable, Clone)]
enum Route {
    #[route("/")]
    Home,

    #[route("/analyze/:log")]
    Analyze { log: String },
}

fn App(cx: Scope) -> Element {
    render! {
        Router::<Route> {}
    }
}

// define a component that renders a div with the text "Hello, world!"
fn Home(cx: Scope) -> Element {
    let files = use_ref(cx, Logs::new);
    render!(div {
        main {
            h1 { "Hello, world!" }
            files.read().list_log_files().iter().map(|file| {
                render!(div {
                    i {
                        Link {
                            to: Route::Analyze {
                                log: file.to_string()
                            },
                            "Analyze"
                        }
                    }
                })
            })
        }
    })
}

#[inline_props]
// define a component that renders a div with the text "Hello, world!"
fn Analyze(cx: Scope, log: String) -> Element {
    let logs = use_ref(cx, Logs::new);
    render!(div {
        main {
            h1 { "Hello, world!" }
            logs.read().read_log(log.to_string())
        }
    })
}

struct Logs {
    path: String,
}

impl Logs {
    fn new() -> Self {
        Self {
            path: "C:\\Program Files (x86)\\World of Warcraft\\_retail_\\Logs".to_string(),
        }
    }

    fn list_log_files(&self) -> Vec<String> {
        let mut files = Vec::new();
        let entries = std::fs::read_dir(&self.path).expect("Could not read directory");

        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name.ends_with(".txt") && file_name.contains("CombatLog") {
                                files.push(file_name.to_string());
                            }
                        }
                    }
                }
            }
        }

        files
    }

    fn read_log(&self, file: String) {
        let path = format!("{}\\{}", self.path, file);
        let parser = parser::Parser::new();
        parser.parse_file(path);
    }
}

struct LogLine {}

impl LogLine {
    fn parse(line: String) -> Self {
        Self {}
    }
}
