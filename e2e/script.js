import { browser } from "k6/browser";
import { check } from "k6";
import { BUTTON_ACTIONS, PAGES, CLAIM_TESTS } from "./config.js";

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

const BASE_URL = __ENV.API_URL ||"http://127.0.0.1:8787";

// Check if the path is reachable
async function checkPath(page, path) {
  const res = await page.goto(`${BASE_URL}${path}`, {
    timeout: 60_000,
    waitUntil: "networkidle",
  });
  check(res, { [`GET ${path} → 200`]: (r) => r && r.status() === 200 });
}

async function handleNavigateAction(page, path, btn, buttonText) {
  const oldUrl = page.url();
  await btn.click();
  await page.waitForTimeout(500);
  const newUrl = page.url();
  const isWorking = oldUrl !== newUrl;
  let msg = `Clicking "${buttonText}" on "${path}" navigated from "${oldUrl}" to "${newUrl}"`;
  if (isWorking) {
    await page.goto(`${BASE_URL}${path}`, {
      timeout: 60_000,
      waitUntil: "networkidle",
    });
  }
  check(isWorking, { [msg]: () => isWorking });
}

async function handleClickableAction(page, path, btn, buttonText) {
  let isWorking = false;
  let msg;
  try {
    await btn.click();
    isWorking = true;
  } catch (e) {
    isWorking = false;
  }
  msg = `Clicking "${buttonText}" on "${path}" did not throw an error`;
  check(isWorking, { [msg]: () => isWorking });
}

async function handleExpectErrorAction(page, path, btn, buttonText, errorMsg) {
  let isWorking = false;
  let msg;
  await btn.click();
  const content = await page.content();
  const errorMatch = content.match(
    new RegExp(errorMsg + "[^<]*", "i"),
  );
  if (errorMatch) {
    isWorking = true;
    msg = `Clicking "${buttonText}" on "${path}" shows error: ${errorMatch[0]}`;
  } else {
    isWorking = false;
    msg = `Clicking "${buttonText}" on "${path}" did not show an error message matching "${errorMsg}"`;
  }
  check(isWorking, { [msg]: () => isWorking });
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
  if (!exists) return;

  // Check if the button is visible
  const isVisible = await btn.isVisible();
  check(isVisible, {
    [`Button "${buttonText}" on "${path}" is visible`]: () => isVisible,
  });
  if (!isVisible) return;

  // Check if the button is enabled
  const isEnabled = await btn.isEnabled();
  check(isEnabled, {
    [`Button "${buttonText}" on "${path}" is enabled`]: () => isEnabled,
  });
  if (!isEnabled || !action) return;

  if (action.type === "navigate") {
    await handleNavigateAction(page, path, btn, buttonText);
  } else if (action.type === "clickable") {
    await handleClickableAction(page, path, btn, buttonText);
  } else if (action.type === "expectError") {
    await handleExpectErrorAction(page, path, btn, buttonText, action.errorMsg);
  }
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

// Checks for required footer links on each page
async function checkFooter(page, path) {
  await checkLink(page, path, "Forest Explorer");
  await checkLink(page, path, "ChainSafe Systems");
}

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
      if (text.trim() === button) {
        claimBtn = b;
        break;
      }
    }
    check(!!claimBtn, { [`Claim button exists on ${path}`]: () => !!claimBtn });
    if (!claimBtn) continue;
    await claimBtn.click();
    await page.waitForTimeout(500);
    let found = false;
    if (shouldSucceed) {
      try {
        const txContainer = await page.waitForSelector(".transaction-container", { timeout: 5000 });
        if (txContainer) found = true;
      } catch (e) {
        found = false;
      }
      check(found, {
        [`Claim success for '${address}' on ${path}`]: () => found,
      });
    } else {
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
