import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { YieldPosition } from './entities/yield-position.entity';
import { YieldPositionResponseDto } from './dto/yield-position-response.dto';

@Injectable()
export class SavingsService {
  constructor(
    @InjectRepository(YieldPosition)
    private readonly yieldPositionRepository: Repository<YieldPosition>,
  ) {}

  ping(): { status: string } {
    return { status: 'ok' };
  }

  /**
   * Get the caller's yield-adapter position.
   *
   * Returns shares, estimated asset value (based on last-known exchange rate),
   * and pending withdrawal cooldown status.
   *
   * If no position exists, returns a well-formed empty response (not an error).
   */
  async getYieldPosition(
    ownerAddress: string,
  ): Promise<YieldPositionResponseDto> {
    const position = await this.yieldPositionRepository.findOne({
      where: { owner: ownerAddress },
    });

    if (!position) {
      return {
        address: ownerAddress,
        shares: '0',
        estimated_asset_value: null,
        exchange_rate_snapshot: null,
        pending_withdrawal_claimable_at: null,
        updated_at: new Date(),
      };
    }

    // Calculate estimated asset value from shares and exchange rate
    let estimatedValue: string | null = null;
    if (position.exchange_rate_snapshot && position.shares !== '0') {
      try {
        const shares = BigInt(position.shares);
        const rate = parseFloat(position.exchange_rate_snapshot);
        if (rate > 0) {
          estimatedValue = Math.floor(Number(shares) * rate).toString();
        }
      } catch {
        // If calculation fails, leave as null
      }
    }

    return {
      address: position.owner,
      shares: position.shares,
      estimated_asset_value: estimatedValue,
      exchange_rate_snapshot: position.exchange_rate_snapshot,
      pending_withdrawal_claimable_at: null, // Will be populated by indexer when withdrawals are tracked
      updated_at: position.updated_at,
    };
  }
}
