import { browser } from 'k6/browser';
import { check } from 'k6';

export const options = {
  scenarios: {
    ui: {
      executor: 'shared-iterations',
      options: {
        browser: {
          type: 'chromium',
        },
      },
    },
  },
  thresholds: {
    checks: ['rate==1.0'],
  },
};

export default async function () {
  const page = await browser.newPage();
  const baseUrl = 'http://127.0.0.1:8787';
  const paths = ['/', '/faucet', '/faucet/mainnet', '/faucet/calibnet'];

  try {
    for (const path of paths) {
      const res = await page.goto(`${baseUrl}${path}`, {
        waitUntil: 'networkidle',
      });

      check(res, {
        [`GET ${path} → 200`]: (r) => r !== null && r.status() === 200,
      });
    }
  } finally {
    await page.close();
  }
}
