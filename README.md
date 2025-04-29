# Forest Explorer

[![codecov](https://codecov.io/github/ChainSafe/forest-explorer/graph/badge.svg?token=J2ZVD5FOEC)](https://codecov.io/github/ChainSafe/forest-explorer)

Forest Explorer is a server-less inspector of the Filecoin blockchain.

# Implementation

[Rust](https://www.rust-lang.org/) + [Leptos](https://leptos.dev/) application
which is compiled to a server [WASM](https://webassembly.org/) module and a
client WASM module. The server module is hosted by
[CloudFlare](https://workers.cloudflare.com/). It pre-renders a HTML response
and [hydrates](https://book.leptos.dev/ssr/index.html) it (i.e. add reactivity)
via the client WASM module.

Anything pushed to `main` will automatically be deployed at
<https://forest-explorer.chainsafe.dev>.

# Development

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

# Dependencies

- [wrangler](https://github.com/cloudflare/wrangler2)
- [wasm-pack](https://github.com/rustwasm/wasm-pack)
- [worker-build](https://github.com/cloudflare/workers-rs/tree/main/worker-build)

# Custom deployments

### Account & domain

1. In [`wrangler.toml`](./wrangler.toml), set `account_id` to your CloudFlare
   account ID.
2. In [`wrangler.toml`](./wrangler.toml), set `pattern` in routes to match your
   domain or comment it out to use default `.workers.dev` domain(recommended for
   generating preview url).

In order to deploy to a different CloudFlare account, you need to do the
following:

### Rate limiter

If you are using a free Cloudflare account:

1. Replace `new_classes` with `new_sqlite_classes` in `Durable Object`
   configuration.
2. To disable the rate limiter, run:
   `npx wrangler@latest secret put RATE_LIMITER_DISABLED true`. :warning: This
   is not recommended for production use as it will expose your service to
   abuse.

### Wallets

Set `SECRET_WALLET` (calibnet) and/or `SECRET_MAINNET_WALLET` (mainnet) using
`npx wrangler@latest secret put` (values are exported private keys, see
`forest-wallet export`).

### Deployment

Run `npx wrangler@latest deploy`.
