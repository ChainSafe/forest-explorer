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

Running `npx wrangler@latest dev` will spawn a local copy of the explorer.

To speed up the build during development, you can run
`npx wrangler@latest dev --env quick` which will skip the optimization step.

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
