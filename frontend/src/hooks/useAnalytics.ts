import { useEffect, useMemo, useState } from 'react';

export interface AnalyticsTimeseriesPoint {
  date: string;
  impressions: number;
  clicks: number;
}

interface UseAnalyticsTimeseriesOptions {
  campaignIds: string[];
  timeframe: '7d' | '30d';
}

export function useAnalyticsTimeseries({ campaignIds, timeframe }: UseAnalyticsTimeseriesOptions) {
  const [data, setData] = useState<AnalyticsTimeseriesPoint[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Memoize campaignIds to prevent unnecessary re-fetches when the array reference changes
  // but the content remains the same.
  const campaignIdsKey = useMemo(() => campaignIds.join(','), [campaignIds]);

  useEffect(() => {
    const controller = new AbortController();

    setLoading(true);
    setError(null);

    // Replace with actual API endpoint or contract call
    fetch(`/api/analytics/timeseries?campaignIds=${campaignIdsKey}&timeframe=${timeframe}`, {
      signal: controller.signal,
    })
      .then(res => {
        if (!res.ok) throw new Error('Failed to fetch analytics timeseries');
        return res.json();
      })
      .then((result: AnalyticsTimeseriesPoint[]) => {
        setData(result);
        setLoading(false);
      })
      .catch(err => {
        if (err.name === 'AbortError') {
          return;
        }

        setError(err.message);
        setLoading(false);
      });

    return () => {
      controller.abort();
    };
  }, [campaignIdsKey, timeframe]);

  return { data, loading, error };
}
