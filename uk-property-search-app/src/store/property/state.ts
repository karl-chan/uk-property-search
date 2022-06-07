import { PropertySummary } from '../../models/property'

export interface PropertyStateInterface {
  properties: PropertySummary[]
}

function state (): PropertyStateInterface {
  return {
    properties: []
  }
}

export default state
