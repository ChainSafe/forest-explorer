// PAGES: Describes the structure of each page, including which buttons and links should be present.
// path: The route to test (relative to the base URL)
// buttons: Array of buttons expected on the page
// links: Array of links expected on the page
export const PAGES = [
  {
    path: "",
    buttons: ["Faucet List"],
    links: ["Filecoin Slack", "documentation"],
  },
  {
    path: "/faucet",
    buttons: ["Home"],
    links: [
      "💰 Calibration Network USDFC Faucet",
      "🧪 Calibration Network Faucet",
      "🌐 Mainnet Network Faucet",
    ],
  },
  {
    path: "/faucet/calibnet_usdfc",
    buttons: ["Faucet List", "Transaction History", "Claim tUSDFC"],
  },
  {
    path: "/faucet/calibnet",
    buttons: ["Faucet List", "Transaction History", "Claim tFIL"],
  },
  {
    path: "/faucet/mainnet",
    buttons: ["Faucet List", "Transaction History", "Claim FIL"],
  },
];

// BUTTON_ACTIONS: Defines the expected behavior for each button on each page.
// navigate: Button should navigate to another page
// clickable: Button should be interactive and should not throw any error when clicked.
// expectError: Button should display an error message when clicked.
export const BUTTON_ACTIONS = {
  "/faucet/calibnet_usdfc": {
    "Faucet List": { type: "navigate" },
    "Transaction History": { type: "clickable" },
    "Claim tUSDFC": { type: "expectError", errorMsg: "Invalid address" },
  },
  "/faucet/calibnet": {
    "Faucet List": { type: "navigate" },
    "Transaction History": { type: "clickable" },
    "Claim tFIL": { type: "expectError", errorMsg: "Invalid address" },
  },
  "/faucet/mainnet": {
    "Faucet List": { type: "navigate" },
    "Transaction History": { type: "clickable" },
    "Claim FIL": { type: "expectError", errorMsg: "Invalid address" },
  },
};

// CLAIM_TESTS: Specifies claim scenarios for each faucet page, including valid and invalid addresses.
// addresses: List of addresses to test
// expectSuccess: Array indicating if each address should succeed or fail
export const CLAIM_TESTS = [
  {
    path: "/faucet/calibnet_usdfc",
    button: "Claim tUSDFC",
    addresses: [
      "0xAe9C4b9508c929966ef37209b336E5796D632CDc", // valid
      "f1mwllxrw7frn2lwhf4u26y4f3m7f6wsl4i3o3jvi", // invalid: mainnet address on calibnet USDFC faucet
    ],
    expectSuccess: [true, false],
  },
  {
    path: "/faucet/mainnet",
    button: "Claim FIL",
    addresses: [
      "f1rgci272nfk4k6cpyejepzv4xstpejjckldlzidy", // valid
      "t1ox5dc3ifjimvn33tawpnyizikkbdikbnllyi2nq", // invalid: calibnet address on mainnet faucet
    ],
    expectSuccess: [true, false],
  },
  {
    path: "/faucet/calibnet",
    button: "Claim tFIL",
    addresses: [
      "t1pxxbe7he3c6vcw5as3gfvq33kprpmlufgtjgfdq", // valid
      "f1mwllxrw7frn2lwhf4u26y4f3m7f6wsl4i3o3jvi", // invalid: mainnet address on calibnet faucet
    ],
    expectSuccess: [true, false],
  },
];
