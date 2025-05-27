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
  const res = await page.goto(`${BASE_URL}${path}`, { timeout: 60_000 });
  check(res, { [`GET ${path} → 200`]: (r) => r && r.status() === 200 });
}

// Check if the button exists, is visible, and is enabled
async function checkButton(page, path, buttonText) {
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
  const existenceMsg = `Button "${buttonText}" on "${path}" ${exists ? "exists" : "does not exist"}`;
  check(exists, { [existenceMsg]: () => exists });
  if (!exists) {
    return;
  }

  // Check if the button is visible
  // Note: In some cases, the button might exist but not be visible
  const isVisible = await btn.isVisible();
  check(isVisible, {
    [`Button "${buttonText}" on "${path}" is visible`]: () => isVisible,
  });

  // Check if the button is enabled
  // Note: In some cases, the button might be visible but not enabled
  check(btn, {
    [`Button "${buttonText}" on "${path}" is enabled`]: () => btn.isEnabled(),
  });
}

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
    buttons: ["To faucet list"],
    links: ["Filecoin Slack", "documentation"],
  },
  {
    path: "/faucet",
    links: ["Calibration Network Faucet", "Mainnet Network Faucet"],
  },
  {
    path: "/faucet/calibnet",
    buttons: ["Back to faucet list", "Transaction History", "Send"],
  },
  {
    path: "/faucet/mainnet",
    buttons: ["Back to faucet list", "Transaction History", "Send"],
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
      await checkButton(page, path, btn);
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
