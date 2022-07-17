import { TubeStation } from '../models/tube'
import api from './api'

class TubeApi {
  async fetchStations (): Promise<TubeStation[]> {
    return api.get<TubeStation[]>('/tube-stations')
  }
}

export default new TubeApi()
