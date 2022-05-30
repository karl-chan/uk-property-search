import { School } from '../models/school'
import api from './api'

class SchoolsApi {
  async fetchSchools (): Promise<School[]> {
    return api.get<School[]>('/schools')
  }
}

export default new SchoolsApi()
