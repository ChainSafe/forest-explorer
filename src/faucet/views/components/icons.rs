use leptos::prelude::*;

#[component]
pub fn CheckIcon(#[prop(default = String::new())] class: String) -> impl IntoView {
    view! {
        <svg class=format!("check-icon {}", class) fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
    }
}

#[component]
pub fn LightningIcon(#[prop(default = String::new())] class: String) -> impl IntoView {
    view! {
        <svg class=format!("lighting-icon {}", class) fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
    }
}

#[component]
pub fn Loader(loading: impl Fn() -> bool + 'static + Send) -> impl IntoView {
    view! { <span class:loader=loading /> }
}
