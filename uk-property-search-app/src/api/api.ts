import axios, { AxiosResponse } from 'axios'

class Api {
  private readonly http = axios.create({ baseURL: '/api' })

  async get<T> (url: string) : Promise<T> {
    const { data } = await this.http.get<T, AxiosResponse<T>>(url)
    return data
  }
}

export default new Api()
