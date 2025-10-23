use leptos::prelude::*;

use crate::api::auth::auth_state::AuthStateSignal;
/// A single document section: heading (without numeric prefix),
/// one-or-more description paragraphs, and optional bullets.
#[derive(Clone)]
pub struct Section {
    pub heading: String,
    pub descriptions: Vec<String>,
    pub bullets: Vec<String>,
}

/// Full document: title, intro paragraphs, and sections.
#[derive(Clone)]
pub struct Doc {
    pub title: String,
    pub intro: Vec<String>,
    pub sections: Vec<Section>,
}

/// Reusable view that renders a Doc with Tailwind styling tuned for the screenshot.
#[component]
pub fn DocView(doc: Doc) -> impl IntoView {
    view! {
        <div class="flex flex-col items-start gap-8 w-full max-w-3xl px-4 sm:px-6 lg:px-0 text-left">
            // Title
            <h1 class="text-xl sm:text-2xl lg:text-3xl font-bold leading-snug tracking-tight text-[#01030A]">
                {doc.title}
            </h1>

            // Intro paragraphs
            { doc.intro.into_iter().map(|p| {
                view! {
                    <p class="text-sm sm:text-base lg:text-lg font-normal leading-relaxed text-[#45556C]">
                        {highlight_email(&p)}
                    </p>
                }
            }).collect_view() }

            // Sections
            { doc.sections.into_iter().enumerate().map(|(idx, section)| {
                view! {
                    <section class="flex flex-col items-start gap-2 w-full">
                        <h2 class="text-base sm:text-lg font-bold leading-relaxed text-[#45556C]">
                            {format!("{}. {}", idx + 1, section.heading)}
                        </h2>

                        // descriptions
                        { section.descriptions.into_iter().map(|d| {
                            view! {
                                <p class="text-sm sm:text-base lg:text-lg font-normal leading-relaxed text-[#45556C]">
                                    {highlight_email(&d)}
                                </p>
                            }
                        }).collect_view() }

                        // bullets
                        { if !section.bullets.is_empty() {
                            view! {
                                <ul class="list-disc ml-4 space-y-1 text-sm sm:text-base lg:text-lg font-normal leading-relaxed text-[#45556C]">
                                    { section.bullets.into_iter().map(|b| view! {
                                        <li>{highlight_email(&b)}</li>
                                    }).collect_view() }
                                </ul>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }}
                    </section>
                }
            }).collect_view() }
        </div>
    }
}

fn highlight_email(text: &str) -> impl IntoView {
    use regex::Regex;
    let email_regex = Regex::new(r"([a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+)").unwrap();

    let mut views = vec![];
    let mut last_end = 0;

    for cap in email_regex.captures_iter(text) {
        let m = cap.get(0).unwrap();
        let start = m.start();
        let end = m.end();

        // push text before email
        if start > last_end {
            views.push(view! { <span>{text[last_end..start].to_string()}</span> }.into_any());
        }

        let auth = AuthStateSignal::auth_state().get();
        // push highlighted email
        views.push({
            let mail = m.as_str().to_string();
            let is_logged_in = move || auth.is_authenticated();
            let mailto = if is_logged_in() {
                format!(
                    "https://mail.google.com/mail/?view=cm&fs=1&to={}",
                    mail.trim()
                )
            } else {
                format!("mailto:{}", mail.trim())
            };
            view! {
                <span><a target="_blank" href={mailto}
                   class="text-blue-600 underline font-medium">
                    {mail}
                </a>
                </span>
            }
            .into_any()
        });

        last_end = end;
    }

    // push remaining text
    if last_end < text.len() {
        views.push(view! { <span>{text[last_end..].to_string()}</span> }.into_any());
    }

    views
}
