//! HTML Templates

use maud::{html, DOCTYPE};

pub fn hello_html() -> String {
    let h = html! {
        (DOCTYPE)
        meta lang="en";
        meta charset="utf-8";
        title { "👋 Hello!" }
    };
    h.into_string()
}

pub fn echo_html(echo: &str) -> String {
    let h = html! {
        (DOCTYPE)
        meta lang="en";
        meta charset="utf-8";
        title { "📣 Echoing \"" (echo) "\"" }
    };
    h.into_string()
}

pub fn not_found_404_html() -> String {
    let h = html! {
        (DOCTYPE)
        meta lang="en";
        meta charset="utf-8";
        title { "💀 404 - Not Found! " }
        h1 { "Oops! 💀" }
        p { "Sorry, that page doesn't exist." }
        p { "💀" }
    };
    h.into_string()
}
