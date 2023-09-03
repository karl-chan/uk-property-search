<template lang='pug'>
property-stats-layout(
  title='Property sizes'
  :stats-getter='statsGetter'
  :slider-options='sliderOptions'
  :format-options='formatOptions'
)
</template>

<script lang="ts">
import PropertyStatsLayout, { FormatOptions, SliderOptions } from 'layouts/PropertyStatsLayout.vue'
import { round } from 'lodash'
import { defineComponent } from 'vue'
import { PropertyAction, PropertySummary, Stats } from '../../models/property'

export default defineComponent({
  name: 'PropertySizesPage',
  components: {
    PropertyStatsLayout
  },
  setup () {
    const sliderOption = {
      min: 0,
      max: 1000,
      step: 100,
      multiplier: 1,
      formatter (value: number) {
        return `${value} sq. ft.`
      }
    }
    function formatShortValue (value: number): string {
      return `${round(value)} sf`
    }
    function formatValue (value: number): string {
      return `${round(value)} sq. ft.`
    }
    const formatOption = {
      formatShortValue,
      formatValue,
      markerWidth: 60
    }

    function statsGetter (property:PropertySummary): Stats {
      return property.stats.squareFeet
    }
    const sliderOptions: SliderOptions = {
      [PropertyAction.Buy]: sliderOption,
      [PropertyAction.Rent]: sliderOption
    }
    const formatOptions: FormatOptions = {
      [PropertyAction.Buy]: formatOption,
      [PropertyAction.Rent]: formatOption
    }

    return {
      statsGetter,
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      sliderOptions,
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      formatOptions
    }
  }
})
</script>
