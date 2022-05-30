export enum Rating {
  Outstanding = 1,
  Good = 2,
  RequiresImprovement = 3,
  Inadequate = 4
}

export interface School {
  id: number;
  name: string;
  postcode?: string;
  coordinates?: [number, number];
  rating?: Rating;
  inspectionDate?: number; // unix milliseconds
}
