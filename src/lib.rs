use leptos::{component, create_local_resource, event_target_value, view, IntoView, SignalGet};
use leptos_meta::*;

use leptos::*;

use serde_json::{json, Value};

const GLIF_CALIBNET: &str = "https://api.calibration.node.glif.io";

#[component]
pub fn Loader(loading: impl Fn() -> bool + 'static) -> impl IntoView {
    view! {
        <span class:loader=loading />
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
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
        <Stylesheet href="/pkg/style.css"/>
        <Link rel="icon" type_="image/x-icon" href="/pkg/favicon.ico"/>
        <h1 class="mb-4 text-4xl font-extrabold leading-none tracking-tight text-gray-900 md:text-5xl lg:text-6xl">Forest Explorer</h1>
        <select on:change=move |ev| {
            rpc_provider.set(event_target_value(&ev))
        }>
            <option value="https://api.calibration.node.glif.io">Glif.io Calibnet</option>
            <option value="https://api.node.glif.io/">Glif.io Mainnet</option>
        </select>
        <p>StateNetworkName</p>
        <p class="px-8">
            <span>{move || network_name.get() }</span>
            <Loader loading={move || network_name.loading().get() }/>
        </p>
        <p>StateNetworkVersion</p>
        <p class="px-8">
            <span>{move || network_version.get() }</span>
            <Loader loading={move || network_name.loading().get() }/>
        </p>
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}

#[cfg(feature = "ssr")]
mod ssr_imports {
    use crate::App;
    use axum::http::{HeaderValue, StatusCode};
    use axum::{
        extract::Path,
        response::IntoResponse,
        routing::{get, post},
        Router,
    };
    use include_dir::{include_dir, Dir};
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use worker::{event, Context, Env, HttpRequest, Result};

    static PKG_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/pkg/");

    async fn serve_static(Path(path): Path<String>) -> impl IntoResponse {
        let mime_type = mime_guess::from_path(&path).first_or_text_plain();
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_str(mime_type.as_ref()).unwrap(),
        );
        match PKG_DIR.get_file(path) {
            None => (StatusCode::NOT_FOUND, headers, "File not found.".as_bytes()),
            Some(file) => (StatusCode::OK, headers, file.contents()),
        }
    }

    fn router() -> Router {
        let leptos_options = LeptosOptions::builder()
            .output_name("client")
            .site_pkg_dir("pkg")
            .build();
        let routes = generate_route_list(App);

        // build our application with a route
        let app: axum::Router<()> = Router::new()
            .leptos_routes(&leptos_options, routes, App)
            .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
            .route("/pkg/*file_name", get(serve_static))
            .with_state(leptos_options);
        app
    }

    #[event(fetch)]
    async fn fetch(
        req: HttpRequest,
        _env: Env,
        _ctx: Context,
    ) -> Result<axum::http::Response<axum::body::Body>> {
        _ = console_log::init_with_level(log::Level::Debug);
        use tower_service::Service;

        console_error_panic_hook::set_once();

        Ok(router().call(req).await?)
    }
}
