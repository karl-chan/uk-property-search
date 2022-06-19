<template lang='pug'>
property-stats-layout(
  title='Listings age'
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
  name: 'PropertyTurnoverPage',
  components: {
    PropertyStatsLayout
  },
  setup () {
    function statsGetter (property:PropertySummary): Stats {
      return property.stats.listedDays
    }
    const sliderOptions: SliderOptions = {
      [PropertyAction.Buy]: {
        min: 0,
        max: 24,
        step: 1,
        multiplier: 30,
        formatter (value: number): string {
          return `${value} months`
        }
      },
      [PropertyAction.Rent]: {
        min: 0,
        max: 24,
        step: 1,
        multiplier: 7,
        formatter (value: number): string {
          return `${value} weeks`
        }
      }
    }
    const formatOptions: FormatOptions = {
      [PropertyAction.Buy]: {
        formatShortValue (value: number): string {
          return `${(value / 30).toFixed(1)} mths`
        },
        formatValue (value: number): string {
          return `${(value / 30).toFixed(1)} months`
        },
        markerWidth: 70
      },
      [PropertyAction.Rent]: {
        formatShortValue (value: number): string {
          return `${(value / 7).toFixed(1)} wks`
        },
        formatValue (value: number): string {
          return `${(value / 7).toFixed(1)} weeks`
        },
        markerWidth: 70
      }
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
