import http from 'k6/http';
import { check, sleep } from 'k6';
import {
  API_CONFIG,
  TEST_ADDRESSES,
  STATUS_CODES,
  TEST_SCENARIOS,
  FaucetTypes
} from './test_claim_token_api_config.js';

export const options = {
  vus: 1,
  iterations: 1,
  thresholds: {
    'checks': ['rate>=1.0'],
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
      const checkName = `${faucetType} - Invalid address "${invalidAddress}" properly rejected (400 and error)`;
      check(response, {
        [checkName]: (r) =>
          r.status === STATUS_CODES.BAD_REQUEST &&
          r.body &&
          r.body.toLowerCase().includes("invalid")
      });
    });
  });

  // Test all other invalid request scenarios (missing parameters, mainnet blocking, etc.)
  TEST_SCENARIOS.INVALID_REQUESTS.forEach((testCase) => {
    const response = makeClaimRequest(testCase.faucet_info, testCase.address);
    const checkName = `${testCase.name}: properly handled (${testCase.expectedStatus} + "${testCase.expectedErrorContains}")`;
    check(response, {
      [checkName]: (r) =>
        r.status === testCase.expectedStatus &&
        r.body &&
        r.body.toLowerCase().includes(testCase.expectedErrorContains.toLowerCase())
    });
  });

  console.log('✅ Input validation tests completed');
}

function testRateLimiting() {
  console.log('\n📊 Testing Faucet-Specific Rate Limiting...');
  console.log('📝 Pattern: One success per faucet → All addresses for that faucet get rate limited');

  sleep(65) // Wait in case there is already a cooldown time
  TEST_SCENARIOS.RATE_LIMIT_TEST_COOLDOWN_CASES.forEach(testCase => {
    const response = makeClaimRequest(testCase.faucet_info, testCase.address);

    const statusCheckName = `${testCase.name}: Expected ${testCase.expectedStatus}, got ${response.status}`;
    check(response, {
      [statusCheckName]: (r) => r.status === testCase.expectedStatus,
    });

    if (response.status === STATUS_CODES.SUCCESS) {
      const hashCheckName = `${testCase.name}: ✅ Valid transaction hash`;
      check(response, {
        [hashCheckName]: (r) => validateTransactionHash(r.body.trim()),
      });
    }

    // Log results for debugging
    if (response.status !== testCase.expectedStatus) {
      console.log(`❌ ${testCase.name}: Expected ${testCase.expectedStatus}, got ${response.status} - ${response.body}`);
    }

    sleep(0.5);
  });
}

export default function () {
  testInputValidation();
  testRateLimiting();
  console.log('\n✅ All tests passed successfully!');
}