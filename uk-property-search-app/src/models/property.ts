
export interface Stats {
  min: number,
  q1: number,
  median: number,
  q3: number,
  max: number,
}

export interface PropertyStats {
  price: Stats,
  post_date: Stats,
  count: number,
}

export interface PropertySummary {
  postcode: string,
  coordinates: [number, number],
  stats: PropertyStats,
}
