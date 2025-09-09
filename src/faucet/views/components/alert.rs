use crate::faucet::controller::FaucetController;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::{IntoView, component, view};
use std::collections::HashSet;
use std::time::Duration;
use uuid::Uuid;

const MESSAGE_FADE_AFTER: Duration = Duration::new(3, 0);
const MESSAGE_REMOVAL_AFTER: Duration = Duration::new(3, 500_000_000);

#[component]
pub fn ErrorMessages(
    errors: Vec<(Uuid, String)>,
    faucet: RwSignal<FaucetController>,
) -> impl IntoView {
    let (fading_messages, set_fading_messages) = signal(HashSet::new());
    view! {
        <div class="error-alert-container">
            {errors
                .into_iter()
                .map(|(id, error)| {
                    spawn_local(async move {
                        set_timeout(
                            move || {
                                set_fading_messages
                                    .update(|fading| {
                                        fading.insert(id);
                                    });
                            },
                            MESSAGE_FADE_AFTER,
                        );
                        set_timeout(
                            move || {
                                set_fading_messages
                                    .update(|fading| {
                                        fading.remove(&id);
                                    });
                                faucet.get().remove_error_message(id);
                            },
                            MESSAGE_REMOVAL_AFTER,
                        );
                    });

                    view! {
                        <div
                            class=move || {
                                if fading_messages.get().contains(&id) { "error-alert-faded" } else { "error-alert" }
                            }
                            role="alert"
                        >
                            <span class="block sm:inline">{error}</span>
                            <span class="absolute top-0 bottom-0 right-0 px-4 py-3">
                                <svg
                                    class="close-icon"
                                    role="button"
                                    xmlns="http://www.w3.org/2000/svg"
                                    viewBox="0 0 20 20"
                                    on:click=move |_| {
                                        faucet.get().remove_error_message(id);
                                    }
                                >
                                    <title>Close</title>
                                    <path d="M14.348 14.849a1.2 1.2 0 0 1-1.697 0L10 11.819l-2.651 3.029a1.2 1.2 0 1 1-1.697-1.697l2.758-3.15-2.759-3.152a1.2 1.2 0 1 1 1.697-1.697L10 8.183l2.651-3.031a1.2 1.2 0 1 1 1.697 1.697l-2.758 3.152 2.758 3.15a1.2 1.2 0 0 1 0 1.698z" />
                                </svg>
                            </span>
                        </div>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}
