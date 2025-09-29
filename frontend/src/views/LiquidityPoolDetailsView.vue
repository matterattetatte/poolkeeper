<template>
  <main>
    <div class="container mx-auto p-4">
      <h1 class="text-2xl font-bold mb-4">Liquidity Pools for ID: {{ id }}</h1>
      <div v-if="loading" class="text-center">Loading...</div>
      <div v-else-if="error" class="text-red-500">{{ error }}</div>
      <div class="mt-8">
        <svg id="liquidityChart" class="w-full h-96"></svg>
        <div class="mt-4">
          <label for="dateRange" class="block mb-2 font-medium">Select Date (for historical data):</label>
          <input
            type="range"
            id="dateRange"
            name="dateRange"
            min="-30"
            max="0"
            value="0"
            step="1"
            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
          >
          <p>Date Price: <span>{{ currentPrice }}</span></p>
          <p>Lower Bound: <span>{{ streamedLowerline }} ({{ - ((1 - streamedLowerline / currentPrice) * 100).toFixed(2) }} %)</span></p>
          <p>Upper Bound: <span>{{ streamedUpperline }} ({{ (((streamedUpperline - currentPrice) / currentPrice) * 100).toFixed(2) }} %)</span></p>
          <p>APR based on current LP distribution, current price, and volume last 24h: <span>{{ (aprData?.dailyAPR?.dailyAPR * 100).toFixed(2) || 'N/A' }}%</span></p>
          <p>Average APR (30 days): <span>{{ (aprData?.averageAPR?.averageAPR * 100).toFixed(2) || 'N/A' }}%</span></p>
        </div>
      </div>
    </div>
  </main>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRoute } from 'vue-router';
import * as d3 from 'd3';
import { calculateDayAPR, calculateAverageAPR, processTicks, createPriceToTickMap, generateDailyData, DailyData, DayAPRData } from '@/utils/lpUtils';

// Route
const route = useRoute();
const id = ref(route.params.id as string);

// State
const loading = ref(true);
const error = ref<string | null>(null);
const tickData = ref<any[]>([]);
const priceData = ref<any>(null);
const positionLiquidity = ref(1000); // Configurable position liquidity
const daysCount = ref(30); // Number of days for average APR
const historyData = ref<any[]>([]);
const selectedDate = ref<string | null>(null);

// Computed properties
const groupedData = computed(() => processTicks(tickData.value));
const labels = computed(() => groupedData.value.map(g => g.averagePrice.toFixed(1)));
const data = computed(() => groupedData.value.map(g => Math.abs(g.totalLiquidity)));

const priceToTick = computed(() => createPriceToTickMap(tickData.value));

const currentPrice = computed(() => {
  if (!priceData.value?.token0?.price || !labels.value.length) return null;
  const currentPriceTick = labels.value.reduce(
    (closestIdx, curr, idx) => (
      Math.abs(Number(curr) - priceData.value.token0.price) < Math.abs(Number(labels.value[closestIdx]) - priceData.value.token0.price) ? idx : closestIdx
    ),
    0
  );
  return labels.value[currentPriceTick];
});

const lowerBoundPrice = ref<string | null>(null);
const upperBoundPrice = ref<string | null>(null);
const streamedLowerline = ref<number | null>(null);
const streamedUpperline = ref<number | null>(null);

const initialBounds = computed(() => {
  if (!currentPrice.value || !labels.value.length) return { lower: null, upper: null };
  const lowerBound = Number(currentPrice.value) * 0.9;
  const upperBound = Number(currentPrice.value) * 1.1;
  const lowerBoundTick = labels.value.reduce(
    (closestIdx, curr, idx) => (
      Math.abs(Number(curr) - lowerBound) < Math.abs(Number(labels.value[closestIdx]) - lowerBound) ? idx : closestIdx
    ),
    0
  );
  const upperBoundTick = labels.value.reduce(
    (closestIdx, curr, idx) => (
      Math.abs(Number(curr) - upperBound) < Math.abs(Number(labels.value[closestIdx]) - upperBound) ? idx : closestIdx
    ),
    0
  );
  return { lower: labels.value[lowerBoundTick], upper: labels.value[upperBoundTick] };
});

// Initialize bound prices reactively
watch(initialBounds, (newBounds) => {
  if (newBounds.lower && newBounds.upper) {
    lowerBoundPrice.value = newBounds.lower;
    upperBoundPrice.value = newBounds.upper;

    streamedLowerline.value = Number(newBounds.lower);
    streamedUpperline.value = Number(newBounds.upper);
  }
}, { immediate: true });

// later: array grouped with a slider that make ssure to display the right data for the day
const dailyData = computed(() => generateDailyData(tickData.value, priceData.value, historyData.value));

const aprData = computed((): { dailyAPR: DayAPRData | null; averageAPR: { averageAPR: number; dailyAPRArray: DayAPRData[] } | null } | null => {
  if (!dailyData.value.length || !streamedLowerline.value || !streamedUpperline.value) {
    return null;
  }
  try {
    const closestLowerTick = Object.keys(priceToTick.value).reduce(
      (closest, curr) =>
        Math.abs(Number(curr) - streamedLowerline.value) < Math.abs(Number(closest) - streamedLowerline.value)
          ? curr
          : closest,
      Object.keys(priceToTick.value)[0]
    );
    const lowerTickValue = priceToTick.value[closestLowerTick];

    const closestUpperTick = Object.keys(priceToTick.value).reduce(
      (closest, curr) =>
        Math.abs(Number(curr) - streamedUpperline.value) < Math.abs(Number(closest) - streamedUpperline.value)
          ? curr
          : closest,
      Object.keys(priceToTick.value)[0]
    );
    const upperTickValue = priceToTick.value[closestUpperTick];

    // check if we are out of bounds, lower tick higher than current price or upper tick lower than current price
    if ((lowerTickValue > priceToTick.value[currentPrice.value!]!) || (upperTickValue < priceToTick.value[currentPrice.value!]!)) {
      return { dailyAPR: { date: '', feesEarned: 0, price: 0, dailyAPR: 0 }, averageAPR: { averageAPR: 0, dailyAPRArray: [] } };
    }

    const dailyAPR = calculateDayAPR(
      0,
      dailyData.value,
      lowerTickValue,
      upperTickValue,
      positionLiquidity.value
    );

    const averageAPR = calculateAverageAPR(
      daysCount.value,
      dailyData.value,
      lowerTickValue,
      upperTickValue,
      positionLiquidity.value
    );

    return {
      dailyAPR,
      averageAPR
    };
  } catch (err) {
    console.error('Error calculating APR data:', err);
    return null;
  }
});

interface Call {
  method: string;
  body: Record<string, any>;
}

interface ApiResponse {
  result?: {
    data?: {
      json?: Record<string, any>;
    };
  };
}

class MetrixApiBatchUrlBuilder {
  private baseUrl: string;

  constructor(baseUrl: string = 'https://app.metrix.finance/api/trpc') {
    this.baseUrl = baseUrl.endsWith('/') ? baseUrl.slice(0, -1) : baseUrl;
  }

  buildFromCalls(calls: Call[]): string {
    const methodsPart = calls.map(call => call.method).join(',');
    const inputObj: Record<string, { json: Record<string, any> }> = {};
    calls.forEach((call, idx) => {
      inputObj[idx] = { json: call.body };
    });
    const inputJson = JSON.stringify(inputObj);
    const encodedInput = encodeURIComponent(inputJson);
    return `${this.baseUrl}/${methodsPart}?batch=1&input=${encodedInput}`;
  }

  async fetchAndFlatten(calls: Call[], useCorsProxy: boolean = false): Promise<Record<string, any>[]> {
    try {
      const url = this.buildFromCalls(calls);
      // const finalUrl = useCorsProxy ? `https://corsproxy.io/?${encodeURIComponent(url)}` : url;
      // const response = await fetch(finalUrl);
      const response = await fetch('/mockdata_28_09.json');

      if (!response.ok) {
        throw new Error(`Failed to fetch data: ${response.status} ${response.statusText}`);
      }

      const data: ApiResponse[] = await response.json();
      return data.map((item, index) => {
        if (!item.result?.data?.json) {
          throw new Error(`Invalid response structure for call at index ${index}`);
        }
        return item.result.data.json;
      });
    } catch (err) {
      throw new Error(`Error fetching and flattening data: ${(err as Error).message}`);
    }
  }
}

// Fetch liquidity data
async function fetchLiquidityData(poolId: string) {
  try {
    const builder = new MetrixApiBatchUrlBuilder();
    const [{ ticks }, { dailyHistory }, priceDataResponse] = await builder.fetchAndFlatten([
      {
        method: 'exchanges.getPoolTicks',
        body: {
          exchange: 'orca',
          network: 'solana',
          poolAddress: poolId,
          token0Decimals: 9,
          token1Decimals: 6,
        },
      },
      {
        method: 'exchanges.getPoolHistory',
        body: {
          exchange: 'orca',
          network: 'solana',
          poolAddress: poolId,
          feeTier: '400',
          apiKey: 1,
          baseTokenAddress: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
        },
      },
      {
        method: 'exchanges.getSimulatePool',
        body: {
          apiKey: 1,
          baseTokenAddress: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
          exchange: 'orca',
          network: 'solana',
          poolAddress: poolId,
        },
      },
    ], true);

    return { ticks, priceData: priceDataResponse, dailyHistory };
  } catch (err) {
    throw new Error('Error fetching data: ' + (err as Error).message);
  }
}

// Render chart with D3.js
function renderChart() {
  if (!groupedData.value.length || !currentPrice.value || !lowerBoundPrice.value || !upperBoundPrice.value) {
    console.error('Invalid data for rendering chart');
    return;
  }

  // Chart dimensions
  const margin = { top: 40, right: 30, bottom: 50, left: 60 };
  const width = 800 - margin.left - margin.right;
  const height = 384 - margin.top - margin.bottom;

  // Clear existing SVG content
  d3.select('#liquidityChart').selectAll('*').remove();

  // Create SVG
  const svg = d3.select('#liquidityChart')
    .attr('width', width + margin.left + margin.right)
    .attr('height', height + margin.top + margin.bottom)
    .append('g')
    .attr('transform', `translate(${margin.left},${margin.top})`);


  // Scales
  const x = d3.scaleBand()
    .domain(labels.value)
    .range([0, width])
    .padding(0.1);

  const y = d3.scaleLinear()
    .domain([0, d3.max(data.value)! * 1.1])
    .range([height, 0]);

  // X-axis with reduced labels
  svg.append('g')
    .attr('transform', `translate(0,${height})`)
    .call(d3.axisBottom(x).tickValues(labels.value.filter((_, i) => i % 100 === 0))) // Show every 5th label
    .selectAll('text')
    .attr('transform', 'rotate(-45)')
    .style('text-anchor', 'end');

  // Y-axis
  svg.append('g')
    .call(d3.axisLeft(y));

  // X-axis label
  svg.append('text')
    .attr('x', width / 2)
    .attr('y', height + margin.bottom - 10)
    .style('text-anchor', 'middle')
    .text('Price');

  // Y-axis label
  svg.append('text')
    .attr('transform', 'rotate(-90)')
    .attr('x', -height / 2)
    .attr('y', -margin.left + 20)
    .style('text-anchor', 'middle')
    .text('Liquidity');

  // Title
  svg.append('text')
    .attr('x', width / 2)
    .attr('y', -margin.top / 2)
    .style('text-anchor', 'middle')
    .style('font-size', '16px')
    .style('font-weight', 'bold')
    .text('Liquidity Pool Distribution');

  // Bars
  svg.selectAll('.bar')
    .data(groupedData.value)
    .enter()
    .append('rect')
    .attr('class', 'bar')
    .attr('x', d => x(d.averagePrice.toFixed(1))!)
    .attr('y', d => y(Math.abs(d.totalLiquidity)))
    .attr('width', x.bandwidth())
    .attr('height', d => height - y(Math.abs(d.totalLiquidity)))
    .attr('fill', 'blue');

  // Helper function to convert price to x-coordinate
  const priceToX = (price: string) => {
    const xVal = x(price);
    return xVal !== undefined ? xVal + x.bandwidth() / 2 : 0;
  };

  // Current price line
  svg.append('line')
    .attr('class', 'current-price')
    .attr('x1', priceToX(currentPrice.value!))
    .attr('x2', priceToX(currentPrice.value!))
    .attr('y1', 0)
    .attr('y2', height)
    .attr('stroke', 'red')
    .attr('stroke-width', 2);

  // Lower bound line
  const lowerLine = svg.append('line')
    .attr('class', 'lower-bound')
    .attr('x1', priceToX(lowerBoundPrice.value!))
    .attr('x2', priceToX(lowerBoundPrice.value!))
    .attr('y1', 0)
    .attr('y2', height)
    .attr('stroke', 'green')
    .attr('stroke-width', 6)
    .attr('stroke-dasharray', '5,5')
    .style('cursor', 'ew-resize');

  // Upper bound line
  const upperLine = svg.append('line')
    .attr('class', 'upper-bound')
    .attr('x1', priceToX(upperBoundPrice.value!))
    .attr('x2', priceToX(upperBoundPrice.value!))
    .attr('y1', 0)
    .attr('y2', height)
    .attr('stroke', 'green')
    .attr('stroke-width', 6)
    .attr('stroke-dasharray', '5,5')
    .style('cursor', 'ew-resize');

  // Assuming priceToX is a function that maps a price to an x-coordinate
// and labels.value is an array of price values
// lowerLine, upperLine are D3 selections for <line> elements
// lowerCircle, upperCircle are D3 selections for <circle> elements
// lowerBoundPrice, upperBoundPrice are reactive variables (e.g., Vue ref or D3 local)
const drag = d3
  .drag<SVGLineElement, unknown>()
  .on("drag", function (event) {
    // Get mouse coordinates relative to the parent <g> element
    const [newX] = d3.pointer(event, this.parentNode); // Adjust for <g> transforms
    // Constrain x position to chart bounds
    const constrainedX = Math.max(0, Math.min(width, newX));

    // Find the closest price from labels.value
    const closestPrice = labels.value.reduce(
      (closest, curr) =>
        Math.abs(priceToX(curr) - constrainedX) < Math.abs(priceToX(closest) - constrainedX)
          ? curr
          : closest,
      labels.value[0]
    );

    // Check if this is the lower or upper bound line
    const isLowerBound = d3.select(this).classed("lower-bound");
    const isUpperBound = d3.select(this).classed("upper-bound");

    if (isLowerBound) streamedLowerline.value = Number(closestPrice);
    if (isUpperBound) streamedUpperline.value = Number(closestPrice);

    // Update line position immediately (avoid reactive conflict)
    if (isLowerBound && Number(closestPrice) <= Number(upperBoundPrice.value)) {
      d3.select(this)
        .attr("x1", priceToX(closestPrice))
        .attr("x2", priceToX(closestPrice));
      // Defer reactive update to dragend
      d3.select(this).datum({ price: closestPrice });
    } else if (isUpperBound && Number(closestPrice) >= Number(lowerBoundPrice.value)) {
      d3.select(this)
        .attr("x1", priceToX(closestPrice))
        .attr("x2", priceToX(closestPrice));
      // Defer reactive update to dragend
      d3.select(this).datum({ price: closestPrice });
    } else {
      // Snap back to last valid position
      const currentPrice = isLowerBound ? lowerBoundPrice.value : upperBoundPrice.value;
      d3.select(this)
        .attr("x1", priceToX(currentPrice))
        .attr("x2", priceToX(currentPrice));
    }

  })
  .on("end", function () {
    // Update reactive state only when drag ends
    const isLowerBound = d3.select(this).classed("lower-bound");
    const isUpperBound = d3.select(this).classed("upper-bound");
    const newPrice = d3.select(this).datum()?.price;

    if (isLowerBound && newPrice != null && Number(newPrice) <= Number(upperBoundPrice.value)) {
      lowerBoundPrice.value = newPrice;
    } else if (isUpperBound && newPrice != null && Number(newPrice) >= Number(lowerBoundPrice.value)) {
      upperBoundPrice.value = newPrice;
    }
  });

// Apply drag behavior to the lines
lowerLine.call(drag);
upperLine.call(drag);
}

// Fetch and load data
async function loadData() {
  loading.value = true;
  error.value = null;
  try {
    const response = await fetchLiquidityData(id.value);
    tickData.value = response.ticks;
    priceData.value = response.priceData;
    historyData.value = response.dailyHistory || [];
  } catch (err) {
    console.error(err);
    error.value = (err as Error).message;
  } finally {
    loading.value = false;
  }
}

// Watch for route changes
watch(() => route.params.id, (newId) => {
  id.value = newId as string;
  loadData();
});

// Watch for changes in computed properties to re-render chart
watch([groupedData, currentPrice, lowerBoundPrice, upperBoundPrice], () => {
  renderChart();
});

// Initial load
onMounted(loadData);
</script>