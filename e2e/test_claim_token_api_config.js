// API Test Configuration
export const API_CONFIG = {
  // Base URL - can be overridden by API_URL environment variable
  BASE_URL: __ENV.API_URL || 'https://forest-explorer.chainsafe.dev',
  ENDPOINT: '/api/claim_token',

  // Test timeouts
  REQUEST_TIMEOUT: '30s',
  MAX_RESPONSE_TIME: 5000, // 5 seconds
};

export const TEST_ADDRESSES = {
  F1_FORMAT_ADDRESS: 'f15ydyu3d65gznpp2qxwpkjsgz4waubeunn6upvla',
  T1_FORMAT_ADDRESS: 't15ydyu3d65gznpp2qxwpkjsgz4waubeunn6upvla',
  T410_ADDRESS: 't410fokwxytjh7socxx4p3y2n3wn4xy2gjcxtc7iv3sy',
  ETH_FORMAT_ADDRESS: '0x72ad7c4d27fc9c2bdf8fde34ddd9bcbe34648af3',
  T0_ADDRESS: 't0163912',
  ETH_ID_CORRESPONDING: '0xff00000000000000000000000000000000028048',

  INVALID: [
    'invalidaddress',
    '0xinvalid',
    't1invalid',
    'f1invalid',
    '',
    '0x123',
    'randomstring',
    '0xABC',
    't1abc',
    'f1xyz',
  ]
};

export const FaucetTypes = {
  CalibnetFIL: 'CalibnetFIL',
  CalibnetUSDFC: 'CalibnetUSDFC',
  MainnetFIL: 'MainnetFIL',
  InvalidFaucet: 'InvalidFaucet'
};

export const STATUS_CODES = {
  SUCCESS: 200,
  BAD_REQUEST: 400,
  TOO_MANY_REQUESTS: 429,
  INTERNAL_SERVER_ERROR: 500,
  IM_A_TEAPOT: 418
};


export const TEST_SCENARIOS = {
  // Invalid request test cases
  INVALID_REQUESTS: [
    // Missing parameter tests
    {
      name: 'Missing both parameters',
      faucet_info: null,
      address: null,
      expectedStatus: STATUS_CODES.INTERNAL_SERVER_ERROR,
      expectedErrorContains: 'missing'
    },
    {
      name: 'Missing faucet_info parameter',
      faucet_info: null,
      address: TEST_ADDRESSES.T1_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.INTERNAL_SERVER_ERROR,
      expectedErrorContains: 'missing'
    },
    {
      name: 'Missing address parameter CalibnetFIL',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: null,
      expectedStatus: STATUS_CODES.INTERNAL_SERVER_ERROR,
      expectedErrorContains: 'missing'
    },
    {
      name: 'Missing address parameter CalibnetUSDFC',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: null,
      expectedStatus: STATUS_CODES.INTERNAL_SERVER_ERROR,
      expectedErrorContains: 'missing'
    },
    {
      name: 'MainnetFIL request (should be blocked)',
      faucet_info: FaucetTypes.MainnetFIL,
      address: TEST_ADDRESSES.F1_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.IM_A_TEAPOT,
      expectedErrorContains: 'teapot'
    },
    // Invalid faucet type tests
    {
      name: 'Invalid faucet type',
      faucet_info: FaucetTypes.InvalidFaucet,
      address: TEST_ADDRESSES.T1_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.INTERNAL_SERVER_ERROR,
      expectedErrorContains: 'unknown variant'
    },
    {
      name: 'Empty faucet_info parameter',
      faucet_info: '',
      address: TEST_ADDRESSES.T1_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.INTERNAL_SERVER_ERROR,
      expectedErrorContains: 'unknown variant'
    }
  ],

  RATE_LIMIT_TEST_COOLDOWN_CASES: [
    // === CalibnetFIL Tests: One success → All addresses rate limited ===
    {
      name: 'CalibnetFIL with t1 address - SUCCESS (starts 60s cooldown for CalibnetFIL)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T1_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.SUCCESS
    },
    {
      name: 'CalibnetFIL with t410 address - RATE LIMITED (within CalibnetFIL cooldown)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T410_ADDRESS,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },
    {
      name: 'CalibnetFIL with ETH address - RATE LIMITED (within CalibnetFIL cooldown)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.ETH_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },
    {
      name: 'CalibnetFIL with t0 address - RATE LIMITED (within CalibnetFIL cooldown)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T0_ADDRESS,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },
    {
      name: 'CalibnetFIL with eth ID address - RATE LIMITED (within CalibnetFIL cooldown)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.ETH_ID_CORRESPONDING,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },

    // === CalibnetUSDFC Tests: Independent cooldown from CalibnetFIL ===
    {
      name: 'CalibnetUSDFC with eth address - SUCCESS (starts 60s cooldown for CalibnetUSDFC)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.ETH_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.SUCCESS
    },
    {
      name: 'CalibnetUSDFC with t410 address - RATE LIMITED (within CalibnetUSDFC cooldown)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.T410_ADDRESS,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },
    {
      name: 'CalibnetUSDFC with t0 address - RATE LIMITED (within CalibnetUSDFC cooldown)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.T0_ADDRESS,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },
    {
      name: 'CalibnetUSDFC with eth ID address - RATE LIMITED (within CalibnetUSDFC cooldown)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.ETH_ID_CORRESPONDING,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    }
  ]
};
