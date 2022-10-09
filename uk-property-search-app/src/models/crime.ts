export interface CrimeStreet {
  id: number,
  name: string
}

export interface CrimeLocation {
  latitude: string,
  longitude: string,
  street: CrimeStreet,
}

export interface CrimeOutcomeStatus {
  category: string,
  date: string, // YYYY-MM
}

export interface Crime {
  category: string,
  context: string,
  id: number,
  location: CrimeLocation
  location_subtype: string,
  location_type: string,
  month: string, // YYYY-MM
  outcome_status?: CrimeOutcomeStatus
  persistent_id: string
}
