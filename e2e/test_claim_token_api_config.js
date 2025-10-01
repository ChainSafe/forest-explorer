// API Test Configuration
export const API_CONFIG = {
  // Base URL - can be overridden by API_URL environment variable
  BASE_URL: __ENV.API_URL || 'http://127.0.0.1:8787',
  ENDPOINT: '/api/claim_token',

  // Test timeouts
  REQUEST_TIMEOUT: '30s',
  MAX_RESPONSE_TIME: 5000, // 5 seconds
};

export const TEST_ADDRESSES = {
  F1_FORMAT_ADDRESS: 'f1pxxbe7he3c6vcw5as3gfvq33kprpmlufgtjgfdq',
  T1_FORMAT_ADDRESS: 't1pxxbe7he3c6vcw5as3gfvq33kprpmlufgtjgfdq',
  T410_ADDRESS: 't410fv2oexfiizeuzm3xtoie3gnxfpfwwglg4q3dgxki',
  ETH_FORMAT_ADDRESS: '0xAe9C4b9508c929966ef37209b336E5796D632CDc',
  T0_ADDRESS: 't0163355',
  ETH_ID_CORRESPONDING: '0xff00000000000000000000000000000000027e1b',

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
      name: 'Missing address parameter',
      faucet_info: FaucetTypes.CalibnetFIL,
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
    // CalibnetFIL with t1 address: success → rate limited
    {
      name: 'CalibnetFIL with t1 address',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T1_FORMAT_ADDRESS,
      expectedTxFormat: 'filecoin',
      expectedStatus: STATUS_CODES.SUCCESS
    },
    {
      name: 'CalibnetFIL with t1 address (immediate retry - rate limited)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T1_FORMAT_ADDRESS,
      expectedTxFormat: 'filecoin',
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },

    // CalibnetFIL with t410 and ETH address: success → rate limited
    {
      name: 'CalibnetFIL with t410 address',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T410_ADDRESS,
      expectedTxFormat: 'filecoin',
      expectedStatus: STATUS_CODES.SUCCESS
    },
    {
      name: 'CalibnetFIL with t410 address (immediate retry - rate limited)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T410_ADDRESS,
      expectedTxFormat: 'filecoin',
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },
    {
      name: 'CalibnetFIL with ETH address (immediate retry after t410 - rate limited as ETH and t410 are both same address)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.ETH_FORMAT_ADDRESS,
      expectedTxFormat: 'filecoin',
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },

    // CalibnetFIL with t0 address: success → rate limited
    {
      name: 'CalibnetFIL with t0 address',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T0_ADDRESS,
      expectedTxFormat: 'filecoin',
      expectedStatus: STATUS_CODES.SUCCESS
    },
    {
      name: 'CalibnetFIL with t0 address (immediate retry - rate limited)',
      faucet_info: FaucetTypes.CalibnetFIL,
      address: TEST_ADDRESSES.T0_ADDRESS,
      expectedTxFormat: 'filecoin',
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },

    // CalibnetUSDFC with eth address: success → rate limited
    {
      name: 'CalibnetUSDFC with eth address',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.ETH_FORMAT_ADDRESS,
      expectedTxFormat: 'ethereum',
      expectedStatus: STATUS_CODES.SUCCESS
    },
    {
      name: 'CalibnetUSDFC with eth address (immediate retry - rate limited)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.ETH_FORMAT_ADDRESS,
      expectedTxFormat: 'ethereum',
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },
    {
      name: 'CalibnetUSDFC with t410 address (immediate retry after ETH - rate limited as ETH and t410 are both same address)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.T410_ADDRESS,
      expectedTxFormat: 'ethereum',
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    },

    // CalibnetUSDFC with eth ID address: success → rate limited
    {
      name: 'CalibnetUSDFC with eth ID address',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.ETH_ID_CORRESPONDING,
      expectedTxFormat: 'ethereum',
      expectedStatus: STATUS_CODES.SUCCESS
    },
    {
      name: 'CalibnetUSDFC with eth ID address (immediate retry - rate limited)',
      faucet_info: FaucetTypes.CalibnetUSDFC,
      address: TEST_ADDRESSES.ETH_ID_CORRESPONDING,
      expectedTxFormat: 'ethereum',
      expectedStatus: STATUS_CODES.TOO_MANY_REQUESTS
    }
  ]
};
