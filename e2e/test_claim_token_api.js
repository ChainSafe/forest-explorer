// k6 Faucet API Testing Script
import http from 'k6/http';
import { check, group, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const BASE_URL = __ENV.BASE_URL || 'http://127.0.0.1:8787';
const API_ENDPOINT = `${BASE_URL}/api/claim_token`;
const CALIBNET_RPC = __ENV.FAUCET_TX_URL_CALIBNET || 'https://api.calibration.node.glif.io';

const TEST_ADDRESSES = {
  FIL_FORMAT_ADDRESS: 't1pxxbe7he3c6vcw5as3gfvq33kprpmlufgtjgfdq',
  ETH_FORMAT_ADDRESS: '0xAe9C4b9508c929966ef37209b336E5796D632CDc',
  INVALID_ADDRESS: 'invalid_address_123',
  MAINNET_ADDRESS: 'f1mwllxrw7frn2lwhf4u26y4f3m7f6wsl4i3o3jvi',
};

export const options = {
  vus: 1,
  iterations: 1,
  maxDuration: '6m',
  thresholds: {
    'checks': ['rate==1.0'],
    'test_run_success_rate': ['rate==1.0'],
  },
};

const testRunSuccessRate = new Rate('test_run_success_rate');

const API_TESTS = [
  {
    name: 'Invalid Faucet Type',
    data: `faucet_info=InvalidFaucet&address=${TEST_ADDRESSES.FIL_FORMAT_ADDRESS}`,
    expectPattern: 'unknown variant',
    expectSuccess: false,
    verifyOnChain: false,
  },
  {
    name: 'Typo in Faucet Type',
    data: `faucet_info=CalibnettFIL&address=${TEST_ADDRESSES.FIL_FORMAT_ADDRESS}`,
    expectPattern: 'unknown variant',
    expectSuccess: false,
    verifyOnChain: false,
  },
  {
    name: 'Invalid Address Format',
    data: `faucet_info=CalibnetFIL&address=${TEST_ADDRESSES.INVALID_ADDRESS}`,
    expectPattern: 'ServerError|Not a valid Testnet address',
    expectSuccess: false,
    verifyOnChain: false,
  },
  {
    name: 'Empty Address',
    data: 'faucet_info=CalibnetFIL&address=',
    expectPattern: 'ServerError|Not a valid Testnet address',
    expectSuccess: false,
    verifyOnChain: false,
  },
  {
    name: 'Missing Address Parameter',
    data: 'faucet_info=CalibnetFIL',
    expectPattern: 'Args|missing field',
    expectSuccess: false,
    verifyOnChain: false,
  },
  {
    name: 'Missing Faucet Info Parameter',
    data: `address=${TEST_ADDRESSES.FIL_FORMAT_ADDRESS}`,
    expectPattern: 'Args|missing field',
    expectSuccess: false,
    verifyOnChain: false,
  },
  {
    name: 'Mainnet FIL Request (Security) - invalid address for testnet',
    data: `faucet_info=CalibnetFIL&address=${TEST_ADDRESSES.MAINNET_ADDRESS}`,
    expectPattern: 'ServerError|Not a valid Testnet address',
    expectSuccess: false,
    verifyOnChain: false,
  },
  {
    name: 'Mainnet FIL Request (Security) - invalid faucet type',
    data: `faucet_info=MainnetFIL&address=${TEST_ADDRESSES.FIL_FORMAT_ADDRESS}`,
    expectPattern: 'ServerError|Mainnet',
  },
];

const RATE_LIMIT_TESTS = [
  {
    name: 'Rate Limit Test: CalibnetFIL First Request (t1 format) - should succeed',
    data: `faucet_info=CalibnetFIL&address=${TEST_ADDRESSES.FIL_FORMAT_ADDRESS}`,
    expectSuccess: true,
    verifyOnChain: true,
    waitTime: 2,
  },
  {
    name: 'Rate Limit Test: CalibnetFIL Consecutive Request (t1 format) - should be rate limited',
    data: `faucet_info=CalibnetFIL&address=${TEST_ADDRESSES.FIL_FORMAT_ADDRESS}`,
    expectPattern: 'ServerError|Rate limited. Try again',
    verifyOnChain: false,
    waitTime: 62,
  },
  {
    name: 'Rate Limit Test: CalibnetFIL First Request (0x format) - should succeed',
    data: `faucet_info=CalibnetFIL&address=${TEST_ADDRESSES.ETH_FORMAT_ADDRESS}`,
    expectSuccess: true,
    verifyOnChain: true,
    waitTime: 2,
  },
  {
    name: 'Rate Limit Test: CalibnetFIL Consecutive Request (0x format) - should be rate limited',
    data: `faucet_info=CalibnetFIL&address=${TEST_ADDRESSES.ETH_FORMAT_ADDRESS}`,
    expectPattern: 'ServerError|Rate limited.',
    expectSuccess: false,
    verifyOnChain: false,
    waitTime: 62,
  },
  {
    name: 'Rate Limit Test: CalibnetUSDFC First Request (0x format) - should succeed',
    data: `faucet_info=CalibnetUSDFC&address=${TEST_ADDRESSES.ETH_FORMAT_ADDRESS}`,
    expectSuccess: true,
    verifyOnChain: true,
    waitTime: 2,
  },
  {
    name: 'Rate Limit Test: CalibnetUSDFC Consecutive Request (0x format) - should be rate limited',
    data: `faucet_info=CalibnetUSDFC&address=${TEST_ADDRESSES.ETH_FORMAT_ADDRESS}`,
    expectPattern: 'ServerError|Rate limited. Try again ',
    expectSuccess: false,
    verifyOnChain: false,
    waitTime: 2,
  },
];

function extractTransactionId(responseBody) {
  const cleanResponse = responseBody.replace(/"/g, '').replace(/%$/, '').trim();

  if (cleanResponse.length < 10) {
    return null;
  }

  if (cleanResponse.startsWith('0x')) {
    return { type: 'ethereum', id: cleanResponse };
  }
  return { type: 'filecoin', id: cleanResponse };
}

function verifyFilecoinTransaction(cid) {
  const rpcRequest = {
    jsonrpc: '2.0',
    method: 'Filecoin.StateSearchMsg',
    params: [null, { '/': cid }, 10, false],
    id: 0,
  };

  const params = {
    headers: { 'Content-Type': 'application/json' },
  };

  try {
    const res = http.post(CALIBNET_RPC, JSON.stringify(rpcRequest), params);

    if (res.status !== 200) {
      console.error(`   RPC Error: ${res.status} - ${res.body}`);
      return 'failed';
    }

    const result = res.json();
    if (result.result === null) {
      console.log(`   🟡 Transaction not yet confirmed (CID: ${cid})`);
      return 'pending';
    }

    if (result.result) {
      return 'confirmed';
    }
  } catch (e) {
    console.error(`   Transaction verification failed: ${e.message}`);
    return 'failed';
  }
  return 'failed';
}

function verifyEthereumTransaction(txHash) {
  const rpcRequest = {
    jsonrpc: '2.0',
    method: 'eth_getTransactionReceipt',
    params: [txHash],
    id: 0,
  };

  const params = {
    headers: { 'Content-Type': 'application/json' },
  };

  try {
    const res = http.post(CALIBNET_RPC, JSON.stringify(rpcRequest), params);

    if (res.status !== 200) {
      console.error(`   RPC Error: ${res.status} - ${res.body}`);
      return 'failed';
    }

    const result = res.json();
    if (result.result === null) {
      console.log(`   🟡 Transaction not yet confirmed (TX: ${txHash})`);
      return 'pending';
    }

    if (result.result && result.result.blockNumber && result.result.status === '0x1') {
      return 'confirmed';
    } else if (result.result && result.result.blockNumber && result.result.status === '0x0') {
      console.error(`   Transaction failed on-chain (TX: ${txHash})`);
      return 'failed';
    }
  } catch (e) {
    console.error(`   Transaction verification failed: ${e.message}`);
    return 'failed';
  }
  return 'failed';
}

function verifyTransaction(transaction) {
  if (transaction.type === 'filecoin') {
    return verifyFilecoinTransaction(transaction.id);
  } else if (transaction.type === 'ethereum') {
    return verifyEthereumTransaction(transaction.id);
  } else {
    console.error(`   Unknown transaction type: ${transaction.type}`);
    return 'failed';
  }
}

function runTestSuite(tests, suiteName) {
  console.log(`\n🧪 Starting ${suiteName}...`);

  for (const test of tests) {
    group(`🧪 ${test.name}`, () => {
      console.log(`Running: ${test.name}`);

      const params = {
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      };

      console.log(`   Request: ${test.data}`);
      const res = http.post(API_ENDPOINT, test.data, params);
      console.log(`   Response: ${res.body}`);

      let testPassed = false;

      if (test.expectSuccess) {
        const isSuccess = check(res, {
          'API call successful (status 200)': (r) => r.status === 200,
        });

        if (isSuccess) {
          const transactionId = extractTransactionId(res.body);
          const hasTxId = check(transactionId, {
            'Response contains a valid transaction ID': (id) => id !== null,
          });

          if (hasTxId) {
            console.log(`   📋 Transaction ID: ${transactionId.id} (${transactionId.type})`);

            if (test.verifyOnChain) {
              console.log(`   🔍 Verifying transaction on ${transactionId.type} network...`);
              const verificationResult = verifyTransaction(transactionId);

              const isVerified = check(verificationResult, {
                'On-chain verification is confirmed or pending': (v) => v === 'confirmed' || v === 'pending',
              });

              if (verificationResult === 'pending') {
                console.log(`   🟡 Transaction submitted successfully (pending confirmation)`);
                console.log('   ✅ PASS: Transaction submitted successfully');
              } else if (verificationResult === 'confirmed') {
                console.log(`   ✅ On-chain verification: CONFIRMED`);
                console.log('   ✅ PASS: Transaction confirmed on-chain');
              } else {
                console.log(`   ❌ On-chain verification: FAILED`);
                console.log('   ❌ FAIL: Transaction verification failed');
              }

              testPassed = isVerified;
            } else {
              testPassed = hasTxId;
              console.log('   ✅ PASS: Valid transaction ID returned');
            }
          } else {
            console.log(`   ❌ FAIL: No valid transaction ID found in response`);
          }
        } else {
          // API call failed when we expected success
          console.log(`   ❌ FAIL: Expected success but API call failed (status ${res.status}): ${res.body}`);
          testPassed = false;
        }
      } else {
        // This is an expected failure case
        const patternRegex = new RegExp(test.expectPattern);
        testPassed = check(res, {
          [`Response body contains expected error pattern "${test.expectPattern}"`]: (r) => patternRegex.test(r.body),
        });

        if (!testPassed) {
          console.log(`   ❌ FAIL: Expected pattern "${test.expectPattern}", got: "${res.body}"`);
        } else {
          console.log(`   ✅ PASS: Found expected error pattern "${test.expectPattern}"`);
        }
      }

      testRunSuccessRate.add(testPassed);
    });

    const waitTime = test.waitTime || 0.1;
    if (waitTime > 1) {
      console.log(`   ⏰ Waiting ${waitTime} seconds before next test (rate limiting)...`);
    }
    sleep(waitTime);
  }
}

export default function () {
  console.log(`🚀 Starting Faucet API Tests`);
  console.log(`   API Endpoint: ${API_ENDPOINT}`);
  console.log(`   RPC Endpoint: ${CALIBNET_RPC}`);

  group('🔧 Health Check', () => {
    const res = http.get(`${BASE_URL}/`);
    check(res, {
      'API is healthy (status 200)': (r) => r.status === 200
    });
    if (res.status === 200) {
      console.log('✅ Health check passed');
    } else {
      console.log(`⚠️ Health check returned status: ${res.status}`);
    }
  });

  runTestSuite(API_TESTS, 'Main Test Suite (Success & Error Cases)');

  console.log(`\n⏰ Starting Rate Limiting Tests (this may take several minutes)...`);
  runTestSuite(RATE_LIMIT_TESTS, 'Rate Limiting Test Suite');

  console.log(`\n🏁 Test execution completed!`);
}