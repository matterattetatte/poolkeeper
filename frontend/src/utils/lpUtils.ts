// src/utils/lpUtils.ts

// Interface for dailyData
export interface DailyData {
  date: string;
  feesUSD: number;
  volumeUSD: number;
  price: number;
  ticks: { tick: number; liquidity: number }[];
}

// Interface for calculateDayAPR return type
export interface DayAPRData {
  date: string;
  feesEarned: number;
  price: number;
  dailyAPR: number;
}

/**
 * Calculate daily fees and APR earned by a position in a concentrated liquidity pool.
 * @param dayIndex - Index of the day (0 = latest day).
 * @param dailyData - Array of daily snapshots with { date, volume, price, ticks: [{tick, liquidity}] }.
 * @param lowerTick - Lower tick boundary of position.
 * @param upperTick - Upper tick boundary of position.
 * @param positionLiquidity - Total liquidity value of position at day.
 * @param volumeFee - Fee rate as decimal (e.g., 0.003 for 0.3%).
 * @returns {DayAPRData} - { date, volumeInRange, feesEarned, price, dailyAPR }
 */
export function calculateDayAPR(
  dayIndex: number,
  dailyData: DailyData[],
  lowerTick: number,
  upperTick: number,
  positionLiquidity: number,
): DayAPRData {
  if (dayIndex < 0 || dayIndex >= dailyData.length) {
    throw new Error('Invalid day index');
  }

  const day = dailyData[dayIndex];
  const { date, feesUSD, price, ticks } = day;

  const ticksInRange = ticks.filter(({ tick }) => tick >= lowerTick && tick <= upperTick);
  const liquidityInRange = ticksInRange.reduce((sum, t) => sum + t.liquidity, 0);
  const positionShare = liquidityInRange > 0 ? 10e9 * positionLiquidity / liquidityInRange : 0;
  const feesEarned = feesUSD * positionShare;
  const dailyReturn = positionLiquidity > 0 ? feesEarned / positionLiquidity : 0;
  const dailyAPR = dailyReturn * 365;

  return { date, feesEarned, price, dailyAPR };
}

/**
 * Calculate average APR over last N days.
 * @param daysCount - Number of days to track (e.g., 30).
 * @param dailyData - Array of daily liquidity/volume snapshots.
 * @param lowerTick - Position lower tick bound.
 * @param upperTick - Position upper tick bound.
 * @param positionLiquidity - Position liquidity value per day.
 * @param volumeFee - Fee rate decimal (e.g., 0.003).
 * @returns {Object} - { averageAPR, dailyAPRArray } with daily APRs and average.
 */
export function calculateAverageAPR(
  daysCount: number,
  dailyData: DailyData[],
  lowerTick: number,
  upperTick: number,
  positionLiquidity: number,
): { averageAPR: number; dailyAPRArray: DayAPRData[] } {
  const dailyAPRArray: DayAPRData[] = [];
  let aprSum = 0;

  for (let i = 0; i < daysCount; i++) {
    if (i >= dailyData.length) break;
    const dayAPRData = calculateDayAPR(i, dailyData, lowerTick, upperTick, positionLiquidity);
    dailyAPRArray.push(dayAPRData);
    aprSum += dayAPRData.dailyAPR;
  }

  const averageAPR = daysCount > 0 ? aprSum / Math.min(daysCount, dailyData.length) : 0;
  return { averageAPR, dailyAPRArray };
}

/**
 * Process tick data to compute cumulative liquidity and bin data for charting.
 * @param tickData - Array of tick objects with { tickIdx, liquidityNet, price0 }.
 * @returns Array of binned data with { binIndex, averagePrice, totalLiquidity }.
 */
export function processTicks(tickData: any[]): { binIndex: number; averagePrice: number; totalLiquidity: number }[] {
  if (!tickData.length) return [];

  const binSize = 1;
  const sortedTicks = tickData.slice().sort((a, b) => Number(a.tickIdx) - Number(b.tickIdx));
  let cumulativeLiquidity = 0;
  const ticksWithCumLiquidity = sortedTicks.map(tick => {
    cumulativeLiquidity += Number(tick.liquidityNet);
    return {
      tickIdx: Number(tick.tickIdx),
      price0: Number(tick.price0),
      cumulativeLiquidity,
    };
  });

  const bins: Record<string, { tickCount: number; liquiditySum: number; priceSum: number }> = {};
  ticksWithCumLiquidity.forEach(({ tickIdx, price0, cumulativeLiquidity }) => {
    const binIndex = Math.floor(tickIdx / binSize);
    if (!bins[binIndex]) {
      bins[binIndex] = { tickCount: 0, liquiditySum: 0, priceSum: 0 };
    }
    bins[binIndex].tickCount++;
    bins[binIndex].liquiditySum += cumulativeLiquidity;
    bins[binIndex].priceSum += price0;
  });

  return Object.entries(bins)
    .map(([binIndex, bin]) => ({
      binIndex: Number(binIndex),
      averagePrice: bin.priceSum / bin.tickCount,
      totalLiquidity: bin.liquiditySum,
    }))
    .sort((a, b) => a.averagePrice - b.averagePrice);
}

/**
 * Create a price-to-tick mapping from tick data.
 * @param tickData - Array of tick objects with { tickIdx, price0 }.
 * @returns Record mapping formatted prices to tick indices.
 */
export function createPriceToTickMap(tickData: any[]): Record<string, number> {
  const priceTickMap: Record<string, number> = {};
  tickData.forEach(({ tickIdx, price0 }) => {
    priceTickMap[Number(price0).toFixed(1)] = Number(tickIdx);
  });
  return priceTickMap;
}

/**
 * Generate a single-day snapshot for dailyData (placeholder).
 * @param tickData - Array of tick objects with { tickIdx, liquidityNet, price0 }.
 * @param priceData - Object with { token0: { price } }.
 * @returns Array of DailyData for APR calculations.
 */
export function generateDailyData(tickData: any[], priceData: any, historyData: any[]): DailyData[] {
  if (!tickData.length || !priceData?.token0?.price) return [];
  const sortedTicks = tickData.slice().sort((a, b) => Number(a.tickIdx) - Number(b.tickIdx));
  let cumulativeLiquidity = 0;
  const ticksWithCumLiquidity = sortedTicks.map(tick => {
    cumulativeLiquidity += Number(tick.liquidityNet);
    return {
      tickIdx: Number(tick.tickIdx),
      cumulativeLiquidity,
    };
  });

  return [
    {
      date: new Date().toISOString().split('T')[0],
      volumeUSD: Number(historyData?.slice(-2)[0]?.volumeUSD || 0), // last full days data
      feesUSD: Number(historyData?.slice(-2)[0]?.feesUSD || 0), // assuming 0.3%
      price: priceData.token0.price,
      ticks: ticksWithCumLiquidity.map(({ tickIdx, cumulativeLiquidity }) => ({
        tick: tickIdx,
        liquidity: cumulativeLiquidity,
      })),
    },
  ];
}

export const indexBy = <T>(array: T[], func: (item: T) => string): Map<string, T> => {
  const map = new Map<string, T>();
  array.forEach(item => {
    const key = func(item);
    map.set(key, item); // overwrite if key repeats, so keys must be unique
  });
  return map;
};
