<template>
  <main>
    <div class="container mx-auto p-4">
      <h1 class="text-2xl font-bold mb-4">Liquidity Pools for ID: {{ id }}</h1>
      <div v-if="loading" class="text-center">Loading...</div>
      <div v-else-if="error" class="text-red-500">{{ error }}</div>
      <div class="mt-8">
        <svg id="liquidityChart" class="w-full h-96"></svg>
        <div class="mt-4">
          <p>Current Price: <span>{{ currentPrice }}</span></p>
          <p>Lower Bound: <span>{{ lowerBoundPrice }}</span></p>
          <p>Upper Bound: <span>{{ upperBoundPrice }}</span></p>
        </div>
      </div>
    </div>
  </main>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useRoute } from 'vue-router';
import * as d3 from 'd3';

// Route
const route = useRoute();
const id = ref(route.params.id as string);

// State
const loading = ref(true);
const error = ref<string | null>(null);
const currentPrice = ref<string | null>(null);
const lowerBoundPrice = ref<string | null>(null);
const upperBoundPrice = ref<string | null>(null);

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
      const finalUrl = useCorsProxy ? `https://corsproxy.io/?${encodeURIComponent(url)}` : url;
      const response = await fetch(finalUrl);

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
    const [{ ticks }, _temp, priceData] = await builder.fetchAndFlatten([
      {
        method: 'exchanges.getPoolTicks',
        body: {
          exchange: 'orca',
          network: 'solana',
          poolAddress: 'Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE',
          token0Decimals: 9,
          token1Decimals: 6,
        },
      },
      {
        method: 'exchanges.getPoolHistory',
        body: {
          exchange: 'orca',
          network: 'solana',
          poolAddress: 'Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE',
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
          poolAddress: 'Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE',
        },
      },
    ], true);

    return { ticks, priceData };
  } catch (err) {
    throw new Error('Error fetching data: ' + (err as Error).message);
  }
}

// Render chart with D3.js
function renderChart(tickData: any[], priceData: any) {
  if (!tickData || !priceData || !priceData.token0?.price) {
    console.error('Invalid data provided to renderChart');
    return;
  }

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

  const groupedData = Object.entries(bins).map(([binIndex, bin]) => ({
    binIndex: Number(binIndex),
    averagePrice: bin.priceSum / bin.tickCount,
    totalLiquidity: bin.liquiditySum,
  }));

  groupedData.sort((a, b) => a.averagePrice - b.averagePrice);
  const labels = groupedData.map(g => g.averagePrice.toFixed(1));
  const data = groupedData.map(g => Math.abs(g.totalLiquidity));

  const currentPriceTick = labels.reduce(
    (closestIdx, curr, idx) => (
      Math.abs(Number(curr) - priceData.token0.price) < Math.abs(Number(labels[closestIdx]) - priceData.token0.price) ? idx : closestIdx
    ),
    0
  );

  const lowerBound = Number(labels[currentPriceTick]) * 0.9;
  const lowerBoundTick = labels.reduce(
    (closestIdx, curr, idx) => (
      Math.abs(Number(curr) - lowerBound) < Math.abs(Number(labels[closestIdx]) - lowerBound) ? idx : closestIdx
    ),
    0
  );

  const upperBound = Number(labels[currentPriceTick]) * 1.1;
  const upperBoundTick = labels.reduce(
    (closestIdx, curr, idx) => (
      Math.abs(Number(curr) - upperBound) < Math.abs(Number(labels[closestIdx]) - upperBound) ? idx : closestIdx
    ),
    0
  );

  // Update reactive state
  currentPrice.value = labels[currentPriceTick];
  lowerBoundPrice.value = labels[lowerBoundTick];
  upperBoundPrice.value = labels[upperBoundTick];

  // Chart dimensions
  const margin = { top: 40, right: 30, bottom: 50, left: 60 };
  const width = 800 - margin.left - margin.right;
  const height = 384 - margin.top - margin.bottom; // h-96 = 384px

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
    .domain(labels)
    .range([0, width])
    .padding(0.1);

  const y = d3.scaleLinear()
    .domain([0, d3.max(data)! * 1.1])
    .range([height, 0]);

  // X-axis
  svg.append('g')
    .attr('transform', `translate(0,${height})`)
    .call(d3.axisBottom(x))
    .selectAll('text')
    .attr('transform', 'rotate(-45)')
    .style('text-anchor', 'end')
    .style('display', (d, i) => i % 100 === 0 ? null : 'none');

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
    .data(groupedData)
    .enter()
    .append('rect')
    .attr('class', 'bar')
    .attr('x', d => x(d.averagePrice.toFixed(1))!)
    .attr('y', d => y(Math.abs(d.totalLiquidity)))
    .attr('width', x.bandwidth())
    .attr('height', d => height - y(Math.abs(d.totalLiquidity)))
    .attr('fill', 'lightblue');

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

  // Drag behavior
  const drag = d3.drag<SVGLineElement, unknown>()
    .on('drag', function (event) {
      const xPos = event.x;
      // Find closest price
      const closestPrice = labels.reduce((closest, curr) => (
        Math.abs(priceToX(curr) - xPos) < Math.abs(priceToX(closest) - xPos) ? curr : closest
      ), labels[0]);

      // Update line position and reactive state
      if (d3.select(this).classed('lower-bound')) {
        lowerBoundPrice.value = closestPrice;
        d3.select(this)
          .attr('x1', priceToX(closestPrice))
          .attr('x2', priceToX(closestPrice));
      } else if (d3.select(this).classed('upper-bound')) {
        upperBoundPrice.value = closestPrice;
        d3.select(this)
          .attr('x1', priceToX(closestPrice))
          .attr('x2', priceToX(closestPrice));
      }
    });

  // Apply drag to bound lines
  lowerLine.call(drag);
  upperLine.call(drag);
}

// Fetch and load data
async function loadData() {
  loading.value = true;
  error.value = null;
  try {
    const response = await fetchLiquidityData(id.value);
    renderChart(response.ticks, response.priceData);
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

// Initial load
onMounted(loadData);
</script>