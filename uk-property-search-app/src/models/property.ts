
export interface Stats {
  min: number,
  q1: number,
  median: number,
  q3: number,
  max: number,
  count: number
}

export interface PropertyStats {
  price: Stats,
  post_date: Stats,
}

export interface PropertySummary {
  postcode: string,
  coordinates: [number, number],
  stats: PropertyStats,
}
