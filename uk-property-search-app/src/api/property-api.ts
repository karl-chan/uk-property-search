
import { PropertySummary } from '../models/property'
import api from './api'

class PropertyApi {
  async fetchProperties (): Promise<PropertySummary[]> {
    return api.get<PropertySummary[]>('/property')
  }
}

export default new PropertyApi()
