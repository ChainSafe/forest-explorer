use leptos::{component, create_local_resource, event_target_value, view, IntoView, SignalGet};
use leptos_meta::*;

use leptos::*;
use leptos_router::*;

use serde_json::{json, Value};

const GLIF_CALIBNET: &str = "https://api.calibration.node.glif.io";

#[component]
pub fn Loader(loading: impl Fn() -> bool + 'static) -> impl IntoView {
    view! { <span class:loader=loading /> }
}

#[component]
pub fn BlockchainExplorer() -> impl IntoView {
    let rpc_provider = create_rw_signal(String::from(GLIF_CALIBNET));
    let network_name = create_local_resource(
        move || rpc_provider.get(),
        move |provider| async move {
            let client = reqwest::Client::new();
            let res = client
                .post(provider)
                .json(&json! {
                    {
                        "jsonrpc": "2.0",
                        "method": "Filecoin.StateNetworkName",
                        "params": [],
                        "id": 0
                    }
                })
                .send()
                .await
                .ok()?;
            log::info!("Got response: {:?}", res);
            Some(String::from(
                res.json::<Value>()
                    .await
                    .ok()?
                    .get("result")
                    .cloned()?
                    .as_str()?,
            ))
        },
    );

    let network_version = create_local_resource(
        move || rpc_provider.get(),
        move |provider| async move {
            let client = reqwest::Client::new();
            let res = client
                .post(provider)
                .json(&json! {
                    {
                        "jsonrpc": "2.0",
                        "method": "Filecoin.StateNetworkVersion",
                        "params": [[]],
                        "id": 0
                    }
                })
                .send()
                .await
                .ok()?;
            log::info!("Got response: {:?}", res);
            res.json::<Value>()
                .await
                .ok()
                .unwrap_or_default()
                .get("result")
                .cloned()
                .unwrap_or_default()
                .as_u64()
        },
    );

    view! {
        <h1 class="mb-4 text-4xl font-extrabold leading-none tracking-tight text-gray-900 md:text-5xl lg:text-6xl">
            Forest Explorer
        </h1>
        <select on:change=move |ev| { rpc_provider.set(event_target_value(&ev)) }>
            <option value="https://api.calibration.node.glif.io">Glif.io Calibnet</option>
            <option value="https://api.node.glif.io/">Glif.io Mainnet</option>
        </select>
        <p>StateNetworkName</p>
        <p class="px-8">
            <span>{move || network_name.get()}</span>
            <Loader loading=move || network_name.loading().get() />
        </p>
        <p>StateNetworkVersion</p>
        <p class="px-8">
            <span>{move || network_version.get()}</span>
            <Loader loading=move || network_name.loading().get() />
        </p>
    }
}

#[component]
pub fn AddressParser() -> impl IntoView {
    let (hex_address, set_hex_address) = create_signal(String::new());
    let (sig_type, set_sig_type) = create_signal(String::new());
    let (parsed_address, set_parsed_address) = create_signal(String::new());

    let on_input = move |ev| {
        use crate::key::*;
        use std::str::FromStr;
        let value = event_target_value(&ev);
        set_hex_address.set(value.clone());
        log::info!("Address input changed: {}", value);

        if let Ok(key_info) = KeyInfo::from_str(&hex_address.get()) {
            if let Ok(key) = Key::try_from(key_info) {
                set_sig_type.set(format!("{:?}", key.key_info.r#type));
                set_parsed_address.set(key.address.to_string());
            }
        }
    };

    view! {
        <h1>Address Parser</h1>
        <input type="text" placeholder="Enter exported address" on:input=on_input prop:value=hex_address />
        <div>
            <label>Signature type:</label>
            <span>{sig_type}</span>
        </div>
        <div>
            <label>Address:</label>
            <span>{parsed_address}</span>
        </div>
    }
}

#[component]
pub fn Signer() -> impl IntoView {
    let to_be_signed = create_rw_signal(String::new());
    let signed_message = create_rw_signal(String::new());

    let public_key = create_rw_signal(String::from("f15ydyu3d65gznpp2qxwpkjsgz4waubeunn6upvla"));
    let in_message = create_rw_signal(String::from("Hello world!"));
    let signature = create_rw_signal(String::from("547cde13e913e7cb716dba12c30cdc2a9789f1defb7d8eec50a008224713e754486ffc2f1c81722e333a8b347a7621c06919221484d7b665cc0a92fd037ffbf301"));

    let is_valid = move || {
        use crate::key::*;
        verify(&signature.get(), &public_key.get(), &in_message.get()).unwrap_or(false)
    };

    let on_sign = move |ev| {
        use crate::key::*;
        let value = event_target_value(&ev);
        let key = secret_key();
        let msg = sign(key.key_info.r#type, &key.key_info.private_key, value.as_bytes());
        match msg {
            Ok(sig) => signed_message.set(hex::encode(sig.bytes())),
            Err(e) => log::error!("Error signing message: {}", e),
        }
    };

    view! {
        <h1 class="text-4xl font-bold mb-6">Sign and Validate</h1>
        <div>
            <h2 class="text-2xl font-bold mb-4">Sign Message</h2>
            <input
                type="text"
                placeholder="Enter message to sign"
                prop:value=to_be_signed
                on:input=on_sign
                class="w-full border border-gray-300 p-2 mb-2 mx-2"
            />
            <div>
                <label>Signed Message:</label>
                <div class="max-w-80 break-all">{signed_message}</div>
            </div>
        </div>

        <div class="my-8">
            <hr />
        </div>

        <div>
            <h2 class="text-2xl font-bold mb-4">Validate Signature</h2>
            <input
                type="text"
                placeholder="Enter public key"
                prop:value=public_key
                on:input=move |ev| { public_key.set(event_target_value(&ev)) }
                class="w-full border border-gray-300 p-2 mb-2 mx-2"
            />
            <input
                type="text"
                placeholder="Enter message"
                prop:value=in_message
                on:input=move |ev| { in_message.set(event_target_value(&ev)) }
                class="w-full border border-gray-300 p-2 mb-2 mx-2"
            />
            <input
                type="text"
                placeholder="Enter signature"
                prop:value=signature
                on:input=move |ev| { signature.set(event_target_value(&ev)) }
                class="w-full border border-gray-300 p-2 mb-2 mx-2"
            />

            {move || {
                if !in_message.get().is_empty() {
                    if is_valid() {
                        view! { <p class="text-4xl font-bold text-green-500">Valid!</p> }
                    } else {
                        view! { <p class="text-4xl font-bold text-red-500">Invalid!</p> }
                    }
                } else {
                    view! {}
                }
            }}
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet href="/style.css" />
        <Link rel="icon" type_="image/x-icon" href="/favicon.ico" />
        <Router>
            <Routes>
                <Route path="/" view=BlockchainExplorer />
                <Route path="/address" view=AddressParser />
                <Route path="/signer" view=Signer />
            </Routes>
        </Router>
    }
}
