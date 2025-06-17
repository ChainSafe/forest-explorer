use crate::utils::format::{SearchPath, format_url};
use crate::utils::transaction_id::TransactionId;
use leptos::prelude::*;
use leptos::{IntoView, component, view};
use url::Url;

use crate::faucet::controller::FaucetController;

#[component]
pub fn TransactionList(
    messages: Vec<(TransactionId, bool)>,
    faucet_tx_base_url: RwSignal<Option<Url>>,
) -> impl IntoView {
    view! {
        <div class="transaction-container">
            <h3 class="title">Transactions:</h3>
            <ul class="bullet-list">
                {messages
                    .into_iter()
                    .map(|(msg, sent)| {
                        let (tx, status) = if sent {
                            let tx = faucet_tx_base_url
                                .get()
                                .as_ref()
                                .and_then(|base_url| {
                                    format_url(base_url, SearchPath::Transaction, &msg.to_string()).ok()
                                })
                                .map(|tx_url| {
                                    view! {
                                        <a href=tx_url.to_string() target="_blank" class="link-text-hover">
                                            {msg.to_string()}
                                        </a>
                                    }
                                        .into_any()
                                })
                                .unwrap_or_else(|| view! { {msg.to_string()} }.into_any());
                            (tx, "(confirmed)")
                        } else {
                            let tx = view! { {msg.to_string()} }.into_any();
                            (tx, "(pending)")
                        };
                        view! { <li>"Tx:" {tx} {status}</li> }
                    })
                    .collect::<Vec<_>>()}
            </ul>
        </div>
    }
}

#[component]
pub fn TransactionHistoryButton(
    faucet: RwSignal<FaucetController>,
    faucet_tx_base_url: RwSignal<Option<Url>>,
) -> impl IntoView {
    view! {
        {move || {
            match faucet_tx_base_url.get() {
                Some(ref base_url) => {
                    match format_url(base_url, SearchPath::Address, &faucet.get().get_sender_address()) {
                        Ok(addr_url) => {
                            view! {
                                <button>
                                    <a class="btn" href=addr_url.to_string() target="_blank" rel="noopener noreferrer">
                                        "Transaction History"
                                    </a>
                                </button>
                            }
                                .into_any()
                        }
                        Err(_) => ().into_any(),
                    }
                }
                None => ().into_any(),
            }
        }}
    }
}
