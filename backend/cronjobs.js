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

const pools = [
    {
        address: '3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv',
        network: 'solana',
        exchange: 'jupiter',
        baseTokenAddress: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v' // sol or usdc?
    },
    {
        address: '0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8',
        network: 'solana',
        exchange: 'orca',
        baseTokenAddress: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v' // sol or usdc?
    },
    {
        address: '6R4r93V5fcMzc13CL2enEepDSYcr4Qx3ptZBDwudTXCo', // nvda/usdc
        network: 'solana',
        exchange: 'orca',
        baseTokenAddress: '0x000'
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
          exchange: 'orca',
          network: 'solana',
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
          exchange: 'orca',
          network: 'solana',
          poolAddress: poolId,
        },
      },
    ], true);

    console.log('daata to store', data)
  } catch (err) {
    throw new Error('Error fetching data: ' + (err as Error).message);
  }
}