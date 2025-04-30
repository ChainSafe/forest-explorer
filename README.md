# Forest Explorer

[![codecov](https://codecov.io/github/ChainSafe/forest-explorer/graph/badge.svg?token=J2ZVD5FOEC)](https://codecov.io/github/ChainSafe/forest-explorer)

Forest Explorer is a server-less inspector of the Filecoin blockchain.

## Implementation

[Rust](https://www.rust-lang.org/) + [Leptos](https://leptos.dev/) application
which is compiled to a server [WASM](https://webassembly.org/) module and a
client WASM module. The server module is hosted by
[CloudFlare](https://workers.cloudflare.com/). It pre-renders a HTML response
and [hydrates](https://book.leptos.dev/ssr/index.html) it (i.e. add reactivity)
via the client WASM module.

Anything pushed to `main` will automatically be deployed at
<https://forest-explorer.chainsafe.dev>.

## Development

Installing node(LTS versions recommended).

Running `corepack enable` to opt-in corepack, see
[docs](https://yarnpkg.com/corepack#installation) for details.

Running `yarn` or `yarn --immutable` once to install all required npm
dependencies.

Running `yarn start` will spawn a local copy of the explorer.

To speed up the build during development, you can run `yarn dev` which will skip
the optimization step.

You can define secrets for your local faucet in the `.dev.vars` file. This file
is ignored by git.

```
SECRET_WALLET=
SECRET_MAINNET_WALLET=
RATE_LIMITER_DISABLED=true
```

Note - the `RATE_LIMITER_DISABLED` variable is required to be set to `true` in
order to bypass the rate limiter in the local environment if you want to test
the faucet.

## Dependencies

- [wrangler](https://github.com/cloudflare/wrangler2)
- [wasm-pack](https://github.com/rustwasm/wasm-pack)
- [worker-build](https://github.com/cloudflare/workers-rs/tree/main/worker-build)

## Custom/Preview deployments

### Account & domain

Update the `[env.preview]` configuration in [`wrangler.toml`](./wrangler.toml):

1. Set `account_id` to your CloudFlare account ID.
2. Optional: Set `routes` to match your custom domain. By default it will deploy
   to: `<YOUR_WORKER_NAME>.<YOUR_SUBDOMAIN>.workers.dev`

### Rate limiter

1. RateLimiter is enabled by default. To disable the rate limiter, run:
   `npx wrangler@latest secret put RATE_LIMITER_DISABLED true`.

:warning: This is not recommended for production use as it will expose your
service to abuse.

### Wallets

Set `SECRET_WALLET` (calibnet) and/or `SECRET_MAINNET_WALLET` (mainnet) using
`npx wrangler@latest secret put` (values are exported private keys, see
`forest-wallet export`).

### Deployment

Run `npx wrangler@latest deploy --env preview`.

:information_source: **Note:**  
To generate clean and consistent preview URLs, it's recommended to configure
your **Cloudflare Workers subdomain** (`account_name`) as
`forest-explorer-preview` in the Cloudflare dashboard (you only need to do this
once).

Then, during deployment, use the `--name` option to set the preview Worker name
based on the latest Git commit hash:

```bash
npx wrangler@latest deploy --env preview --name $(git rev-parse --short HEAD)
```

This will deploy your worker to a URL like:
`https://<COMMIT_HASH>.forest-explorer-preview.workers.dev`

:lock: **Setting Secrets for Preview Workers**

If you use a commit-based name, you **must also specify it when setting
secrets**, so they are attached to the correct Worker:

```bash
npx wrangler@latest secret put MY_SECRET --env preview --name $(git rev-parse --short HEAD)
```
