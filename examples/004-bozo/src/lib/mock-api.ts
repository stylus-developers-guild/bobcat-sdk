// Mock API for local development
import { GameState, Winners, RouteQuote, LeaderboardEntry, RoundWinner } from '../types';

import { config } from './config';

const mockGame: GameState = {
  potTokenAmount: '3.23',
  potUsd: 10450.00,
  minPct: 0.01,
  minToResetUsd: 104.50,
  deadline: new Date(Date.now() + 180000).toISOString(), // 3 minutes from now
  lastDepositor: { 
    address: '0x1234567890123456789012345678901234567890', 
    fid: 12345, 
    handle: 'alice' 
  },
  status: config.testEndGameScreen ? 'Closed' : 'Active',
  homeToken: 'ETH',
  chain: 'base',
  nextGameStartsAt: new Date(Date.now() + 300000).toISOString() // 5 minutes from now
};

const mockRoundWinners: RoundWinner[] = [
  // Round 6 (most recent)
  {
    type: 'winner',
    roundNumber: 6,
    address: '0x93db...694e',
    handle: 'vitalik',
    amountToken: '54.6',
    amountUsd: 174720,
    ts: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 6,
    address: 'xabbu.🐵🕳️',
    handle: 'xabbu',
    amountToken: '16.1',
    amountUsd: 51520,
    ts: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 6,
    address: '0x1ee5...ab3d',
    amountToken: '16.1',
    amountUsd: 51520,
    ts: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 6,
    address: '0x81db...031b',
    amountToken: '16.1',
    amountUsd: 51520,
    ts: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 6,
    address: '0x81db...051b',
    amountToken: '16.1',
    amountUsd: 51520,
    ts: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 6,
    address: '0x81db...051b',
    amountToken: '16.1',
    amountUsd: 51520,
    ts: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 6,
    address: '0x81db...051b',
    amountToken: '16.1',
    amountUsd: 51520,
    ts: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 6,
    address: '0x81db...051b',
    amountToken: '16.1',
    amountUsd: 51520,
    ts: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 6,
    address: '0x81db...051b',
    amountToken: '16.1',
    amountUsd: 51520,
    ts: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 6,
    address: '0x81db...051b',
    amountToken: '16.1',
    amountUsd: 51520,
    ts: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 6,
    address: '0x81db...051b',
    amountToken: '16.1',
    amountUsd: 51520,
    ts: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 6,
    address: '0x93db...694e',
    amountToken: '16.1',
    amountUsd: 51520,
    ts: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000).toISOString()
  },
  
  // Round 5
  {
    type: 'winner',
    roundNumber: 5,
    address: '0x93db...694e',
    handle: 'jessepollak',
    amountToken: '85.0',
    amountUsd: 272000,
    ts: new Date(Date.now() - 6 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 5,
    address: '0xfe36...7ea5',
    amountToken: '26.1',
    amountUsd: 83520,
    ts: new Date(Date.now() - 6 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 5,
    address: '0x2e1a...8bc4',
    amountToken: '26.1',
    amountUsd: 83520,
    ts: new Date(Date.now() - 6 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 5,
    address: '0x8f3b...4cd2',
    amountToken: '26.1',
    amountUsd: 83520,
    ts: new Date(Date.now() - 6 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 5,
    address: '0x7a9c...2de1',
    amountToken: '26.1',
    amountUsd: 83520,
    ts: new Date(Date.now() - 6 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 5,
    address: '0x4b2d...9fa3',
    amountToken: '26.1',
    amountUsd: 83520,
    ts: new Date(Date.now() - 6 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 5,
    address: '0x1c8e...5ab7',
    amountToken: '26.1',
    amountUsd: 83520,
    ts: new Date(Date.now() - 6 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 5,
    address: '0x9d5f...3bc8',
    amountToken: '26.1',
    amountUsd: 83520,
    ts: new Date(Date.now() - 6 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 5,
    address: '0x6a4c...7de2',
    amountToken: '26.1',
    amountUsd: 83520,
    ts: new Date(Date.now() - 6 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 5,
    address: '0x3e7b...1fa9',
    amountToken: '26.1',
    amountUsd: 83520,
    ts: new Date(Date.now() - 6 * 24 * 60 * 60 * 1000).toISOString()
  },
  {
    type: 'lottery',
    roundNumber: 5,
    address: '0x2f1c...8cd4',
    amountToken: '26.1',
    amountUsd: 83520,
    ts: new Date(Date.now() - 6 * 24 * 60 * 60 * 1000).toISOString()
  }
];

export const mockApi = {
  async getGame(): Promise<GameState> {
    await new Promise(resolve => setTimeout(resolve, 200));
    return mockGame;
  },

  async prepareClaim() {
    await new Promise(resolve => setTimeout(resolve, 200));
    return {
      ok: true,
      tx: {
        to: '0xPOT123',
        data: '0xclaim123',
        value: '0',
        chainId: 8453
      }
    };
  },

  async getRouteQuote(params: {
    sourceChain: string;
    sourceAsset: string;
    amountSource: string;
    slippageBps: number;
  }): Promise<RouteQuote> {
    await new Promise(resolve => setTimeout(resolve, 500));
    
    const depositUsd = parseFloat(params.amountSource);
    const isCrossChain = params.sourceChain !== 'base';
    
    return {
      ok: true,
      depositUsd,
      meetsMinPct: depositUsd >= mockGame.minToResetUsd,
      estArrivalSec: isCrossChain ? 120 : 15,
      hops: isCrossChain
        ? [
            { chain: params.sourceChain, asset: params.sourceAsset, action: 'approve' },
            { chain: params.sourceChain, asset: params.sourceAsset, action: 'bridge', toChain: 'base' },
            { chain: 'base', asset: 'ETH', action: 'deposit' }
          ]
        : [
            { chain: params.sourceChain, asset: params.sourceAsset, action: 'approve' },
            { chain: params.sourceChain, asset: params.sourceAsset, action: 'deposit' }
          ],
      router: 'lifi',
      tx: {
        to: '0xROUTER123',
        data: '0x1234567890',
        value: '0',
        chainId: params.sourceChain === 'base' ? 8453 : 137
      },
      pendingIntent: isCrossChain ? {
        expiresInSec: 120,
        hash: '0xPI' + Math.random().toString(36).substring(7)
      } : undefined
    };
  },

  async commitDeposit(params: { pendingIntentHash?: string; txHash: string; comment?: string }) {
    await new Promise(resolve => setTimeout(resolve, 300));
    return { ok: true, status: 'Confirmed' };
  },

  async getWinners(): Promise<Winners> {
    await new Promise(resolve => setTimeout(resolve, 200));
    return {
      winner: {
        address: '0x1234567890123456789012345678901234567890',
        fid: 9999,
        handle: 'alice',
        amountToken: '54.60',
        amountUsd: 174720.00
      },
      community: [
        { address: '0xabc...123', handle: 'bob', amountUsd: 21840 },
        { address: '0xdef...456', handle: 'charlie', amountUsd: 21840 },
        { address: '0xghi...789', handle: 'diana', amountUsd: 21840 },
        { address: '0xjkl...012', handle: 'eve', amountUsd: 21840 },
        { address: '0xmno...345', amountUsd: 21840 },
        { address: '0xpqr...678', handle: 'frank', amountUsd: 21840 },
        { address: '0xstu...901', amountUsd: 21840 },
        { address: '0xvwx...234', handle: 'grace', amountUsd: 21840 },
        { address: '0xyz...567', amountUsd: 21840 },
        { address: '0x123...890', handle: 'henry', amountUsd: 21840 }
      ]
    };
  },

  async getRoundWinners(): Promise<RoundWinner[]> {
    await new Promise(resolve => setTimeout(resolve, 200));
    return mockRoundWinners;
  },

  async getLeaderboard(type: 'winners' | 'first' | 'sniped'): Promise<LeaderboardEntry[]> {
    await new Promise(resolve => setTimeout(resolve, 200));
    
    if (type === 'winners') {
      return [
        { rank: 1, address: '0x111...', handle: 'alice', wins: 12, totalWon: 45600 },
        { rank: 2, address: '0x222...', handle: 'bob', wins: 9, totalWon: 38200 },
        { rank: 3, address: '0x333...', handle: 'carol', wins: 7, totalWon: 29800 },
        { rank: 4, address: '0x444...', handle: 'dave', wins: 5, totalWon: 18900 },
        { rank: 5, address: '0x555...', handle: 'eve', wins: 4, totalWon: 15200 }
      ];
    } else if (type === 'first') {
      return [
        { rank: 1, address: '0x111...', handle: 'quickdraw', firstIns: 23, totalDeposited: 12300 },
        { rank: 2, address: '0x222...', handle: 'speedster', firstIns: 18, totalDeposited: 9800 },
        { rank: 3, address: '0x333...', handle: 'earlybird', firstIns: 15, totalDeposited: 8200 },
        { rank: 4, address: '0x444...', handle: 'pioneer', firstIns: 12, totalDeposited: 6700 },
        { rank: 5, address: '0x555...', handle: 'starter', firstIns: 10, totalDeposited: 5400 }
      ];
    } else {
      return [
        { rank: 1, address: '0x111...', handle: 'unlucky', timesSniped: 34, totalLost: 15600 },
        { rank: 2, address: '0x222...', handle: 'almostwon', timesSniped: 28, totalLost: 12900 },
        { rank: 3, address: '0x333...', handle: 'socloseyet', timesSniped: 24, totalLost: 11200 },
        { rank: 4, address: '0x444...', handle: 'tryharder', timesSniped: 21, totalLost: 9800 },
        { rank: 5, address: '0x555...', handle: 'nexttime', timesSniped: 18, totalLost: 8100 }
      ];
    }
  }
};
