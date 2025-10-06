// API Test Configuration
export const API_CONFIG = {
  // Base URL - can be overridden by API_URL environment variable
  BASE_URL: __ENV.API_URL || 'http://127.0.0.1:8787',
  ENDPOINT: '/api/claim_token',

  // Test timeouts
  REQUEST_TIMEOUT: '30s',
  MAX_RESPONSE_TIME: 5000, // 5 seconds

  FAUCET_COOLDOWN_BUFFER_SECONDS: 65, // 65 seconds
};

export const TEST_ADDRESSES = {
  F1_FORMAT_ADDRESS: 'f15ydyu3d65gznpp2qxwpkjsgz4waubeunn6upvla',
  T1_FORMAT_ADDRESS: 't15ydyu3d65gznpp2qxwpkjsgz4waubeunn6upvla',
  T410_ADDRESS: 't410fw6vb5heeptnf6yhrvzxwlq7k4reerva7p667swi',
  ETH_FORMAT_ADDRESS: '0xb7aA1e9c847CDA5F60f1AE6f65C3eae44848D41f',
  T0_ADDRESS: 't0175013',
  ETH_ID_CORRESPONDING: '0xff0000000000000000000000000000000002aba5',

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
    },
    {
      name: 'Invalid address format for CalibnetUSDFC',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.T1_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.INTERNAL_SERVER_ERROR,
      expectedErrorContains: 'invalid address'
    }
  ],

  RATE_LIMIT_TEST_COOLDOWN_CASES: [
    // === CalibnetFIL Tests: One success → All addresses rate limited ===
    {
      name: 'CalibnetFIL (t1) - 1st SUCCESS (starts 60s cooldown for CalibnetFIL)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T1_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.SUCCESS
    },
    {
      name: 'CalibnetFIL (t410) - RATE LIMITED (within CalibnetFIL cooldown)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T410_ADDRESS,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },
    {
      name: 'CalibnetFIL (eth) - RATE LIMITED (within CalibnetFIL cooldown)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.ETH_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },
    {
      name: 'CalibnetFIL (t0) - RATE LIMITED (within CalibnetFIL cooldown)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T0_ADDRESS,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },
    {
      name: 'CalibnetFIL (ID) - RATE LIMITED (within CalibnetFIL cooldown)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.ETH_ID_CORRESPONDING,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },

    // === CalibnetUSDFC Tests: Independent cooldown from CalibnetFIL ===
    {
      name: 'CalibnetUSDFC (eth) - 1st SUCCESS (starts 60s cooldown for CalibnetUSDFC)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.ETH_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.SUCCESS
    },
    {
      name: 'CalibnetUSDFC (t410) - RATE LIMITED (within CalibnetUSDFC cooldown)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.T410_ADDRESS,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },
    {
      name: 'CalibnetUSDFC (t0) - RATE LIMITED (within CalibnetUSDFC cooldown)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.T0_ADDRESS,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },
    // CalibnetUSDFC doesn't support the t1 format address
    {
      name: 'CalibnetUSDFC (ID) - RATE LIMITED (within CalibnetUSDFC cooldown)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.ETH_ID_CORRESPONDING,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    }
  ],

  RATE_LIMIT_TEST_WALLET_CAP_CASES: [
    // === CalibnetFIL t1 Wallet (already has 1 transaction in RATE_LIMIT_TEST_COOLDOWN_CASES) ===
    {
      name: 'CalibnetFIL (t1) - 2nd SUCCESS (reaches cap)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T1_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.SUCCESS,
      waitBefore: 65, // Wait for cooldown from the main rate-limit tests to expire
      walletCapErrorResponse: false,
    },
    {
      name: 'CalibnetFIL (t1) - 3rd attempt (WALLET CAPPED)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T1_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS,
      waitBefore: 65, // Wait for cooldown from its own 2nd transaction
      walletCapErrorResponse: true,
    },

    // === CalibnetFIL eth/t410 Wallet (fresh wallet for this faucet) ===
    {
      name: 'CalibnetFIL (eth) - 1st SUCCESS',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.ETH_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.SUCCESS,
      waitBefore: 65, // Wait for cooldown from the previous test group
      walletCapErrorResponse: false,
    },
    {
      name: 'CalibnetFIL (eth) - 2nd SUCCESS (reaches cap)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.ETH_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.SUCCESS,
      waitBefore: 65, // Wait for its own cooldown
      walletCapErrorResponse: false,
    },
    {
      name: 'CalibnetFIL (eth) - 3rd attempt (WALLET CAPPED)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.ETH_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS,
      waitBefore: 65, // Wait for its own cooldown
      walletCapErrorResponse: true,
    },
    {
      name: 'CalibnetFIL (t410) - check equivalence (WALLET CAPPED)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T410_ADDRESS,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS,
      waitBefore: 0, // No wait needed, should be capped, already from the previous step
      walletCapErrorResponse: true,
    },
    {
      name: 'CalibnetFIL (t0) - check equivalence (WALLET CAPPED)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T0_ADDRESS,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS,
      waitBefore: 0, // No wait needed, should be capped, already from the previous step
      walletCapErrorResponse: true,
    },
    {
      name: 'CalibnetFIL (ID) - check equivalence (WALLET CAPPED)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.ETH_ID_CORRESPONDING,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS,
      waitBefore: 0, // No wait needed, should be capped, already from the previous step
      walletCapErrorResponse: true,
    },

    // === CalibnetUSDFC eth/t410 Wallet (already has 1 transaction in RATE_LIMIT_TEST_COOLDOWN_CASES) ===
    {
      name: 'CalibnetUSDFC (eth) - 2nd SUCCESS (reaches cap)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.ETH_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.SUCCESS,
      waitBefore: 65, // Wait for cooldown from the previous test group to expire
      walletCapErrorResponse: false,
    },
    {
      name: 'CalibnetUSDFC (eth) - 3rd attempt (WALLET CAPPED)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.ETH_FORMAT_ADDRESS,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS,
      waitBefore: 65, // Wait for cooldown from its own 2nd transaction
      walletCapErrorResponse: true,
    },
    {
      name: 'CalibnetUSDFC (t410) - check equivalence (WALLET CAPPED)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.T410_ADDRESS, // This is the same wallet as the ETH address
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS,
      waitBefore: 0, // No wait needed, should be capped, already from the previous step
      walletCapErrorResponse: true,
    },

    // === CalibnetUSDFC ID Wallet (fresh wallet, 0 transactions) ===
    {
      name: 'CalibnetUSDFC (ID) - 1st SUCCESS (fresh wallet)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.ETH_ID_CORRESPONDING,
      expectedStatus: STATUS_CODES.SUCCESS,
      waitBefore: 65, // Wait for cooldown from the previous test group to expire
      walletCapErrorResponse: false,
    },
    {
      name: 'CalibnetUSDFC (ID) - 2nd SUCCESS (reaches cap)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.ETH_ID_CORRESPONDING,
      expectedStatus: STATUS_CODES.SUCCESS,
      waitBefore: 65, // Wait for cooldown from its own 1st transaction
      walletCapErrorResponse: false,
    },
    {
      name: 'CalibnetUSDFC (ID) - 3rd attempt (WALLET CAPPED)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.ETH_ID_CORRESPONDING,
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS,
      waitBefore: 65, // Wait for cooldown from its own 2nd transaction
      walletCapErrorResponse: true,
    },
    {
      name: 'CalibnetUSDFC (t0) - check equivalence (WALLET CAPPED)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.T0_ADDRESS, // This is the same wallet as the ID address
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS,
      waitBefore: 0, // No wait needed, should be capped already
      walletCapErrorResponse: true,
    },
  ]
};
