
export enum PropertyAction {
  Buy = 1,
  Rent = 2
}

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
  days: Stats,
}

export interface PropertySummary {
  postcode: string,
  coordinates: [number, number],
  action: PropertyAction,
  numBeds: number,
  stats: PropertyStats,
}
