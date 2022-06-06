import { TubeStation } from '../../models/tube'

export interface TubeStateInterface {
  stations: TubeStation[]
}

function state (): TubeStateInterface {
  return {
    stations: []
  }
}

export default state
