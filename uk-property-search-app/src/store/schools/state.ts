import { School } from '../../models/school'

export interface SchoolsStateInterface {
  schools: School[];
}

function state (): SchoolsStateInterface {
  return {
    schools: []
  }
}

export default state
