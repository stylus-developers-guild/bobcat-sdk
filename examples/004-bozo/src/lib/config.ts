// Environment configuration

const env = typeof import.meta !== 'undefined' && import.meta.env ? import.meta.env : {};

export const config = {
  // Home chain (base or arbitrum)
  homeChain: (env.VITE_HOME_CHAIN as 'base' | 'arbitrum') || 'base',

  // Router service (lifi, socket, or across)
  router: env.VITE_ROUTER || 'lifi',

  // API URL
  apiUrl: env.VITE_API_URL || 'http://localhost:3001/api',

  // GraphQL endpoint for Bozo data
  graphqlUrl: env.VITE_GRAPHQL_URL || 'http://localhost:8080/query',
  
  // Farcaster Hub
  fcHub: env.VITE_FC_HUB || '',
  
  // Image base URL
  imgBase: env.VITE_IMG_BASE || 'https://img.bozo.gg',
  
  // Chain IDs
  chainIds: {
    base: 8453,
    arbitrum: 42161,
    ethereum: 1,
    polygon: 137
  },
  
  // RPC URLs (for development)
  rpcUrls: {
    base: 'https://mainnet.base.org',
    arbitrum: 'https://arb1.arbitrum.io/rpc',
    ethereum: 'https://eth.llamarpc.com',
    polygon: 'https://polygon-rpc.com'
  },

  contracts: {
    bozo: (env.VITE_BOZO_CONTRACT as `0x${string}`) || '0x6221a9c005f6e47eb398fd867784cacfdcfff4e7',
  },
  
  // Testing flags
  testEndGameScreen: false // Set to true to preview the end game screen
};
