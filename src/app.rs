use fvm_shared::{
    address::{Address, Network},
    econ::TokenAmount,
};
use leptos::{component, create_local_resource, event_target_value, view, IntoView, SignalGet};
use leptos_meta::*;

use leptos::*;
use leptos_router::*;

use std::str::FromStr;

use crate::message::{message_cid, message_transfer, Message};
use crate::rpc_context::RpcContext;
use crate::{
    key::{secret_key, sign},
    message::SignedMessage,
};

#[component]
pub fn Loader(loading: impl Fn() -> bool + 'static) -> impl IntoView {
    view! { <span class:loader=loading /> }
}

#[component]
pub fn BlockchainExplorer() -> impl IntoView {
    let rpc_context = RpcContext::use_context();
    let network_name = create_local_resource(
        move || rpc_context.get(),
        move |provider| async move { provider.network_name().await.ok() },
    );

    let network_version = create_local_resource(
        move || rpc_context.get(),
        move |provider| async move { provider.network_version().await.ok() },
    );

    view! {
        <h1 class="mb-4 text-4xl font-extrabold leading-none tracking-tight text-gray-900 md:text-5xl lg:text-6xl">
            Forest Explorer
        </h1>
        <select on:change=move |ev| { rpc_context.set(event_target_value(&ev)) }>
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
        let msg = sign(
            key.key_info.r#type,
            &key.key_info.private_key,
            value.as_bytes(),
        );
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
                    view! { <p></p> }
                }
            }}
        </div>
    }
}

fn parse_address(s: &str) -> anyhow::Result<Address> {
    Ok(Network::Testnet
        .parse_address(s)
        .or_else(|_| Network::Mainnet.parse_address(s))?)
}

#[component]
pub fn Faucet() -> impl IntoView {
    let rpc_context = RpcContext::use_context();
    let faucet_balance = create_local_resource_with_initial_value(
        move || rpc_context.get(),
        move |provider| async move {
            provider
                .wallet_balance(
                    parse_address("t15ydyu3d65gznpp2qxwpkjsgz4waubeunn6upvla")
                        .unwrap_or(Address::new_id(1)),
                )
                .await
                .ok()
                .unwrap_or(TokenAmount::from_atto(0))
        },
        Some(TokenAmount::from_atto(0)),
    );
    let target_address =
        create_rw_signal(String::from("t12icwx77skr3hv4mekth7kol3fuhymcya6zczxgi"));
    let target_balance = create_local_resource_with_initial_value(
        move || (rpc_context.get(), target_address.get()),
        move |(provider, address)| async move {
            if let Ok(address) = parse_address(&address) {
                provider
                    .wallet_balance(address)
                    .await
                    .ok()
                    .unwrap_or(TokenAmount::from_atto(0))
            } else {
                TokenAmount::from_atto(0)
            }
        },
        None,
    );

    view! {
        <h1 class="text-4xl font-bold mb-6 text-center">Faucet</h1>

        <div class="max-w-2xl mx-auto">
            <div class="my-4 flex">
                <input
                    type="text"
                    placeholder="Enter target address"
                    prop:value=target_address
                    on:input=move |ev| { target_address.set(event_target_value(&ev)) }
                    class="flex-grow border border-gray-300 p-2 rounded-l"
                />
                <button
                    class="bg-green-500 hover:bg-green-600 text-white font-bold py-2 px-4 rounded-r"
                    on:click=move |_| {
                        match parse_address(&target_address.get()) {
                            Ok(addr) => {
                                let rpc = rpc_context.get();
                                let from = secret_key();
                                spawn_local(async move {
                                    let nonce = rpc.mpool_get_nonce(from.address).await.unwrap();
                                    let mut msg = message_transfer(from.address, addr, TokenAmount::from_whole(1));
                                    msg.sequence = nonce;
                                    log::info!("Pre Estimated gas: {:?}", msg);
                                    match rpc.estimate_gas(msg).await {
                                        Ok(msg) => {
                                            log::info!("Post Estimated gas: {:?}", msg);
                                            match sign(
                                                from.key_info.r#type,
                                                &from.key_info.private_key,
                                                message_cid(&msg).to_bytes().as_slice(),
                                            ) {
                                                Ok(sig) => {
                                                    let smsg = SignedMessage::new_unchecked(msg, sig);
                                                    let cid = rpc.mpool_push(smsg).await;
                                                    log::info!("Sent message: {:?}", cid);
                                                    log::info!("Send button clicked {}", target_address.get());
                                                }
                                                Err(e) => log::error!("Error signing message: {}", e),
                                            }
                                        }
                                        Err(e) => log::error!("Error estimating gas: {}", e),
                                    }
                                });
                            }
                            Err(e) => {
                                log::error!("Error parsing address: {}", e);
                            }
                        }
                        log::info!("Send button clicked {}", target_address.get());
                    }
                >
                    Send
                </button>
            </div>
            <div class="flex justify-between my-4">
                <div>
                    <h3 class="text-lg font-semibold">Faucet Balance:</h3>
                    <p class="text-xl">{move || faucet_balance.get().unwrap_or_default().to_string()}</p>
                </div>
                <div>
                    <h3 class="text-lg font-semibold">Target Balance:</h3>
                    <p class="text-xl">{move || target_balance.get().unwrap_or_default().to_string()}</p>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    RpcContext::provide_context();

    view! {
        <Stylesheet href="/style.css" />
        <Link rel="icon" type_="image/x-icon" href="/favicon.ico" />
        <Router>
            <Routes>
                <Route path="/" view=BlockchainExplorer />
                <Route path="/address" view=AddressParser />
                <Route path="/signer" view=Signer />
                <Route path="/faucet" view=Faucet />
            </Routes>
        </Router>
    }
}
