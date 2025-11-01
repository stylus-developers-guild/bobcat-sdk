import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from './ui/dialog';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { Textarea } from './ui/textarea';
import { Alert, AlertDescription } from './ui/alert';
import { formatUsd } from '../lib/utils';
import { GameState } from '../types';
import { Loader2, AlertCircle, CheckCircle } from 'lucide-react';
import { toast } from 'sonner@2.0.3';
import { mockApi } from '../lib/mock-api';
import {
  useAccount,
  usePublicClient,
  useReadContract,
  useBalance,
  useSwitchChain,
  useWriteContract,
} from 'wagmi';
import { arbitrum } from 'wagmi/chains';
import { formatUnits, parseUnits } from 'viem';
import { config as appConfig } from '../lib/config';

interface BozoModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  game: GameState;
  isConnected: boolean;
  poolAssetAddress: `0x${string}` | null;
  assetDecimals: number;
  tokenPriceUsd: number;
}

const ZERO_ADDRESS = '0x0000000000000000000000000000000000000000' as const;
const bozoAbi = [
  {
    type: 'function',
    name: 'play',
    stateMutability: 'nonpayable',
    inputs: [
      { name: 'asset', type: 'address' },
      { name: 'camelotMinAssetOut', type: 'uint256' },
      { name: 'camelotDeadline', type: 'uint256' },
      { name: 'amount', type: 'uint256' },
      { name: 'recipient', type: 'address' },
    ],
    outputs: [
      { name: 'epoch', type: 'uint256' },
      { name: 'deposited', type: 'uint256' },
    ],
  },
] as const;

const erc20Abi = [
  {
    type: 'function',
    name: 'allowance',
    stateMutability: 'view',
    inputs: [
      { name: 'owner', type: 'address' },
      { name: 'spender', type: 'address' },
    ],
    outputs: [{ name: '', type: 'uint256' }],
  },
  {
    type: 'function',
    name: 'approve',
    stateMutability: 'nonpayable',
    inputs: [
      { name: 'spender', type: 'address' },
      { name: 'value', type: 'uint256' },
    ],
    outputs: [{ name: '', type: 'bool' }],
  },
] as const;

export function BozoModal({
  open,
  onOpenChange,
  game,
  isConnected,
  poolAssetAddress,
  assetDecimals,
  tokenPriceUsd,
}: BozoModalProps) {
  const { address, chainId } = useAccount();
  const publicClient = usePublicClient();
  const { writeContractAsync, isPending: isWriting } = useWriteContract();
  const { switchChainAsync, isPending: isSwitchingChain } = useSwitchChain();

  const [amountToken, setAmountToken] = useState('');
  const [comment, setComment] = useState('');
  const [isApproving, setIsApproving] = useState(false);
  const [isDepositing, setIsDepositing] = useState(false);
  const [hasPromptedChain, setHasPromptedChain] = useState(false);

  const { data: balanceData, refetch: refetchBalance } = useBalance({
    address,
    chainId: arbitrum.id,
    token: poolAssetAddress ?? undefined,
    query: {
      enabled: Boolean(open && address && poolAssetAddress),
    },
    watch: Boolean(address && poolAssetAddress),
  });

  const derivedTokenPriceUsd = useMemo(() => {
    if (Number.isFinite(tokenPriceUsd) && tokenPriceUsd > 0) {
      return tokenPriceUsd;
    }
    return 0;
  }, [tokenPriceUsd]);

  const amountWei = useMemo(() => {
    if (!amountToken) {
      return null;
    }
    try {
      return parseUnits(amountToken, assetDecimals);
    } catch (error) {
      return null;
    }
  }, [amountToken, assetDecimals]);

  const amountTokenNumber = useMemo(() => {
    const value = parseFloat(amountToken);
    return Number.isFinite(value) ? value : 0;
  }, [amountToken]);

  const approxUsd = useMemo(() => {
    if (derivedTokenPriceUsd > 0 && amountTokenNumber > 0) {
      return amountTokenNumber * derivedTokenPriceUsd;
    }
    return 0;
  }, [amountTokenNumber, derivedTokenPriceUsd]);

  const minDepositTokens = useMemo(() => {
    if (derivedTokenPriceUsd > 0 && game.minToResetUsd > 0) {
      return game.minToResetUsd / derivedTokenPriceUsd;
    }
    return 0;
  }, [derivedTokenPriceUsd, game.minToResetUsd]);

  const meetsMinimum = useMemo(() => {
    if (minDepositTokens === 0) {
      return amountTokenNumber > 0;
    }
    return amountTokenNumber >= minDepositTokens;
  }, [amountTokenNumber, minDepositTokens]);

  const { data: allowance, refetch: refetchAllowance } = useReadContract({
    address: poolAssetAddress ?? ZERO_ADDRESS,
    abi: erc20Abi,
    functionName: 'allowance',
    args:
      address && poolAssetAddress
        ? [address, appConfig.contracts.bozo as `0x${string}`]
        : undefined,
    chainId: arbitrum.id,
    query: {
      enabled: Boolean(open && address && poolAssetAddress),
    },
  });

  const allowanceValue = typeof allowance === 'bigint' ? allowance : 0n;
  const needsApproval = Boolean(
    poolAssetAddress && amountWei && allowanceValue < amountWei,
  );
  const isWrongChain =
    typeof chainId === 'number' && chainId !== arbitrum.id && isConnected;

  const maxAmount = useMemo(() => {
    if (!balanceData?.value) {
      return '';
    }

    try {
      return formatUnits(balanceData.value, balanceData.decimals);
    } catch (error) {
      console.error('Failed to format balance:', error);
      return '';
    }
  }, [balanceData]);

  const balanceDisplay = useMemo(() => {
    if (!balanceData?.formatted) {
      return '0';
    }

    const [whole, fraction = ''] = balanceData.formatted.split('.');
    const trimmedFraction = fraction.slice(0, 4).replace(/0+$/, '');
    return trimmedFraction ? `${whole}.${trimmedFraction}` : whole;
  }, [balanceData?.formatted]);

  const hasBalance = useMemo(() => {
    const numeric = parseFloat(maxAmount);
    return Number.isFinite(numeric) && numeric > 0;
  }, [maxAmount]);

  const isActionDisabled =
    !amountWei ||
    amountWei === 0n ||
    !poolAssetAddress ||
    isApproving ||
    isDepositing ||
    isWriting ||
    isSwitchingChain;

  const resetForm = () => {
    setAmountToken('');
    setComment('');
  };

  const ensureCorrectChain = useCallback(async () => {
    if (!isWrongChain) {
      return true;
    }

    try {
      if (switchChainAsync) {
        await switchChainAsync({ chainId: arbitrum.id });
        return true;
      }
    } catch (error) {
      console.error('Failed to switch chain:', error);
      toast.error('Please switch to Arbitrum in your wallet.');
      return false;
    }

    toast.error('Please switch to Arbitrum in your wallet.');
    return false;
  }, [isWrongChain, switchChainAsync]);

  const handleSetMax = useCallback(() => {
    if (!hasBalance || !maxAmount) {
      return;
    }

    setAmountToken(maxAmount);
  }, [hasBalance, maxAmount]);

  const handleApprove = async () => {
    if (!address || !poolAssetAddress) {
      toast.error('Pool asset information not available.');
      return;
    }

    if (!(await ensureCorrectChain())) {
      return;
    }

    if (!amountWei || amountWei === 0n) {
      toast.error('Enter an amount to approve.');
      return;
    }

    const approvalAmount = amountWei as bigint;

    try {
      setIsApproving(true);
      const txHash = await writeContractAsync({
        address: poolAssetAddress,
        abi: erc20Abi,
        functionName: 'approve',
        args: [appConfig.contracts.bozo as `0x${string}`, approvalAmount],
        chainId: arbitrum.id,
      });

      toast.success('Approval transaction submitted.');

      if (publicClient) {
        await publicClient.waitForTransactionReceipt({ hash: txHash });
        toast.success('Token approval confirmed.');
      }

      await refetchAllowance();
    } catch (error) {
      console.error('Approval failed:', error);
      const description =
        error instanceof Error ? error.message : 'Approval failed';
      toast.error('Approval failed', { description });
    } finally {
      setIsApproving(false);
    }
  };

  const handleBozo = async () => {
    if (!isConnected) {
      toast.error('Connect your wallet to BOZO.');
      return;
    }

    if (!address) {
      toast.error('Wallet address unavailable.');
      return;
    }

    if (!poolAssetAddress) {
      toast.error('Pool asset not detected. Try again shortly.');
      return;
    }

    if (!(await ensureCorrectChain())) {
      return;
    }

    if (!amountWei || amountWei === 0n) {
      toast.error('Enter an amount to deposit.');
      return;
    }

    if (!meetsMinimum) {
      toast.error('Deposit does not meet the current minimum.');
      return;
    }

    if (needsApproval) {
      toast.error('Approve the token before depositing.');
      return;
    }

    try {
      setIsDepositing(true);
      const nowSec = Math.floor(Date.now() / 1000);
      const deadline = BigInt(nowSec + 3600);

      const txHash = await writeContractAsync({
        address: appConfig.contracts.bozo as `0x${string}`,
        abi: bozoAbi,
        functionName: 'play',
        args: [
          poolAssetAddress,
          0n,
          deadline,
          amountWei,
          address,
        ],
        chainId: arbitrum.id,
      });

      toast.success('Deposit submitted. Waiting for confirmation...');

      if (publicClient) {
        await publicClient.waitForTransactionReceipt({ hash: txHash });
      }

      toast.success('Deposit confirmed. RIP BOZO! 🤡');

      if (comment.trim()) {
        try {
          await mockApi.commitDeposit({ txHash, comment: comment.trim() });
        } catch (error) {
          console.error('Failed to record comment:', error);
        }
      }

      await refetchBalance().catch((error) => {
        console.error('Failed to refresh balance:', error);
      });

      onOpenChange(false);
      resetForm();
    } catch (error) {
      console.error('Deposit failed:', error);
      const description =
        error instanceof Error ? error.message : 'Deposit failed';
      toast.error('Deposit failed', { description });
    } finally {
      setIsDepositing(false);
    }
  };

  useEffect(() => {
    if (!open || !poolAssetAddress || !address) {
      return;
    }

    void refetchBalance().catch((error) => {
      console.error('Failed to refresh balance:', error);
    });
  }, [open, poolAssetAddress, address, refetchBalance]);

  useEffect(() => {
    if (!open || !isWrongChain) {
      setHasPromptedChain(false);
      return;
    }

    if (isConnected && !hasPromptedChain) {
      setHasPromptedChain(true);
      void ensureCorrectChain();
    }
  }, [open, isConnected, isWrongChain, ensureCorrectChain, hasPromptedChain]);

  if (!isConnected) {
    return (
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="bg-[#1a1d32] border-border max-w-md">
          <DialogHeader>
            <DialogTitle className="text-foreground text-center">
              <div className="flex flex-col items-center gap-3 mb-4">
                <div className="w-16 h-16 rounded-full bg-[#FF4B4B] flex items-center justify-center">
                  <div className="w-8 h-8 rounded-full bg-[#FFF2E1]" />
                </div>
                <h3>BOZO</h3>
              </div>
            </DialogTitle>
            <DialogDescription className="sr-only">
              Connect your wallet to deposit into the Bozo pot
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <p className="text-center text-muted-foreground">
              Connect your wallet to deposit
            </p>
            <Button
              className="w-full bg-[#FF4B4B] hover:bg-[#FF4B4B]/90 text-[#FFF2E1]"
              size="lg"
            >
              Connect Wallet
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="bg-[#1a1d32] border-border max-w-md">
        <DialogHeader>
          <DialogTitle className="text-foreground text-center">
            <div className="flex flex-col items-center gap-3 mb-4">
              <div className="w-16 h-16 rounded-full bg-[#FF4B4B] flex items-center justify-center">
                <div className="w-8 h-8 rounded-full bg-[#FFF2E1]" />
              </div>
              <h3>BOZO</h3>
            </div>
          </DialogTitle>
          <DialogDescription className="sr-only">
            Deposit into the Bozo pot on Arbitrum
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="rounded-lg border border-border/40 bg-[#252840]/60 p-3 text-xs text-muted-foreground">
            <div className="flex justify-between">
              <span>Network</span>
              <span className="text-foreground">Arbitrum</span>
            </div>
            <div className="flex justify-between">
              <span>Token</span>
              <span className="text-foreground">{game.homeToken}</span>
            </div>
          </div>

          {isWrongChain && (
            <Alert className="bg-[#FF4B4B]/10 border-[#FF4B4B]">
              <AlertCircle className="h-4 w-4 text-[#FF4B4B]" />
              <AlertDescription className="text-sm">
                You are connected to the wrong network. Switch to Arbitrum to BOZO.
              </AlertDescription>
            </Alert>
          )}

          <div className="space-y-2">
            <div className="flex items-center justify-between gap-2">
              <Label className="text-sm text-muted-foreground">
                Amount ({game.homeToken})
              </Label>
              <div className="flex items-center gap-2 text-xs">
                <span className="text-muted-foreground">
                  Balance: {balanceDisplay}{' '}
                  {balanceData?.symbol ?? game.homeToken}
                </span>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="h-7 px-3 border-border/60 text-foreground hover:bg-[#2ED4B7]/10 hover:border-[#2ED4B7]/50"
                  onClick={handleSetMax}
                  disabled={!hasBalance}
                >
                  MAX
                </Button>
              </div>
            </div>
            <Input
              type="number"
              placeholder="0.00"
              value={amountToken}
              onChange={(e) => {
                setAmountToken(e.target.value);
              }}
              className="bg-[#252840] border-0 text-lg font-mono"
              min="0"
              step="0.000001"
            />
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">
                ≈ {formatUsd(approxUsd)} USD
              </span>
              <span className="text-[#2ED4B7]">
                Min reset ≈{' '}
                {minDepositTokens > 0
                  ? `${minDepositTokens.toFixed(4)} ${game.homeToken}`
                  : 'N/A'}
              </span>
            </div>
          </div>

          {amountToken && (
            <Alert
              className={
                meetsMinimum
                  ? 'bg-[#2ED4B7]/10 border-[#2ED4B7]'
                  : 'bg-[#FF4B4B]/10 border-[#FF4B4B]'
              }
            >
              {meetsMinimum ? (
                <CheckCircle className="h-4 w-4 text-[#2ED4B7]" />
              ) : (
                <AlertCircle className="h-4 w-4 text-[#FF4B4B]" />
              )}
              <AlertDescription className="text-sm">
                {meetsMinimum
                  ? 'Ready to BOZO. This meets the current minimum deposit.'
                  : 'Deposit is below the minimum required to reset the timer.'}
              </AlertDescription>
            </Alert>
          )}

          {needsApproval && (
            <Alert className="bg-[#F6C445]/10 border-[#F6C445]">
              <AlertCircle className="h-4 w-4 text-[#F6C445]" />
              <AlertDescription className="text-sm text-foreground">
                Approve {game.homeToken} to the Bozo contract before depositing.
              </AlertDescription>
            </Alert>
          )}

          <div className="bg-[#252840] rounded-lg p-4 space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">MINIMUM TO RESET</span>
              <span className="text-[#F6C445]">
                {formatUsd(game.minToResetUsd)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">YOU&apos;LL STAY LEADER FOR</span>
              <span className="text-foreground">5 MINUTES</span>
            </div>
          </div>

          <div className="space-y-2">
            <Label className="text-sm text-muted-foreground">
              Comment (optional)
            </Label>
            <Textarea
              placeholder="RIP BOZO 🤡"
              value={comment}
              onChange={(e) => setComment(e.target.value)}
              className="bg-[#252840] border-0 resize-none h-20"
              maxLength={140}
            />
            <div className="text-xs text-right text-muted-foreground">
              {comment.length}/140
            </div>
          </div>

          <div className="space-y-2">
            {needsApproval && (
              <Button
                onClick={handleApprove}
                disabled={
                  !amountWei ||
                  amountWei === 0n ||
                  isApproving ||
                  isSwitchingChain ||
                  isWrongChain
                }
                className="w-full bg-[#FF4B4B] hover:bg-[#FF4B4B]/90 text-[#FFF2E1]"
                size="lg"
              >
                {isApproving ? (
                  <>
                    <Loader2 className="w-5 h-5 mr-2 animate-spin" />
                    Approving...
                  </>
                ) : (
                  `Approve ${game.homeToken}`
                )}
              </Button>
            )}

            {isWrongChain && (
              <Button
                onClick={ensureCorrectChain}
                disabled={isSwitchingChain}
                className="w-full bg-[#1a1d32] border border-[#F6C445] text-[#F6C445] hover:bg-[#1a1d32]/80"
                size="lg"
              >
                {isSwitchingChain ? (
                  <>
                    <Loader2 className="w-5 h-5 mr-2 animate-spin" />
                    Switching...
                  </>
                ) : (
                  'Switch to Arbitrum'
                )}
              </Button>
            )}

            <Button
              onClick={handleBozo}
              disabled={isActionDisabled || needsApproval || isWrongChain}
              className="w-full bg-[#F6C445] hover:bg-[#F6C445]/90 text-[#0E1020]"
              size="lg"
            >
              {isDepositing || isWriting ? (
                <>
                  <Loader2 className="w-5 h-5 mr-2 animate-spin" />
                  Depositing...
                </>
              ) : (
                'BOZO'
              )}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
