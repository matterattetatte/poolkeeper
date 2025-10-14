<template>
  <main>
    <div class="container mx-auto p-4">
      <h1 class="text-2xl font-bold mb-4">Details for {{ route.query.name }}</h1>
      <div class="mt-8">
        <div class="w-full flex justify-end mb-2">
          <ComingSoon>
            <DropInButton />
          </ComingSoon>   
        </div>
        <div style="min-height: 350px">
          <div v-if="loading" class="text-center w-full">Loading...</div>
          <div v-else-if="error" class="text-red-500">{{ error }}</div>
          <div class="w-full flex">
            <svg id="liquidityChart" style="min-width: 70%; min-height: 100%;"></svg>
            <div v-if="!loading" class="mt-8">
              <h2>
                {{ (aprData?.dailyAPR?.dailyAPR * 100).toFixed(2) || 'N/A' }}%
              </h2>
              <p>APR based on selected date's LP distribution, price, and volume: </p>
              <hr style="color:var(--color-primary); margin-top:16px" />
              <h2>
                {{ (aprData?.averageAPR?.averageAPR * 100).toFixed(2) || 'N/A' }}%
              </h2>
              <p>Average backtracked APR (up to 30 days from selected date)</p>
              <!-- TODO: FIX LATER -->
              <!-- My position liquidity size:
              <div class="mt-2">
                <input
                  type="number"
                  v-model.number="positionLiquidity"
                  class="border border-gray-300 rounded px-2 py-1 w-32"
                />
              </div> -->
            </div>
          </div>
        </div>
        <div class="mt-4">
          <label for="dateRange" class="block mb-2 font-medium">
            Select Date (Current: {{ displayedDate }})
          </label>
          <input
            type="range"
            id="dateRange"
            name="dateRange"
            min="-30"
            max="0"
            v-model="selectedDayOffset"
            step="1"
            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
          >
          <p>Date Price: <span>{{ activePrice }}</span></p>
          <p>Lower Bound: <span>{{ streamedLowerline }} ({{ - ((1 - streamedLowerline / activePrice) * 100).toFixed(2) }} %)</span></p>
          <p>Upper Bound: <span>{{ streamedUpperline }} ({{ (((streamedUpperline - activePrice) / activePrice) * 100).toFixed(2) }} %)</span></p>
          <p>
            For technical analysis:
            <a
              :href="`https://dexscreener.com/solana/${route.params.id}`"
              target="_blank"
              rel="noopener noreferrer"
              class="text-blue-500 underline"
            >Dexscreener
            </a>
          </p>
        </div>
      </div>
    </div>
    <ComingSoon>
      <button>
        Compare with other pools...
      </button>
    </ComingSoon>
      <AIBotVerdict v-if="fullLPData" :fullLPData="fullLPData" />
  </main>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRoute } from 'vue-router';
import * as d3 from 'd3';
import { calculateDayAPR, calculateAverageAPR, processTicks, createPriceToTickMap, generateDailyData, DailyData, DayAPRData, groupBy, indexBy } from '@/utils/lpUtils';
import supabase from '@/lib/supabase';
import ComingSoon from '@/components/ComingSoon.vue';
import AIBotVerdict from '@/components/AIBotVerdict.vue';
import DropInButton from '@/components/DropInButton.vue';

const todaysDate = new Date().toISOString().slice(0, 10);
// Route
const route = useRoute();

// State
const loading = ref(true);
const error = ref<string | null>(null);
const fullLPData = ref<any>(null);
const positionLiquidity = ref(1000); // Configurable position liquidity
const daysCount = ref(30); // Number of days for average APR
const selectedDayOffset = ref(0);

const displayedDate = computed(() => {
  const date = new Date();
  date.setDate(date.getDate() + Number(selectedDayOffset.value));
  return date.toISOString().slice(0, 10);
});

const todaysTickData = computed(() => {
  if (!fullLPData.value) return [];

  return fullLPData.value.get(todaysDate).data[0].ticks
});

const todaysPriceData = computed(() => {
  if (!fullLPData.value) return null;

  return fullLPData.value.get(todaysDate).data[2]
});

const tickData = computed(() => {
  if (!fullLPData.value) return [];

  return fullLPData.value.get(displayedDate.value).data[0].ticks
});

const historyData = computed(() => {
  if (!fullLPData.value) return [];

  return fullLPData.value.get(displayedDate.value).data[1].dailyHistory
});

const priceData = computed(() => {
  if (!fullLPData.value) return null;

  return fullLPData.value.get(displayedDate.value).data[2]
});

// Computed properties
const groupedData = computed(() => processTicks(tickData.value));
const labels = computed(() => processTicks(todaysTickData.value).map(g => g.averagePrice.toFixed(1)));
const data = computed(() => processTicks(todaysTickData.value).map(g => Math.abs(g.totalLiquidity)));

const priceToTick = computed(() => createPriceToTickMap(tickData.value));

const activePriceToday = computed(() => {
  const tokenPrice = Math.max(todaysPriceData.value?.token0?.price || 0, 1 / (todaysPriceData.value?.token0?.price || 1));
  if (!tokenPrice || !labels.value.length) return null;

  const currentPriceTick = labels.value.reduce(
    (closestIdx, curr, idx) => (
      Math.abs(Number(curr) - tokenPrice) < Math.abs(Number(labels.value[closestIdx]) - tokenPrice) ? idx : closestIdx
    ),
    0
  );
  return labels.value[currentPriceTick];
});

const activePrice = computed(() => {
  if (!priceData.value?.token0?.price || !labels.value.length) return null;

  const mainTokenPrice = Math.max(priceData.value.token0.price, 1 / priceData.value.token0.price);

  const currentPriceTick = labels.value.reduce(
    (closestIdx, curr, idx) => (
      Math.abs(Number(curr) - mainTokenPrice) < Math.abs(Number(labels.value[closestIdx]) - mainTokenPrice) ? idx : closestIdx
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
  if (!activePriceToday.value || activePriceToday.value === '0.0' || !labels.value.length || streamedLowerline.value) return { lower: null, upper: null };
  const lowerBound = Number(activePriceToday.value) * 0.9;
  const upperBound = Number(activePriceToday.value) * 1.1;
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
const dailyData = computed(() => generateDailyData(tickData.value, historyData.value, priceData.value,));

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
    if ((lowerTickValue > priceToTick.value[activePrice.value!]!) || (upperTickValue < priceToTick.value[activePrice.value!]!)) {
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
      fullLPData.value,
      lowerTickValue,
      upperTickValue,
      positionLiquidity.value,
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

// Fetch liquidity data
async function fetchLiquidityData(poolId: string) {
  try {
   const { data, error } = await supabase
    .from('DeFiPools_snapshots')
    .select('*')
    .eq('poolAddress', poolId)
    // .order('date', { ascending: false });

    if (error) {
      throw new Error('Supabase error: ' + error.message);
    }

    const indexedByDate = indexBy(data || [], (item) => item.date);

    console.log('Supabase data:', indexedByDate);    

    return indexedByDate
  } catch (err) {
    throw new Error('Error fetching data: ' + (err as Error).message);
  }
}

// Render chart with D3.js
function renderChart() {
  if (!groupedData.value.length || !activePrice.value || !lowerBoundPrice.value || !upperBoundPrice.value) {
    console.error('Invalid data for rendering chart');
    return;
  }

  // Chart dimensions
  const margin = { top: 80, right: 150, bottom: 80, left: 150 };
  const width = 900 - margin.left - margin.right;
  const height = 400 - margin.top - margin.bottom;

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
    .domain([0, d3.max(data.value)! * 1.50])
    .range([height, 0]);

  // X-axis with max 5 ticks
  const maxTicks = 30;
  let tickLabels: string[] = [];
  if (labels.value.length <= maxTicks) {
    tickLabels = labels.value;
  } else {
    const step = (labels.value.length - 1) / (maxTicks - 1);
    tickLabels = Array.from({ length: maxTicks }, (_, i) => labels.value[Math.round(i * step)]);
  }

  // 350,000,000,000,000 divided by 10e9 in display....

  svg.append('g')
    .attr('transform', `translate(0,${height})`)
    .call(d3.axisBottom(x).tickValues(tickLabels))
    .selectAll('text')
    .attr('transform', 'rotate(-45)')
    .style('text-anchor', 'end');

  svg.append('g')
    .call(d3.axisLeft(y).tickFormat((d: number) => {
      const normalized = d / 1e9

      return normalized.toLocaleString('en-US', { style: 'currency', currency: 'USD', maximumFractionDigits: 0 })
    }));

  // X-axis label
  svg.append('text')
    .attr('x', width / 2)
    .attr('y', height + margin.bottom - 10)
    .style('text-anchor', 'middle')
    .style('fill', 'white')
    .text(`Price (${route.query.name?.split('(')[0]})`);

  // Y-axis label
  svg.append('text')
    .attr('transform', 'rotate(-90)')
    .attr('x', -height / 2)
    .attr('y', -margin.left + 20)
    .style('text-anchor', 'middle')
    .style('fill', 'white')
    .text('Liquidity (USD)');

  // Title
  svg.append('text')
    .attr('x', width / 2)
    .attr('y', -margin.top / 2)
    .style('text-anchor', 'middle')
    .style('font-size', '16px')
    .style('font-weight', 'bold')
    .style('fill', 'white')
    .text(`Liquidity Pool Distribution (Total TVL: ${todaysPriceData.value.tvl.toLocaleString('en-US', { style: 'currency', currency: 'USD', maximumFractionDigits: 0 })})`);

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
    .attr('fill', 'var(--color-primary)');

  // Helper function to convert price to x-coordinate
  const priceToX = (price: string) => {
    const xVal = x(price);
    return xVal !== undefined ? xVal + x.bandwidth() / 2 : 0;
  };

  // Current price line
  svg.append('line')
    .attr('class', 'current-price')
    .attr('x1', priceToX(activePrice.value!))
    .attr('x2', priceToX(activePrice.value!))
    .attr('y1', 0)
    .attr('y2', height)
    .attr('stroke', 'var(--color-accent)')
    .attr('stroke-width', 2);

  // Lower bound line
  const lowerLine = svg.append('line')
    .attr('class', 'lower-bound')
    .attr('x1', priceToX(lowerBoundPrice.value!))
    .attr('x2', priceToX(lowerBoundPrice.value!))
    .attr('y1', 0)
    .attr('y2', height)
    .attr('stroke', 'var(--color-border)')
    .attr('stroke-width', 6)
    .attr('stroke-dasharray', '10,1')
    .style('cursor', 'ew-resize');

  // Upper bound line
  const upperLine = svg.append('line')
    .attr('class', 'upper-bound')
    .attr('x1', priceToX(upperBoundPrice.value!))
    .attr('x2', priceToX(upperBoundPrice.value!))
    .attr('y1', 0)
    .attr('y2', height)
    .attr('stroke', 'var(--color-border)')
    .attr('stroke-width', 6)
    .attr('stroke-dasharray', '10,1')
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
    const response = await fetchLiquidityData(route.params.id as string);

    fullLPData.value = response;
  } catch (err) {
    console.error(err);
    error.value = (err as Error).message;
  } finally {
    loading.value = false;
  }
}

// Watch for changes in computed properties to re-render chart
watch([groupedData, activePrice, lowerBoundPrice, upperBoundPrice], () => {
  renderChart();
});

// Initial load
onMounted(loadData);
</script>