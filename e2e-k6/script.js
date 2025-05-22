import { browser } from "k6/browser";
import { check } from "k6";

const BASE_URL = "http://127.0.0.1:8787";

// Check if the path is reachable
async function checkPath(page, path) {
  const res = await page.goto(`${BASE_URL}${path}`, {
    waitUntil: "networkidle",
  });
  check(res, { [`GET ${path} → 200`]: (r) => r && r.status() === 200 });
}

// Check if the button exists, is visible, and is enabled
async function checkButton(page, path, buttonText) {
  await page.goto(`${BASE_URL}${path}`, { waitUntil: 'networkidle' });
  const buttons = await page.$$('button');
  let btn = null;
  for (const b of buttons) {
    const text = await b.evaluate(el => el.textContent.trim());
    if (text === buttonText) {
      btn = b;
      break;
    }
  }

  // Check if the button exists
  const exists = btn !== null;
  const existenceMsg = `Button "${buttonText}" on "${path}" ${exists ? 'exists' : 'does not exist'}`;
  check(exists, { [existenceMsg]: () => exists });
  if (!exists) {
    return;
  }

  // Check if the button is visible
  // Note: In some cases, the button might exist but not be visible
  const isVisible = await btn.isVisible();
  check(isVisible, { [`Button "${buttonText}" on "${path}" is visible`]: () => isVisible });

  // Check if the button is enabled
  // Note: In some cases, the button might be visible but not enabled
  const isEnabled = await btn.isEnabled();
  check(isEnabled, { [`Button "${buttonText}" on "${path}" is enabled`]: () => isEnabled });
}

export const options = {
  scenarios: {
    home: {
      executor: "shared-iterations",
      exec: "testHome",
      options: { browser: { type: "chromium" } },
    },
    faucet: {
      executor: "shared-iterations",
      exec: "testFaucet",
      options: { browser: { type: "chromium" } },
    },
  },
  thresholds: {
    checks: ["rate==1.0"],
  },
};

export async function testHome() {
  const page = await browser.newPage();
  try {
    await checkPath(page, "/");
    await checkButton(page, "/", "To faucet list");
    await checkPath(page, "/faucet");
  } finally {
    await page.close();
  }
}

export async function testFaucet() {
  const page = await browser.newPage();
  try {
    for (const path of ["/faucet/calibnet", "/faucet/mainnet"]) {
      await checkPath(page, path);
      await checkButton(page, path, "Send");
      await checkButton(page, path, "Back to faucet list");
      await checkButton(page, path, "Transaction History");
    }
  } finally {
    await page.close();
  }
}
