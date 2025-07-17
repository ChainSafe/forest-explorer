import { browser } from "k6/browser";
import { check } from "k6";

export const options = {
  scenarios: {
    ui: {
      executor: "shared-iterations",
      options: {
        browser: {
          type: "chromium",
        },
      },
    },
  },
  thresholds: {
    checks: ["rate==1.0"],
  },
};

const BASE_URL = "http://127.0.0.1:8787";

// Check if the path is reachable
async function checkPath(page, path) {
  const res = await page.goto(`${BASE_URL}${path}`, {
    timeout: 60_000,
    waitUntil: "networkidle",
  });
  check(res, { [`GET ${path} → 200`]: (r) => r && r.status() === 200 });
}

// Check if the button exists, is visible, and is enabled
async function checkButton(page, path, buttonText, action) {
  const buttons = await page.$$("button");
  let btn = null;
  for (const b of buttons) {
    const text = await b.evaluate((el) => el.textContent.trim());
    if (text === buttonText) {
      btn = b;
      break;
    }
  }

  // Check if the button exists
  const exists = btn !== null;
  check(exists, {
    [`Button "${buttonText}" on "${path}" exists`]: () => exists,
  });
  // Check if the button is visible
  // Note: In some cases, the button might exist but not be visible
  const isVisible = await btn.isVisible();
  check(isVisible, {
    [`Button "${buttonText}" on "${path}" is visible`]: () => isVisible,
  });
  // Check if the button is enabled
  // Note: In some cases, the button might be visible but not enabled
  const isEnabled = await btn.isEnabled();
  check(isEnabled, {
    [`Button "${buttonText}" on "${path}" is enabled`]: () => isEnabled,
  });
  if (!isEnabled || !action) return;

  let isClickable = false;
  let msg;
  if (action.type === "navigate") {
    const oldUrl = page.url();
    await btn.click();
    const newUrl = page.url();
    isClickable = oldUrl !== newUrl;
    msg = `Clicking "${buttonText}" on "${path}" navigated in the same tab`;
    if (isClickable) {
      await page.goto(`${BASE_URL}${path}`);
    }
  } else if (action.type === "clickable") {
    try {
      await btn.click();
      isClickable = true;
    } catch (e) {
      isClickable = false;
    }
    msg = `Clicking "${buttonText}" on "${path}" did not throw an error`;
  } else if (action.type === "error") {
    await btn.click();
    const content = await page.content();
    const errorMatch = content.match(
      new RegExp(action.errorMsg + "[^<]*", "i"),
    );
    if (errorMatch) {
      isClickable = true;
      msg = `Clicking "${buttonText}" on "${path}" shows error: ${errorMatch[0]}`;
    } else {
      isClickable = false;
      msg = `Clicking "${buttonText}" on "${path}" did not show an error message matching "${action.errorMsg}"`;
    }
  }
  check(isClickable, { [msg]: () => isClickable });
}

const BUTTON_ACTIONS = {
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

// Check if the link exists, is visible, and has a valid href
async function checkLink(page, path, linkText) {
  const links = await page.$$("a");
  let link = null;
  for (const l of links) {
    const text = await l.evaluate((el) => el.textContent.trim());
    if (text === linkText) {
      link = l;
      break;
    }
  }

  // Check if the link exists
  const exists = link !== null;
  const existenceMsg = `Link "${linkText}" on "${path}" ${exists ? "exists" : "does not exist"}`;
  check(exists, { [existenceMsg]: () => exists });
  if (!exists) {
    return;
  }

  // Check if the link is visible
  // Note: In some cases, the link might exist but not be visible
  const isVisible = await link.isVisible();
  check(isVisible, {
    [`Link "${linkText}" on "${path}" is visible`]: () => isVisible,
  });

  // Check if the link is enabled
  // Note: In some cases, the link might be visible but not enabled
  const href = await link.evaluate((el) => el.getAttribute("href"));
  const hasHref = Boolean(href && href.trim());
  check(hasHref, {
    [`Link "${linkText}" on "${path}" has valid href`]: () => hasHref,
  });
}

async function checkFooter(page, path) {
  await checkLink(page, path, "Forest Explorer");
  await checkLink(page, path, "ChainSafe Systems");
}

const PAGES = [
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

// Loops through each page config, performing:
// - checkPath
// - checkButton
// - checkLink
// - checkFooter
async function runChecks(page) {
  for (const { path, buttons = [], links = [] } of PAGES) {
    await checkPath(page, path);
    for (const btn of buttons) {
      const action = BUTTON_ACTIONS[path]?.[btn];
      await checkButton(page, path, btn, action);
    }
    for (const lnk of links) {
      await checkLink(page, path, lnk);
    }
    await checkFooter(page, path);
  }
}

const CLAIM_TESTS = [
  {
    path: "/faucet/calibnet",
    button: "Claim tFIL",
    addresses: [
      "t1ox5dc3ifjimvn33tawpnyizikkbdikbnllyi2nq", // valid
      "f1mwllxrw7frn2lwhf4u26y4f3m7f6wsl4i3o3jvix", // invalid
    ],
    expectSuccess: [true, false],
  },
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
      "f1mwllxrw7frn2lwhf4u26y4f3m7f6wsl4i3o3jvi", // valid
      "t1ox5dc3ifjimvn33tawpnyizikkbdikbnllyi2nq", // invalid
    ],
    expectSuccess: [true, false],
  },
];

async function runClaimTests(page, { path, button, addresses, expectSuccess }) {
  await checkPath(page, path);
  for (let i = 0; i < addresses.length; i++) {
    const address = addresses[i];
    const shouldSucceed = Array.isArray(expectSuccess)
      ? expectSuccess[i]
      : expectSuccess;
    const input = await page.$("input.input");
    check(!!input, { [`Input exists on ${path}`]: () => !!input });
    if (!input) continue;
    await input.click({ clickCount: 3 });
    await input.press("Backspace");
    await input.type(address);
    const buttons = await page.$$("button");
    let claimBtn = null;
    for (const b of buttons) {
      const text = await b.evaluate((el) => el.textContent.trim());
      if (text === button) {
        claimBtn = b;
        break;
      }
    }
    check(!!claimBtn, { [`Claim button exists on ${path}`]: () => !!claimBtn });
    if (!claimBtn) continue;
    await claimBtn.click();
    let found = false;
    if (shouldSucceed) {
      await page.waitForTimeout(500);
      const txContainer = await page.$(".transaction-container");
      if (txContainer) found = true;
      check(found, {
        [`Claim success for '${address}' on ${path}`]: () => found,
      });
    } else {
      await page.waitForTimeout(250);
      const content = await page.content();
      if (/Invalid address/i.test(content)) {
        found = true;
      }
      check(found, {
        [`Invalid address error for '${address}' on ${path}`]: () => found,
      });
    }
  }
}

export default async function () {
  const page = await browser.newPage();
  try {
    await runChecks(page);
    for (const test of CLAIM_TESTS) {
      await runClaimTests(page, test);
    }
  } finally {
    await page.close();
  }
}
