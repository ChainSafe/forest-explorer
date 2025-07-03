<!--

## A short guide to adding a changelog entry

- pick a section to which your change belongs in _Forest Explorer unreleased_,
- the entry should follow the format:

  `[#ISSUE_NO](link to the issue): <short description>`, for example:

  [#1234](https://github.com/chainsafe/forest-explorer/pull/1234): Add support for pineconenet

- if the change does not have an issue, use the PR number instead - the PR must
  have a detailed description of the change and its motivation. Consider
  creating a separate issue if the change is complex enough to warrant it,
- the changelog is not a place for the full description of the change, it should
  be a short summary of the change,
- if the change does not directly affect the user, it should not be included in
  the changelog - for example, refactoring of the codebase,
- review the entry to make sure it is correct and understandable and that it
  does not contain any typos,
- the entries should not contradict each other - if you add a new entry, ensure
  it is consistent with the existing entries.

-->

## Forest Explorer unreleased

### Breaking

### Added

- [#135](https://github.com/ChainSafe/forest-explorer/pull/135) Added link to
  request for faucet top-up which is configurable using github env var.

- [#200](https://github.com/ChainSafe/forest-explorer/pull/200) Make the CID of
  transaction clickable.

- [#199](https://github.com/ChainSafe/forest-explorer/pull/199) Added faucet
  transaction history button which is configurable using github env var.

- [#137](https://github.com/ChainSafe/forest-explorer/issues/137) Different rate
  limiter for calibnet and mainnet

- [#257](https://github.com/ChainSafe/forest-explorer/issues/257) Added per
  wallet daily cap on token claim.

### Changed

- [#136](https://github.com/ChainSafe/forest-explorer/pull/136) Increased the
  drip time to 10 minutes.

### Removed

### Fixed

- [#176](https://github.com/ChainSafe/forest-explorer/pull/176) Fixed the
  message encoding based on the network

-[#205](https://github.com/ChainSafe/forest-explorer/pull/205) Reposition top-up
request link.

## Forest Explorer v1.0.0

The first release of the Forest Explorer, featuring:

- calibration network faucet available at
  [https://forest-explorer.chainsafe.dev/faucet/calibnet](https://forest-explorer.chainsafe.dev/faucet/calibnet)
- mainnet faucet available at
  [https://forest-explorer.chainsafe.dev/faucet/mainnet](https://forest-explorer.chainsafe.dev/faucet/mainnet)
- block explorer Proof of Concept available at
  [https://forest-explorer.chainsafe.dev/](https://forest-explorer.chainsafe.dev/)

Faucets have in-built rate limiting to prevent abuse.

The frontend has been designed by world-class designers and likely to be
featured across world art galleries.
