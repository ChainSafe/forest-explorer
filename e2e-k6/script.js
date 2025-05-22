import { browser } from "k6/browser";
import { check } from "k6";

const BASE_URL = "http://127.0.0.1:8787";

async function checkPath(page, path) {
    const res = await page.goto(`${BASE_URL}${path}`, {
      waitUntil: "networkidle",
    });
    check(res, { [`GET ${path} → 200`]: (r) => r && r.status() === 200 });
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
    for (const path of ["/", "/faucet"]) {
      await checkPath(page, path);
    }
  } finally {
    await page.close();
  }
}

export async function testFaucet() {
  const page = await browser.newPage();
  try {
    for (const path of ["/faucet/calibnet", "/faucet/mainnet"]) {
      await checkPath(page, path);
    }
  } finally {
    await page.close();
  }
}
