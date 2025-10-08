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
      console.log('url', url)
      const finalUrl = useCorsProxy ? `https://corsproxy.io/?${encodeURIComponent(url)}` : url;
      const response = await fetch(finalUrl);
      // const response = await fetch(url);

      if (!response.ok) {
        throw new Error(`Failed to fetch data: ${response.status} ${response.statusText}`);
      }

      const data: ApiResponse[] = await response.json();
      return data.map((item, index) => {
        if (!item.result?.data?.json) {
          console.log('item', item)
          throw new Error(`Invalid response structure for call at index ${index}`);
        }
        return item.result.data.json;
      });
    } catch (err) {
      throw new Error(`Error fetching and flattening data: ${(err as Error).message}`);
    }
  }
}

const pools = [
    {
        address: '3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv',
        name: 'WSOL-USDC (Raydium)',
        network: 'solana',
        exchange: 'raydium',
        baseTokenAddress: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v' // usdc
    },
    {
        address: 'Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE',
        name: 'WSOL/USDC (Orca)',
        network: 'solana',
        exchange: 'orca',
        baseTokenAddress: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v' // usdc
    },
    {
        address: '6R4r93V5fcMzc13CL2enEepDSYcr4Qx3ptZBDwudTXCo',
        name: 'NVDAx-USDC (Orca)',
        network: 'solana',
        exchange: 'orca',
        baseTokenAddress: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'  // usdc
    },
]


async function fetchLiquidityData(exchange, network, poolId, baseTokenAddress) {
  try {
    const builder = new MetrixApiBatchUrlBuilder();
    const data = await builder.fetchAndFlatten([
      {
        method: 'exchanges.getPoolTicks',
        body: {
          exchange,
          network,
          poolAddress: poolId,
          token0Decimals: 9,
          token1Decimals: 6,
        },
      },
      {
        method: 'exchanges.getPoolHistory',
        body: {
          exchange,
          network,
          poolAddress: poolId,
          feeTier: '400',
          apiKey: 1,
          baseTokenAddress,
        },
      },
      {
        method: 'exchanges.getSimulatePool',
        body: {
          apiKey: 1,
          baseTokenAddress,
          exchange,
          network,
          poolAddress: poolId,
        },
      },
    ], true);

    console.log('data to store', data)

    return data
  } catch (err) {
    throw new Error('Error fetching data: ' + (err as Error).message);
  }
}


import { serve } from 'https://deno.land/std@0.201.0/http/server.ts';
import { createClient } from "https://cdn.jsdelivr.net/npm/@supabase/supabase-js/+esm";
const SUPABASE_URL = Deno.env.get('SUPABASE_URL');
const SUPABASE_SERVICE_ROLE_KEY = Deno.env.get('SUPABASE_SERVICE_ROLE_KEY');
const supabase = createClient(SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY);

async function checkBalances() {
  for (const pool of pools) {
    const data = await fetchLiquidityData(pool.exchange, pool.network, pool.address, pool.baseTokenAddress);
    console.log('Fetched data for pool:', pool.address, data);

    const { error } = await supabase.from('DeFiPools_snapshots').insert({
      date: new Date().toISOString().split('T')[0],
      poolAddress: pool.address,
      data,
    });

    console.log('Error?:', error)
  }
}


serve(async (req)=>{
  if (req.method === 'OPTIONS') {
    return new Response('ok', {
      headers: {
        'Access-Control-Allow-Origin': '*'
      }
    });
  }
  try {
    await checkBalances();
    return new Response('Balances checked and alerts sent if necessary.', {
      status: 200
    });
  } catch (error) {
    console.error('Error in Edge Function:', error);
    return new Response('Internal Server Error', {
      status: 500
    });
  }
});
