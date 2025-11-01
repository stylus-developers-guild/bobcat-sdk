import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Avatar, AvatarFallback } from './ui/avatar';
import { ScrollArea } from './ui/scroll-area';
import { formatAddress, formatTokenAmount, formatUsd } from '../lib/utils';
import { Deposit } from '../types';
import { useComments } from '../providers/CommentsProvider';

interface DepositsFeedProps {
  deposits: Deposit[];
  homeToken: string;
}

export function DepositsFeed({ deposits, homeToken }: DepositsFeedProps) {
  const { getCommentForTxHash } = useComments();

  const formatTime = (ts: string) => {
    const date = new Date(ts);
    const now = Date.now();
    const diff = now - date.getTime();
    const minutes = Math.floor(diff / 60000);
    
    if (minutes < 1) return 'Just now';
    if (minutes === 1) return '1m ago';
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    if (hours === 1) return '1h ago';
    if (hours < 24) return `${hours}h ago`;
    return `${Math.floor(hours / 24)}d ago`;
  };

  return (
    <Card className="bg-card border-border h-full">
      <CardHeader>
        <CardTitle className="text-foreground">RECENT BOZOS</CardTitle>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-[600px] pr-4">
          <div className="space-y-3">
            {deposits.map((deposit) => {
              const commentText = getCommentForTxHash(deposit.txHash);
              return (
                <div
                  key={deposit.txHash}
                  className="flex items-start gap-3 p-4 rounded-lg bg-[#252840] hover:bg-[#252840]/80 transition-colors"
                >
                  <Avatar className="w-10 h-10 mt-1">
                    <AvatarFallback className="bg-[#FF4B4B] text-[#FFF2E1] text-xs">
                      {deposit.handle?.[0]?.toUpperCase() || deposit.address.slice(2, 4).toUpperCase()}
                    </AvatarFallback>
                  </Avatar>

                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-foreground">
                        {deposit.handle || formatAddress(deposit.address)}
                      </span>
                      <span className="text-xs text-muted-foreground">
                        {formatTime(deposit.ts)}
                      </span>
                    </div>

                    <div className="text-sm text-muted-foreground mb-2">
                      BOZO&apos;D {formatTokenAmount(deposit.amountToken, 4)} {homeToken}
                    </div>

                    {commentText && (
                      <div className="text-sm text-foreground bg-[#1a1d32] rounded px-3 py-2 mb-2">
                        {commentText}
                      </div>
                    )}

                    <div className="flex items-center gap-3 text-xs">
                      <div className="text-[#2ED4B7]">
                        +{formatUsd(deposit.amountUsd).replace('$', '')}
                      </div>
                      <div className="text-muted-foreground">
                        Pot after: {formatUsd(deposit.potAfterUsd)}
                      </div>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
