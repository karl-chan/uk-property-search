
import { LastUpdated } from '../models/last-updated'
import api from './api'

class LastUpdatedApi {
  async fetchLastUpdated (): Promise<LastUpdated> {
    return api.get<LastUpdated>('/last-updated')
  }
}

export default new LastUpdatedApi()
