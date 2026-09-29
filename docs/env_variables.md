# Environment Variables

This document outlines the environment variables used in the Forest Explorer
project. These variables configure various aspects of the application, such as
URL endpoints and token drip amounts.

| Environment Variable            | Description                                             | Default Value                                                |
| ------------------------------- | ------------------------------------------------------- | ------------------------------------------------------------ |
| FAUCET_TOPUP_REQ_URL            | URL for faucet top-up requests                          | https://github.com/ChainSafe/forest-explorer/discussions/134 |
| FAUCET_TX_URL_CALIBNET          | Base URL for Calibnet transactions                      | https://beryx.io/fil/calibration/                            |
| FAUCET_TX_URL_MAINNET           | Base URL for Mainnet transactions                       | https://beryx.io/fil/mainnet/                                |
| MAINNET_DRIP_AMOUNT             | Amount of tokens to drip on Mainnet in nanoFIL          | 10000000 (0.01 FIL)                                          |
| CALIBNET_DRIP_AMOUNT            | Amount of tokens to drip on Calibnet in nanoFIL         | 5000000000 (5 tFIL)                                          |
| CALIBNET_USDFC_DRIP_AMOUNT      | Amount of USDFC tokens to drip on Calibnet in nanoUSDFC | 5000000000 (5 tUSDFC)                                        |
| CALIBNET_USDFC_CONTRACT_ADDRESS | Contract address for Calibnet USDFC                     | 0xb3042734b608a1B16e9e86B374A3f3e389B4cDf0                   |
