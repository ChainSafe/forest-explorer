use leptos::prelude::*;

#[component]
pub fn CheckIcon(
    #[prop(default = String::new())] class: String
) -> impl IntoView {
    view! {
        <svg 
            class=format!("h-5 w-5 text-green-500 mr-2 flex-shrink-0 {}", class)
            fill="none" 
            stroke="currentColor" 
            viewBox="0 0 24 24"
        >
            <path 
                stroke-linecap="round" 
                stroke-linejoin="round" 
                stroke-width="2" 
                d="M5 13l4 4L19 7" 
            />
        </svg>
    }
} 