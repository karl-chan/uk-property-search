<template lang='pug'>
property-stats-layout(
  title='Property prices'
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
  name: 'PropertyPricesPage',
  components: {
    PropertyStatsLayout
  },
  setup () {
    function formatShortValue (value: number): string {
      if (value < 10000) {
        return `£${round(value)}`
      } else if (value < 1_000_000) {
        return `£${round(value / 1000)}k`
      } else {
        return `£${(value / 1_000_000).toFixed(1)}M`
      }
    }
    function formatValue (value: number): string {
      return `£${round(value)}`
    }
    const formatOption = {
      formatShortValue,
      formatValue,
      markerWidth: 60
    }

    function statsGetter (property:PropertySummary): Stats {
      return property.stats.price
    }
    const sliderOptions: SliderOptions = {
      [PropertyAction.Buy]: {
        min: 0,
        max: 2_000_000,
        step: 50_000,
        multiplier: 1,
        formatter (value: number) {
          return `£${value}`
        }
      },
      [PropertyAction.Rent]: {
        min: 0,
        max: 4_000,
        step: 100,
        multiplier: 1,
        formatter: (value: number) => `£${value}`
      }
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
