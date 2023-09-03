<template lang='pug'>
property-stats-layout(
  title='Rental Yield'
  :stats-getter='statsGetter'
  :slider-options='sliderOptions'
  :format-options='formatOptions'
)
</template>

<script lang="ts">
import PropertyStatsLayout, { FormatOptions, SliderOptions } from 'layouts/PropertyStatsLayout.vue'
import { defineComponent } from 'vue'
import { PropertyAction, PropertySummary, Stats } from '../../models/property'

export default defineComponent({
  name: 'PropertyYieldPage',
  components: {
    PropertyStatsLayout
  },
  setup () {
    const sliderOption = {
      min: 0,
      max: 100,
      step: 1,
      multiplier: 1 / 100,
      formatter (value: number): string {
        return `${value}%`
      }
    }
    function formatShortValue (value: number): string {
      return `${(value * 100).toFixed(1)}%`
    }
    function formatValue (value: number): string {
      return `${(value * 100).toFixed(1)}%`
    }
    const formatOption = {
      formatShortValue,
      formatValue,
      markerWidth: 50
    }

    function statsGetter (property:PropertySummary): Stats {
      return property.stats.rentalYield
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
