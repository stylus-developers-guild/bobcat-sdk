import { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button } from '../components/ui/button';
import { Card } from '../components/ui/card';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '../components/ui/tabs';
import { BozoModal } from '../components/BozoModal';
import { EndGameScreen } from '../components/EndGameScreen';
import { HowItWorksDialog } from '../components/HowItWorksDialog';
import { Avatar, AvatarFallback } from '../components/ui/avatar';
import { Alert, AlertDescription } from '../components/ui/alert';
import { GameState, Deposit, Winners, RoundWinner } from '../types';
import { mockApi } from '../lib/mock-api';
import { formatAddress, formatTokenAmount, formatUsd, getTimeRemaining } from '../lib/utils';
import { Loader2, AlertTriangle, Settings, HelpCircle } from 'lucide-react';
import { toast } from 'sonner@2.0.3';

import { useAccount, usePublicClient } from 'wagmi';
import { parseAbi, formatEther } from 'viem';

import { ConnectButton } from '@rainbow-me/rainbowkit';
import { useComments } from '../providers/CommentsProvider';
import { config } from '../lib/config';

const depositEventAbi = parseAbi([
  'event DepositMade(address indexed recipient, uint256 indexed amount, uint256 indexed currentPool)'
]);

const DEPOSIT_LOOKBACK_BLOCKS = 200_000n;

export function Game() {
  const navigate = useNavigate();
  const [deposits, setDeposits] = useState<Deposit[]>([]);
  const [loading, setLoading] = useState(true);
  const [bozoModalOpen, setBozoModalOpen] = useState(false);
  const [howItWorksOpen, setHowItWorksOpen] = useState(false);
  const [winners] = useState<Winners | null>(null);
  const [roundWinners, setRoundWinners] = useState<RoundWinner[]>([]);
  const [currentTime, setCurrentTime] = useState(Date.now());
  const [poolAssetAddress, setPoolAssetAddress] = useState<`0x${string}` | null>(null);
  const [assetDecimals, setAssetDecimals] = useState<number>(18);
  const [homeToken, setHomeToken] = useState<string>(DEFAULT_HOME_TOKEN);

  const { address, isConnected } = useAccount();
  const publicClient = usePublicClient();
  const { getCommentForTxHash, refresh: refreshComments } = useComments();

  const loadGame = useCallback(async () => {
    try {
      const result = await mockApi.getGame();
      setGame(result);

  const poolSizeUsd = useMemo(() => {
    const numericAmount = parseFloat(poolSizeTokens);
    if (Number.isFinite(numericAmount)) {
      return numericAmount * HARD_CODED_TOKEN_PRICE_USD;
    }
    return 0;
  }, [poolSizeTokens]);

  const lastBettorAmountTokens = useMemo(() => {
    if (typeof lastBettorAmountData === 'bigint') {
      try {
        return formatUnits(lastBettorAmountData, assetDecimals);
      } catch (error) {
        console.error('Failed to format last bettor amount:', error);
      }
    }
  }, []);

  const loadDeposits = useCallback(() => {
    if (!publicClient) {
      return;
    }

    const run = async () => {
      try {
        try {
          await refreshComments();
        } catch (err) {
          console.error('Failed to refresh comments:', err);
        }

        const latestBlock = await publicClient.getBlockNumber();
        const fromBlock =
          latestBlock > DEPOSIT_LOOKBACK_BLOCKS ? latestBlock - DEPOSIT_LOOKBACK_BLOCKS : 0n;

        const events = await publicClient.getContractEvents({
          address: config.contracts.bozo as `0x${string}`,
          abi: depositEventAbi,
          eventName: 'DepositMade',
          fromBlock,
          toBlock: latestBlock,
        });

        const recentEvents = events.slice(-100);
        const blockNumbers = Array.from(
          new Set(
            recentEvents
              .map((event) => event.blockNumber)
              .filter((blockNumber): blockNumber is bigint => typeof blockNumber === 'bigint')
          )
        );

        if (blockNumbers.length === 0) {
          setDeposits([]);
          return;
        }

        const blocks = await Promise.all(
          blockNumbers.map((blockNumber) => publicClient.getBlock({ blockNumber }))
        );

        const blockTimestamps = new Map<bigint, string>();
        blocks.forEach((block, index) => {
          const timestamp = Number(block.timestamp) * 1000;
          blockTimestamps.set(blockNumbers[index], new Date(timestamp).toISOString());
        });

        const depositsFromEvents = [...recentEvents]
          .reverse()
          .map((event) => {
            if (!event.transactionHash || !event.blockNumber) {
              return null;
            }

            const recipient = event.args?.recipient as string | undefined;
            const amountRaw = event.args?.amount;
            const poolRaw = event.args?.currentPool;
            if (!recipient) {
              return null;
            }

            const timestampIso = blockTimestamps.get(event.blockNumber);
            if (!timestampIso) {
              return null;
            }

            const amount = typeof amountRaw === 'bigint' ? amountRaw : 0n;
            const pool = typeof poolRaw === 'bigint' ? poolRaw : 0n;

            const amountToken = formatEther(amount);
            const potAfterToken = formatEther(pool);

            return {
              ts: timestampIso,
              address: recipient,
              amountToken,
              amountUsd: parseFloat(amountToken),
              potAfterUsd: parseFloat(potAfterToken),
              txHash: event.transactionHash,
            } satisfies Deposit;
          })
          .filter((deposit): deposit is Deposit => deposit !== null);

        setDeposits(depositsFromEvents);
      } catch (error) {
        console.error('Failed to load deposits:', error);
      }
    };

    void run();
  }, [publicClient, refreshComments]);

  const loadRoundWinners = useCallback(async () => {
    try {
      const result = await mockApi.getRoundWinners();
      setRoundWinners(result);
    } catch (error) {
      console.error('Failed to load round winners:', error);
    }
  }, []);

  useEffect(() => {
    loadGame();
    loadRoundWinners();

    const gameInterval = setInterval(() => {
      loadGame();
    }, 5000);

    const timerInterval = setInterval(() => {
      setCurrentTime(Date.now());
    }, 1000);

    return () => {
      clearInterval(gameInterval);
      clearInterval(timerInterval);
    };
  }, [loadGame, loadRoundWinners]);

  useEffect(() => {
    if (!publicClient) {
      return;
    }

    loadDeposits();

    const interval = setInterval(() => {
      loadDeposits();
    }, 5000);

    return () => {
      clearInterval(interval);
    };
  }, [publicClient, loadDeposits]);

  const handleShare = async () => {
    const text = 'RIP BOZO 🤡';
    const shareText = `${text}\n${window.location.href}`;

    try {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        await navigator.clipboard.writeText(shareText);
        toast.success('Link copied to clipboard');
      } else {
        toast.success('Share: ' + shareText, {
          duration: 5000
        });
      }
    } catch (error) {
      toast.success('Share: ' + shareText, {
        duration: 5000
      });
    }
  };

  const handleClaim = async () => {
    try {
      const claimTx = await mockApi.prepareClaim();
      toast.success('Claim transaction prepared');
    } catch (error) {
      toast.error('Failed to prepare claim');
    }
  };

  const formatTime = (ts: string) => {
    const date = new Date(ts);
    const now = Date.now();
    const diff = now - date.getTime();
    const minutes = Math.floor(diff / 60000);

    if (minutes < 1) return 'JUST NOW';
    if (minutes === 1) return '1 MIN AGO';
    if (minutes < 60) return `${minutes} MINS AGO`;
    const hours = Math.floor(minutes / 60);
    if (hours === 1) return '1 HOUR AGO';
    if (hours < 24) return `${hours} HOURS AGO`;
    const days = Math.floor(hours / 24);
    if (days === 1) return '1 DAY AGO';
    return `${days} DAYS AGO`;
  };

  const toRoman = (num: number): string => {
    const romanNumerals: [number, string][] = [
      [10, 'X'], [9, 'IX'], [5, 'V'], [4, 'IV'], [1, 'I']
    ];
    let result = '';
    for (const [value, numeral] of romanNumerals) {
      while (num >= value) {
        result += numeral;
        num -= value;
      }
    }
    return result;
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <Loader2 className="w-8 h-8 animate-spin text-[#FF4B4B]" />
      </div>
    );
  }

  // Show end game screen if game is closed
  if (isGameClosed && winners) {
    return (
      <div className="min-h-screen bg-background relative overflow-hidden">
        {/* Decorative elements */}
        <div className="absolute top-8 left-8 w-32 h-32 rounded-full bg-[#FF4B4B] opacity-20 blur-3xl"></div>
        <div className="absolute top-8 right-8 w-32 h-32 rounded-full bg-[#F6C445] opacity-20 blur-3xl"></div>

        {/* Top Nav */}
        <header className="relative z-10 border-b border-border/50 bg-background/80 backdrop-blur-sm">
          <div className="container mx-auto px-4 py-4">
            <div className="flex items-center justify-between max-w-6xl mx-auto">
              <div className="flex items-center gap-6">
                <div className="flex items-center gap-2">
                  <div className="w-8 h-8 rounded-full bg-[#FF4B4B] flex items-center justify-center">
                    <div className="w-4 h-4 rounded-full bg-[#FFF2E1]" />
                  </div>
                  <span className="text-foreground tracking-wider">BOZO</span>
                </div>
                <nav className="flex items-center gap-6">
                  <button
                    onClick={() => navigate('/')}
                    className="text-sm text-foreground hover:text-[#F6C445] transition-colors"
                  >
                    GAME
                  </button>
                  <button
                    onClick={() => navigate('/stats')}
                    className="text-sm text-muted-foreground hover:text-foreground transition-colors"
                  >
                    LEADERBOARD
                  </button>
                  <button
                    onClick={() => navigate('/faq')}
                    className="text-sm text-muted-foreground hover:text-foreground transition-colors"
                  >
                    FAQ
                  </button>
                  <button
                    onClick={() => setHowItWorksOpen(true)}
                    className="text-sm text-muted-foreground hover:text-foreground transition-colors flex items-center gap-1"
                  >
                    <HelpCircle className="w-4 h-4" />
                    HOW IT WORKS
                  </button>
                </nav>
              </div>
            </div>
          </div>
        </header>

        <EndGameScreen
          winners={winners}
          nextGameStartsAt={new Date(Date.now() + 300000)}
          homeToken={homeToken}
        />

        <HowItWorksDialog open={howItWorksOpen} onOpenChange={setHowItWorksOpen} />
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background relative overflow-hidden">
      {/* Decorative Clown Nose - Top Left */}
      <div className="absolute top-8 left-8 w-32 h-32 rounded-full bg-[#FF4B4B] opacity-20 blur-3xl"></div>

      {/* Decorative Clown Nose - Top Right */}
      <div className="absolute top-8 right-8 w-32 h-32 rounded-full bg-[#F6C445] opacity-20 blur-3xl"></div>

      {/* Top Nav */}
      <header className="relative z-10 border-b border-border/50 bg-background/80 backdrop-blur-sm">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between max-w-6xl mx-auto">
            <div className="flex items-center gap-6">
              <div className="flex items-center gap-2">
                <div className="w-8 h-8 rounded-full bg-[#FF4B4B] flex items-center justify-center">
                  <div className="w-4 h-4 rounded-full bg-[#FFF2E1]" />
                </div>
                <span className="text-foreground tracking-wider">BOZO</span>
              </div>
              <nav className="flex items-center gap-6">
                <button
                  onClick={() => navigate('/')}
                  className="text-sm text-foreground hover:text-[#F6C445] transition-colors"
                >
                  GAME
                </button>
                <button
                  onClick={() => navigate('/stats')}
                  className="text-sm text-muted-foreground hover:text-foreground transition-colors"
                >
                  LEADERBOARD
                </button>
                <button
                  onClick={() => navigate('/faq')}
                  className="text-sm text-muted-foreground hover:text-foreground transition-colors"
                >
                  FAQ
                </button>
                <button
                  onClick={() => setHowItWorksOpen(true)}
                  className="text-sm text-muted-foreground hover:text-foreground transition-colors flex items-center gap-1"
                >
                  <HelpCircle className="w-4 h-4" />
                  HOW IT WORKS
                </button>
              </nav>
            </div>

            <ConnectButton />
          </div>
        </div>
      </header>

      {/* Game Status Alert */}
      {isGamePaused && (
        <div className="container mx-auto px-4 mt-4 max-w-6xl">
          <Alert className="bg-[#FF4B4B]/10 border-[#FF4B4B]">
            <AlertTriangle className="h-4 w-4 text-[#FF4B4B]" />
            <AlertDescription className="text-foreground">
              Deposits paused. Check back soon.
            </AlertDescription>
          </Alert>
        </div>
      )}

      {/* Main Content */}
      <div className="container mx-auto px-4 py-12 max-w-6xl">
        {/* Top Section - Pot & Timer */}
        <div className="grid grid-cols-2 gap-8 mb-8">
          {/* Lottery Pool */}
          <div className="text-left">
            <div className="flex items-center gap-2 mb-2">
              <div className="w-2 h-2 rounded-full bg-[#2ED4B7] animate-pulse"></div>
              <span className="text-sm text-muted-foreground tracking-wider">LOTTERY POOL</span>
            </div>
            <div className="font-mono text-6xl text-[#F6C445] tracking-tight">{displayPot}</div>
            <div className="text-sm text-muted-foreground mt-1">
              AWARDED ACROSS TEN RANDOM BOZOS
            </div>
          </div>

          {/* Time Remaining */}
          <div className="text-right">
            <div className="text-sm text-muted-foreground mb-2 tracking-wider">TIME REMAINING</div>
            <div className={`font-mono text-6xl tracking-tight ${
              timeRemaining.total < 60000 ? 'text-[#FF4B4B]' : 'text-[#F6C445]'
            }`}>
              {timeRemaining.formatted}
            </div>
            <div className="text-sm text-muted-foreground mt-1">UNTIL THE GAME STARTS</div>
          </div>
        </div>

        {/* BOZO Button */}
        <Button
          onClick={() => setBozoModalOpen(true)}
          disabled={!isConnected || !isGameActive}
          className="w-full h-16 bg-[#F6C445] hover:bg-[#F6C445]/90 text-[#0E1020] text-xl tracking-widest mb-12"
        >
          {isConnected ? 'BOZO' : 'CONNECT TO BOZO'}
        </Button>

        {/* Activity Feed */}
        <div className="bg-[#1a1d32]/50 rounded-lg border border-border/50 overflow-hidden">
          <Tabs defaultValue="latest" className="w-full">
            <div className="border-b border-border/50 px-6 py-4">
              <div className="flex items-center justify-between">
                <h3 className="text-foreground tracking-wider">BOZOS</h3>
                <TabsList className="bg-transparent h-auto p-0 gap-6">
                  <TabsTrigger
                    value="latest"
                    className="bg-transparent data-[state=active]:bg-transparent data-[state=active]:text-[#2ED4B7] data-[state=active]:shadow-none text-muted-foreground px-0"
                  >
                    LATEST
                  </TabsTrigger>
                  <TabsTrigger
                    value="winners"
                    className="bg-transparent data-[state=active]:bg-transparent data-[state=active]:text-[#2ED4B7] data-[state=active]:shadow-none text-muted-foreground px-0"
                  >
                    WINNERS
                  </TabsTrigger>
                  <TabsTrigger
                    value="yours"
                    className="bg-transparent data-[state=active]:bg-transparent data-[state=active]:text-[#2ED4B7] data-[state=active]:shadow-none text-muted-foreground px-0"
                  >
                    YOURS
                  </TabsTrigger>
                </TabsList>
              </div>
            </div>

            <TabsContent value="latest" className="mt-0">
              <div className="divide-y divide-border/30">
                {deposits.slice(0, 10).map((deposit, index) => {
                  const commentText = getCommentForTxHash(deposit.txHash);
                  return (
                  <div
                    key={deposit.txHash}
                    className="px-6 py-4 hover:bg-[#252840]/50 transition-colors"
                  >
                    <div className="flex items-start justify-between gap-4">
                      <div className="flex items-start gap-3 flex-1 min-w-0">
                        <div className="text-sm font-mono text-muted-foreground w-6 mt-1">
                          #{deposits.length - index}
                        </div>
                        <Avatar className="w-8 h-8 mt-1">
                          <AvatarFallback className="bg-[#FF4B4B] text-[#FFF2E1] text-xs">
                            {deposit.handle?.[0]?.toUpperCase() || deposit.address.slice(2, 4).toUpperCase()}
                          </AvatarFallback>
                        </Avatar>
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 mb-1">
                            <div className="text-sm text-foreground font-mono">
                              {deposit.handle || formatAddress(deposit.address)}
                            </div>
                            {index === 0 && (
                              <div className="flex items-center gap-1">
                                <div className="w-2 h-2 rounded-full bg-[#2ED4B7]"></div>
                                <span className="text-xs text-[#2ED4B7] tracking-wider">LEADER</span>
                              </div>
                            )}
                          </div>
                          {commentText && (
                            <div className="text-sm text-foreground/90 bg-[#252840]/80 rounded px-3 py-2 mb-2 mt-2">
                              &quot;{commentText}&quot;
                            </div>
                          )}
                        </div>
                      </div>
                      <div className="text-right flex-shrink-0">
                        <div className="text-sm font-mono text-foreground">
                          {formatTokenAmount(deposit.amountToken, 2)} ${homeToken}
                        </div>
                        <div className="text-xs text-muted-foreground">
                          {formatTime(deposit.ts)}
                        </div>
                      </div>
                    </div>
                  </div>
                  );
                })}
              </div>
            </TabsContent>

            <TabsContent value="winners" className="mt-0">
              {roundWinners.length > 0 ? (
                <div className="divide-y divide-border/30">
                  {roundWinners.map((winner, index) => (
                    <div
                      key={`${winner.address}-${winner.roundNumber}-${index}`}
                      className="px-6 py-3 hover:bg-[#252840]/50 transition-colors"
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                          <div className="text-xl">
                            {winner.type === 'winner' ? '👑' : '🎫'}
                          </div>
                          <div>
                            <div className="text-sm text-foreground font-mono">
                              {winner.handle || winner.address}
                            </div>
                            <div className={`text-xs tracking-wider ${
                              winner.type === 'winner' ? 'text-[#F6C445]' : 'text-[#FF4B4B]'
                            }`}>
                              {winner.type === 'winner'
                                ? `WINNER ROUND ${toRoman(winner.roundNumber)}`
                                : 'LOTTERY WINNER'
                              }
                            </div>
                          </div>
                        </div>
                        <div className="text-right">
                          <div className="text-sm font-mono text-foreground">
                            {formatTokenAmount(winner.amountToken, 1)} ${homeToken}
                          </div>
                          <div className="text-xs text-muted-foreground">
                            {formatTime(winner.ts)}
                          </div>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="px-6 py-12 text-center text-muted-foreground">
                  No winners yet
                </div>
              )}
            </TabsContent>

            <TabsContent value="yours" className="mt-0">
              <div className="px-6 py-12 text-center text-muted-foreground">
                {isConnected ? 'No deposits yet' : 'Connect wallet to view your deposits'}
              </div>
            </TabsContent>
          </Tabs>
        </div>

        {/* Info Footer */}
        <div className="mt-8 text-center text-xs text-muted-foreground">
          Min deposit: {formatUsd(minToResetUsd)} • Pool asset: {homeToken} ({poolAssetDisplay}) • 80% to winner • 20% to 10 random bozos
        </div>
      </div>

      {/* Bozo Modal */}
      <BozoModal
        open={bozoModalOpen}
        onOpenChange={setBozoModalOpen}
        game={game}
        isConnected={isConnected}
      />

      <HowItWorksDialog open={howItWorksOpen} onOpenChange={setHowItWorksOpen} />
    </div>
  );
}
