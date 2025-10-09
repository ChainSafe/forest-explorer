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
  txHash = txHash.replace(/^"|"$/g, '');
  // Both CalibnetFIL and CalibnetUSDFC now return an Ethereum format: 0x + 64 hex chars = 66 total
  return txHash.startsWith('0x') && txHash.length === 66;
}

function runTestScenarios(scenarios, options = {}) {
  const { 
    sleepBetween = 0, 
    allowWaiting = false,
    additionalChecks = null 
  } = options;

  scenarios.forEach(testCase => {
    if (allowWaiting && testCase.waitBefore && testCase.waitBefore > 0) {
      console.log(`  ...waiting ${testCase.waitBefore}s before next test...`);
      sleep(testCase.waitBefore);
    }

    const response = makeClaimRequest(testCase.faucet_info, testCase.address);

    const commonChecks = {
      [`${testCase.name}: Expected status ${testCase.expectedStatus}`]: (r) => 
        r.status === testCase.expectedStatus,
      [`${testCase.name}: Valid transaction hash (if success)`]: (r) => 
        r.status !== STATUS_CODES.SUCCESS || validateTransactionHash(r.body.trim())
    };

    // Add any additional checks specific to the test type
    const allChecks = additionalChecks 
      ? { ...commonChecks, ...additionalChecks(testCase) }
      : commonChecks;

    check(response, allChecks);

    if (response.status !== testCase.expectedStatus) {
      console.log(`❌ ${testCase.name}: Expected ${testCase.expectedStatus}, got ${response.status} - ${response.body}`);
    }

    if (sleepBetween > 0) {
      sleep(sleepBetween);
    }
  });
}

function validateServerConnectivity() {
  console.log('🔗 Checking server connectivity...');
  
  const healthResponse = http.get(API_CONFIG.BASE_URL, {
    timeout: API_CONFIG.CONNECTION_TIMEOUT,
    tags: { test_type: 'connectivity' }
  });

  const connectivityChecks = check(healthResponse, {
    'Server connectivity: Can reach base URL': (r) => r.status !== 0 && !r.error,
    'Server connectivity: Response time < 10s': (r) => r.timings.duration < 10000,
    'Server connectivity: No network errors': (r) => !r.error
  });

  if (!connectivityChecks) {
    console.error(`❌ Server connectivity failed: ${healthResponse.body || healthResponse.error || 'Unknown error'}`);
    return false;
  }

  console.log('✅ Server connectivity confirmed');
  return true;
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

  const response = http.get(url, {
    timeout: API_CONFIG.REQUEST_TIMEOUT,
    tags: { faucet_type: faucetInfo || 'unknown' }
  });

  const requestDescriptor = `${faucetInfo || 'unknown'} request to ${address ? address.substring(0, 10) + '...' : 'null'}`;
  check(response, {
    [`Network: ${requestDescriptor} - Response received`]: (r) => r.status !== 0,
    [`Network: ${requestDescriptor} - No errors`]: (r) => !r.error,
    [`Network: ${requestDescriptor} - Within timeout`]: (r) => r.timings.duration < API_CONFIG.MAX_RESPONSE_TIME
  });

  return response;
}

// Test input validation
function testInputValidation() {
  console.log('🧪 Starting input validation tests...');

  // Test invalid addresses for both faucet types
  const faucetTypes = [FaucetTypes.CalibnetFIL, FaucetTypes.CalibnetUSDFC];

  faucetTypes.forEach((faucetType) => {
    TEST_ADDRESSES.INVALID.forEach((invalidAddress, index) => {
      const response = makeClaimRequest(faucetType, invalidAddress);
      const testName = `${faucetType} - Invalid address "${invalidAddress}"`;

      check(response, {
        [`${testName}: Proper rejection (400)`]: (r) => r.status === STATUS_CODES.BAD_REQUEST,
        [`${testName}: Error message contains 'invalid'`]: (r) => 
          r.body && r.body.toLowerCase().includes("invalid")
      });
    });
  });

  // Test all other invalid request scenarios (missing parameters, mainnet blocking, etc.)
  TEST_SCENARIOS.INVALID_REQUESTS.forEach((testCase) => {
    const response = makeClaimRequest(testCase.faucet_info, testCase.address);

    check(response, {
      [`${testCase.name}: Expected status ${testCase.expectedStatus}`]: (r) => 
        r.status === testCase.expectedStatus,
      [`${testCase.name}: Contains expected error "${testCase.expectedErrorContains}"`]: (r) =>
        r.body && r.body.toLowerCase().includes(testCase.expectedErrorContains.toLowerCase())
    });
  });

  console.log('✅ Input validation tests completed');
}

function testRateLimiting() {
  console.log('\n📊 Testing Faucet-Specific Rate Limiting...');
  console.log('📝 Pattern: One success per faucet → All addresses for that faucet get rate limited');

  runTestScenarios(TEST_SCENARIOS.RATE_LIMIT_TEST_COOLDOWN_CASES, {
    sleepBetween: 0.5
  });
}

function testWalletCap() {
  console.log('\n💰 Testing Wallet Cap Limits (2 drips per wallet)...');

  const walletCapChecks = (testCase) => ({
    [`${testCase.name}: Wallet cap retry time >1h (if capped)`]: (r) => {
      if (!testCase.walletCapErrorResponse || r.status !== STATUS_CODES.TOO_MANY_REQUESTS) {
        return true;
      }
      const retrySeconds = parseInt((r.body.match(/(\d+)/) || [null, 0])[1]);
      return retrySeconds > 3600;
    }
  });

  runTestScenarios(TEST_SCENARIOS.RATE_LIMIT_TEST_WALLET_CAP_CASES, {
    allowWaiting: true,
    additionalChecks: walletCapChecks
  });
}

export default function () {
  // Validate server connectivity using k6 checks
  const connectivityResult = validateServerConnectivity();

  check(null, {
    'Pre-flight: Server connectivity successful': () => connectivityResult
  });

  if (!connectivityResult) {
    console.error('❌ Server connectivity failed - aborting tests');
    return;
  }

  testInputValidation();
  console.log(`⏰ Waiting ${API_CONFIG.FAUCET_COOLDOWN_BUFFER_SECONDS} seconds to ensure previous global faucet cooldowns have expired...`);
  sleep(API_CONFIG.FAUCET_COOLDOWN_BUFFER_SECONDS);
  testRateLimiting();
  testWalletCap();
  console.log('\n✅ All tests passed successfully!');
}