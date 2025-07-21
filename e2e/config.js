// PAGES: describes the structure of each page for navigation and checks
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

// BUTTON_ACTIONS: describes what each button should do on each page
export const BUTTON_ACTIONS = {
  "/faucet/calibnet_usdfc": {
    "Faucet List": { type: "navigate" },
    "Transaction History": { type: "clickable" },
    "Claim tUSDFC": { type: "error", errorMsg: "Invalid address" },
  },
  "/faucet/calibnet": {
    "Faucet List": { type: "navigate" },
    "Transaction History": { type: "clickable" },
    "Claim tFIL": { type: "error", errorMsg: "Invalid address" },
  },
  "/faucet/mainnet": {
    "Faucet List": { type: "navigate" },
    "Transaction History": { type: "clickable" },
    "Claim FIL": { type: "error", errorMsg: "Invalid address" },
  },
};

// CLAIM_TESTS: describes claim test cases for each page
export const CLAIM_TESTS = [
  {
    path: "/faucet/calibnet_usdfc",
    button: "Claim tUSDFC",
    addresses: [
      "0xAe9C4b9508c929966ef37209b336E5796D632CDc", // valid
      "f1mwllxrw7frn2lwhf4u26y4f3m7f6wsl4i3o3jvi", // invalid
    ],
    expectSuccess: [true, false],
  },
  {
    path: "/faucet/mainnet",
    button: "Claim FIL",
    addresses: [
      "f1rgci272nfk4k6cpyejepzv4xstpejjckldlzidy", // valid
      "t1ox5dc3ifjimvn33tawpnyizikkbdikbnllyi2nq", // invalid
    ],
    expectSuccess: [true, false],
  },
  {
    path: "/faucet/calibnet",
    button: "Claim tFIL",
    addresses: [
      "t1pxxbe7he3c6vcw5as3gfvq33kprpmlufgtjgfdq", // valid
      "f1mwllxrw7frn2lwhf4u26y4f3m7f6wsl4i3o3jvi", // invalid
    ],
    expectSuccess: [true, false],
  },
];
