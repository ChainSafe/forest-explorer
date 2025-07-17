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
  const res = await page.goto(`${BASE_URL}${path}`, { timeout: 60_000, waitUntil: "networkidle" });
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
  check(exists, { [`Button "${buttonText}" on "${path}" exists`]: () => exists });
  // Check if the button is visible
  // Note: In some cases, the button might exist but not be visible
  const isVisible = await btn.isVisible();
  check(isVisible, { [`Button "${buttonText}" on "${path}" is visible`]: () => isVisible });
  // Check if the button is enabled
  // Note: In some cases, the button might be visible but not enabled
  const isEnabled = await btn.isEnabled();
  check(isEnabled, { [`Button "${buttonText}" on "${path}" is enabled`]: () => isEnabled });
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
    const errorMatch = content.match(new RegExp(action.errorMsg + "[^<]*", "i"));
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
    "Claim tUSDFC": { type: "error", errorMsg: "Invalid address" }
  },
  "/faucet/calibnet": {
    "Faucet List": { type: "navigate" },
    "Transaction History": { type: "clickable" },
    "Claim tFIL": { type: "error", errorMsg: "Invalid address" }
  },
  "/faucet/mainnet": {
    "Faucet List": { type: "navigate" },
    "Transaction History": { type: "clickable" },
    "Claim FIL": { type: "error", errorMsg: "Invalid address" }
  }
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
    links: ["💰 Calibration Network USDFC Faucet", "🧪 Calibration Network Faucet", "🌐 Mainnet Network Faucet"],
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

export default async function () {
  const page = await browser.newPage();
  try {
    await runChecks(page);
  } finally {
    await page.close();
  }
}
