import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';
import {
  API_CONFIG,
  TEST_ADDRESSES,
  STATUS_CODES,
  TEST_SCENARIOS,
  FaucetTypes
} from './test_claim_token_api_config.js';

// Custom metrics
const errorRate = new Rate('errors');

export const options = {
  vus: 1,
  iterations: 1,
  thresholds: {
    'checks': ['rate>=1.0'],           // 100% of checks MUST pass
    'http_req_failed': ['rate<=0.0'],  // 0% HTTP failures allowed
    'http_req_duration': ['p(95)<5000'],
  },
};

function validateTransactionHash(txHash) {
  // Remove outer quotes if present
  if (txHash.startsWith('"') && txHash.endsWith('"')) {
    txHash = txHash.slice(1, -1);
  }
  // Remove inner quotes if present
  if (txHash.startsWith('"') && txHash.endsWith('"')) {
    txHash = txHash.slice(1, -1);
  }

  // Both CalibnetFIL and CalibnetUSDFC now return Ethereum format: 0x + 64 hex chars = 66 total
  return txHash.startsWith('0x') && txHash.length === 66;
}

function makeClaimRequest(faucetInfo, address) {
  let url = `${API_CONFIG.BASE_URL}${API_CONFIG.ENDPOINT}`;
  const params = [];

  if (faucetInfo !== null && faucetInfo !== undefined) {
    params.push(`faucet_info=${encodeURIComponent(faucetInfo)}`);
  }
  if (address !== null && address !== undefined) {
    params.push(`address=${encodeURIComponent(address)}`);
  }

  if (params.length > 0) {
    url += `?${params.join('&')}`;
  }

  return http.get(url, {
    timeout: API_CONFIG.REQUEST_TIMEOUT,
    tags: { faucet_type: faucetInfo || 'unknown' }
  });
}

// Test input validation
function testInputValidation() {
  console.log('🧪 Starting input validation tests...');

  // Test invalid addresses for both faucet types
  const faucetTypes = [FaucetTypes.CalibnetFIL, FaucetTypes.CalibnetUSDFC];

  faucetTypes.forEach((faucetType) => {
    TEST_ADDRESSES.INVALID.forEach((invalidAddress, index) => {
      const response = makeClaimRequest(faucetType, invalidAddress);
      check(response, {
        [`${faucetType} - Invalid address "${invalidAddress}" properly rejected (400 and error)`]: (r) =>
          r.status === STATUS_CODES.BAD_REQUEST &&
          r.body &&
          r.body.toLowerCase().includes("invalid")
      }) || errorRate.add(1);
    });
  });

  // Test all other invalid request scenarios (missing parameters, mainnet blocking, etc.)
  TEST_SCENARIOS.INVALID_REQUESTS.forEach((testCase) => {
    const response = makeClaimRequest(testCase.faucet_info, testCase.address);
    check(response, {
      [`${testCase.name}: properly handled (${testCase.expectedStatus} + "${testCase.expectedErrorContains}")`]: (r) =>
        r.status === testCase.expectedStatus &&
        r.body &&
        r.body.toLowerCase().includes(testCase.expectedErrorContains.toLowerCase())
    }) || errorRate.add(1);
  });

  console.log('✅ Input validation tests completed');
}

function testRateLimiting() {
  console.log('\n📊 Testing Faucet-Specific Rate Limiting...');
  console.log('📝 Pattern: One success per faucet → All addresses for that faucet get rate limited');

  TEST_SCENARIOS.RATE_LIMIT_TEST_COOLDOWN_CASES.forEach(testCase => {
    const response = makeClaimRequest(testCase.faucet_info, testCase.address);

    check(response, {
      [`${testCase.name}: Expected ${testCase.expectedStatus}, got ${response.status}`]: (r) =>
        r.status === testCase.expectedStatus,
    }) || errorRate.add(1);

    if (response.status === STATUS_CODES.SUCCESS) {
      check(response, {
        [`${testCase.name}: ✅ Valid transaction hash`]: (r) =>
          validateTransactionHash(r.body.trim()),
      }) || errorRate.add(1);
    }

    // Log results for debugging
    if (response.status !== testCase.expectedStatus) {
      console.log(`❌ ${testCase.name}: Expected ${testCase.expectedStatus}, got ${response.status} - ${response.body}`);
    }

    sleep(0.1);
  });
}

export default function () {
  testInputValidation();
  testRateLimiting();
}