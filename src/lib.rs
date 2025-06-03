mod app;
mod utils;
#[cfg(feature = "hydrate")]
use app::App;
mod faucet;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[cfg(feature = "ssr")]
mod ssr_imports {
    use std::sync::Arc;

    use crate::{app::App, faucet};
    use axum::{routing::post, Extension, Router};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use leptos_meta::*;
    use worker::{event, Context, Env, HttpRequest, Result};

    fn shell(options: LeptosOptions) -> impl IntoView {
        view! {
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <title>Filecoin Forest Explorer Faucet - Get Free tFIL, USDFC and FIL</title>
                    <meta charset="utf-8" />
                    <meta name="robots" content="index, follow" />
                    <meta name="viewport" content="width=device-width, initial-scale=1" />
                    <meta
                        name="description"
                        content="Get free tFIL, USDFC and FIL on the Filecoin Forest Explorer Faucet by ChainSafe. Quickly connect your wallet, request tokens, and start building or experimenting on the Filecoin testnet or mainnet with ease."
                    />

                    <AutoReload options=options.clone() />
                    <HydrationScripts options />
                    <MetaTags />
                </head>
            </html>
        }
    }

    fn router(env: Env) -> Router {
        let leptos_options = LeptosOptions::builder()
            .output_name("client")
            .site_pkg_dir("pkg")
            .build();
        let routes = generate_route_list(App);

        // build our application with a route
        let app: axum::Router<()> = Router::new()
            .leptos_routes(&leptos_options, routes, {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            })
            .route("/api/{*fn_name}", post(leptos_axum::handle_server_fns))
            .with_state(leptos_options)
            .layer(Extension(Arc::new(env)));
        app
    }

    #[event(start)]
    fn register() {
        server_fn::axum::register_explicit::<faucet::server_api::SignWithSecretKey>();
        server_fn::axum::register_explicit::<faucet::server_api::SignedErc20Transfer>();
        server_fn::axum::register_explicit::<faucet::server_api::FaucetAddress>();
        server_fn::axum::register_explicit::<faucet::server_api::FaucetAddressStr>();
    }

    #[event(fetch)]
    async fn fetch(
        req: HttpRequest,
        env: Env,
        _ctx: Context,
    ) -> Result<axum::http::Response<axum::body::Body>> {
        _ = console_log::init_with_level(log::Level::Debug);
        use tower_service::Service;

        console_error_panic_hook::set_once();

        Ok(router(env).call(req).await?)
    }
}
