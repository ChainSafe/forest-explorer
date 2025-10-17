import http from 'k6/http';
import { check } from 'k6';
import { FaucetTypes, TEST_ADDRESSES } from "./test_claim_token_api_config.js";

export const options = {
  vus: 1,
  iterations: 1,
  thresholds: {
    'checks': ['rate>=1.0'],
  },
};

const API_URL = __ENV.API_URL || 'http://127.0.0.1:8787';
const CLAIM_TOKEN_ENDPOINT = `${API_URL}/api/claim_token`;

/**
 * Test CORS Configuration for the Claim Token API
 * 
 * This test validates that the API properly handles Cross-Origin Resource Sharing (CORS)
 * by checking the appropriate headers in both preflight (OPTIONS) and actual requests.
 */
export default function () {
  // Test 1: Preflight OPTIONS Request
  const preflightResponse = http.options(CLAIM_TOKEN_ENDPOINT, null, {
    headers: {
      'Origin': 'https://external-example.com',
      'Access-Control-Request-Method': 'GET',
    },
  });

  check(preflightResponse, {
    'Preflight: Status is 200 or 204': (r) => r.status === 200 || r.status === 204,
    'Preflight: Has Access-Control-Allow-Origin header': (r) =>
      r.headers['Access-Control-Allow-Origin'] !== undefined,
    'Preflight: Allows all origins (*)': (r) =>
      r.headers['Access-Control-Allow-Origin'] === '*',
    'Preflight: Has Access-Control-Allow-Methods': (r) =>
      r.headers['Access-Control-Allow-Methods'] !== undefined,
    'Preflight: Allows GET method': (r) => {
      const methods = r.headers['Access-Control-Allow-Methods'] || '';
      return methods.toUpperCase().includes('GET');
    },
  });

  // Test 2: Actual GET Request with Origin Header
  const url = `${CLAIM_TOKEN_ENDPOINT}?faucet_info=${FaucetTypes.CalibnetFIL}&address=${TEST_ADDRESSES.ETH_FORMAT_ADDRESS}`;

  const actualResponse = http.get(url, {
    headers: {
      'Origin': 'https://external-example.com',
    },
  });

  check(actualResponse, {
    'Actual Request: Has Access-Control-Allow-Origin in response': (r) =>
      r.headers['Access-Control-Allow-Origin'] !== undefined,
    'Actual Request: CORS header allows all origins': (r) =>
      r.headers['Access-Control-Allow-Origin'] === '*',
    'Actual Request: Response received (not blocked by CORS)': (r) =>
      r.status !== 0 && r.body !== '',
  });

  // Test 3: Request WITHOUT Origin (same-origin simulation)
  const sameOriginResponse = http.get(url);

  check(sameOriginResponse, {
    'Same-Origin: Request succeeds': (r) => r.status === 200 || r.status === 429,
    'Same-Origin: Has Access-Control-Allow-Origin (even for same-origin)': (r) =>
      r.headers['Access-Control-Allow-Origin'] !== undefined,
  });

  // Test 4: Multiple Origins Test
  const origins = [
    'https://app.example.com',
    'http://localhost:3000',
    'https://wallet.filecoin.io',
  ];

  origins.forEach(origin => {
    const response = http.get(url, {
      headers: { 'Origin': origin },
    });

    check(response, {
      [`Multiple Origins: ${origin} - Request succeeds`]: (r) =>
        r.status === 200 || r.status === 429,
      [`Multiple Origins: ${origin} - CORS allows origin`]: (r) =>
        r.headers['Access-Control-Allow-Origin'] === '*',
      [`Multiple Origins: ${origin} - Response received`]: (r) =>
        r.status !== 0 && r.body !== '',
    });
  });

  // Test 5: Check for Security Headers
  const securityResponse = http.get(url, {
    headers: { 'Origin': 'https://external-example.com' },
  });

  check(securityResponse, {
    'Security: Has Access-Control-Allow-Origin': (r) =>
      r.headers['Access-Control-Allow-Origin'] !== undefined,
    'Security: Access-Control-Allow-Credentials is not set (safer for public APIs)': (r) =>
      r.headers['Access-Control-Allow-Credentials'] === undefined ||
      r.headers['Access-Control-Allow-Credentials'] === 'false',
  });

  // Test 6: Edge Case - CORS headers should be present even on error responses
  // This is critical - even 500 errors should have CORS headers, so browser apps can read the error
  const errorResponse = http.get(`${CLAIM_TOKEN_ENDPOINT}?invalid=params`, {
    headers: { 'Origin': 'https://external-example.com' },
  });

  check(errorResponse, {
    'Error Response: CORS headers present even on errors': (r) =>
      r.headers['Access-Control-Allow-Origin'] !== undefined,
  });
}
