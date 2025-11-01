import { useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from './ui/dialog';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { Textarea } from './ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { Checkbox } from './ui/checkbox';
import { Alert, AlertDescription } from './ui/alert';
import { formatUsd } from '../lib/utils';
import { GameState } from '../types';
import { Loader2, AlertCircle, CheckCircle } from 'lucide-react';
import { toast } from 'sonner@2.0.3';
import { mockApi } from '../lib/mock-api';

interface BozoModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  game: GameState;
  isConnected: boolean;
}

const CHAINS = [
  { value: 'base', label: 'Base' },
  { value: 'arbitrum', label: 'Arbitrum' },
  { value: 'polygon', label: 'Polygon' },
  { value: 'ethereum', label: 'Ethereum' }
];

const ASSETS = [
  { value: 'ETH', label: 'ETH' },
  { value: 'USDC', label: 'USDC' },
  { value: 'USDT', label: 'USDT' }
];

export function BozoModal({ open, onOpenChange, game, isConnected }: BozoModalProps) {
  const [sourceChain, setSourceChain] = useState('base');
  const [sourceAsset, setSourceAsset] = useState('ETH');
  const [amountUsd, setAmountUsd] = useState('');
  const [comment, setComment] = useState('');
  const [agreed, setAgreed] = useState(false);
  const [isQuoting, setIsQuoting] = useState(false);
  const [isDepositing, setIsDepositing] = useState(false);
  const [quote, setQuote] = useState<any>(null);

  const guaranteedMinutes = 5;

  const handleAmountChange = (value: string) => {
    setAmountUsd(value);
    setQuote(null);
    setAgreed(false);
  };

  const handleGetQuote = async () => {
    if (!amountUsd || parseFloat(amountUsd) <= 0) {
      toast.error('Please enter a valid amount');
      return;
    }

    setIsQuoting(true);
    try {
      const result = await mockApi.getRouteQuote({
        sourceChain,
        sourceAsset,
        amountSource: amountUsd,
        slippageBps: 50
      });
      setQuote(result);

      if (!result.meetsMinPct) {
        toast.error('Amount too low', {
          description: `Minimum required: ${formatUsd(game.minToResetUsd)}`
        });
      }
    } catch (error) {
      toast.error('Failed to get quote');
    } finally {
      setIsQuoting(false);
    }
  };

  const handleBozo = async () => {
    if (!quote) {
      await handleGetQuote();
      return;
    }

    if (!agreed) {
      toast.error('Please confirm your deposit');
      return;
    }

    setIsDepositing(true);
    try {
      await new Promise(resolve => setTimeout(resolve, 2000));

      if (sourceChain !== game.chain) {
        toast.success('Pending intent created — you\'re on the clock.');
      } else {
        toast.success('Deposit confirmed. Timer reset to 60:00. RIP BOZO! 🤡');
      }

      onOpenChange(false);
      setAmountUsd('');
      setComment('');
      setQuote(null);
      setAgreed(false);
    } catch (error) {
      toast.error('Deposit failed');
    } finally {
      setIsDepositing(false);
    }
  };

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

  const tokenEquiv = amountUsd && game.potUsd > 0 && parseFloat(game.potTokenAmount) > 0
    ? (parseFloat(amountUsd) / (game.potUsd / parseFloat(game.potTokenAmount))).toFixed(4)
    : '0.0000';

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
            Deposit into the Bozo pot using any chain or asset
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* Chain & Asset Selection */}
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-2">
              <Label className="text-sm text-muted-foreground">Chain</Label>
              <Select value={sourceChain} onValueChange={(val) => { setSourceChain(val); setQuote(null); setAgreed(false); }}>
                <SelectTrigger className="bg-[#252840] border-0">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {CHAINS.map(chain => (
                    <SelectItem key={chain.value} value={chain.value}>
                      {chain.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label className="text-sm text-muted-foreground">Asset</Label>
              <Select value={sourceAsset} onValueChange={(val) => { setSourceAsset(val); setQuote(null); setAgreed(false); }}>
                <SelectTrigger className="bg-[#252840] border-0">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {ASSETS.map(asset => (
                    <SelectItem key={asset.value} value={asset.value}>
                      {asset.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>

          {/* Amount Input */}
          <div className="space-y-2">
            <Label className="text-sm text-muted-foreground">YOU BOZO</Label>
            <Input
              type="number"
              placeholder="0.00"
              value={amountUsd}
              onChange={(e) => handleAmountChange(e.target.value)}
              className="bg-[#252840] border-0 text-lg font-mono"
            />
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">~{tokenEquiv} {sourceAsset}</span>
              <span className="text-[#2ED4B7]">{amountUsd ? formatUsd(parseFloat(amountUsd)) : '$0.00'}</span>
            </div>
          </div>

          {/* Info Display */}
          <div className="bg-[#252840] rounded-lg p-4 space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">GUARANTEED MINIMUM</span>
              <span className="text-[#F6C445]">{formatUsd(game.minToResetUsd)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">GUARANTEED TIME AS LEADER</span>
              <span className="text-foreground">{guaranteedMinutes} MINUTES</span>
            </div>
          </div>

          {/* Comment */}
          <div className="space-y-2">
            <Label className="text-sm text-muted-foreground">Comment (optional)</Label>
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

          {/* Quote Info */}
          {quote && (
            <Alert className={quote.meetsMinPct ? "bg-[#2ED4B7]/10 border-[#2ED4B7]" : "bg-[#FF4B4B]/10 border-[#FF4B4B]"}>
              {quote.meetsMinPct ? (
                <CheckCircle className="h-4 w-4 text-[#2ED4B7]" />
              ) : (
                <AlertCircle className="h-4 w-4 text-[#FF4B4B]" />
              )}
              <AlertDescription className="text-sm">
                <div className="space-y-1">
                  {quote.meetsMinPct ? (
                    <div className="text-foreground">Ready to BOZO</div>
                  ) : (
                    <div className="text-[#FF4B4B]">Below minimum amount</div>
                  )}
                  <div className="text-muted-foreground">ETA: ~{quote.estArrivalSec}s</div>
                  {sourceChain !== game.chain && (
                    <div className="text-[#F6C445]">Cross-chain bridging required</div>
                  )}
                </div>
              </AlertDescription>
            </Alert>
          )}

          {/* Confirmation Checkbox */}
          {quote && quote.meetsMinPct && (
            <div className="flex items-center space-x-2">
              <Checkbox
                id="agree"
                checked={agreed}
                onCheckedChange={(checked) => setAgreed(checked as boolean)}
                className="border-border data-[state=checked]:bg-[#FF4B4B] data-[state=checked]:border-[#FF4B4B]"
              />
              <label
                htmlFor="agree"
                className="text-sm text-foreground cursor-pointer select-none"
              >
                BOZO
              </label>
              {agreed && (
                <span className="text-xs text-[#2ED4B7] ml-auto">READY</span>
              )}
            </div>
          )}

          {/* Bozo Button */}
          <Button
            onClick={handleBozo}
            disabled={isQuoting || isDepositing || !amountUsd || (quote && !agreed)}
            className="w-full bg-[#F6C445] hover:bg-[#F6C445]/90 text-[#0E1020]"
            size="lg"
          >
            {isQuoting ? (
              <>
                <Loader2 className="w-5 h-5 mr-2 animate-spin" />
                Getting Quote...
              </>
            ) : isDepositing ? (
              <>
                <Loader2 className="w-5 h-5 mr-2 animate-spin" />
                Depositing...
              </>
            ) : quote ? (
              'BOZO'
            ) : (
              'GET QUOTE'
            )}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
