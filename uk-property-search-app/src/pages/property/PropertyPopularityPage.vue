<template lang='pug'>
property-stats-layout(
  title='Popularity'
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
  name: 'PropertyPopularityPage',
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
        return `${value} %`
      }
    }
    function formatShortValue (value: number): string {
      return `${round(value * 100)} %`
    }
    function formatValue (value: number): string {
      return `${round(value * 100)} %`
    }
    const formatOption = {
      formatShortValue,
      formatValue,
      markerWidth: 60
    }

    function statsGetter (property:PropertySummary): Stats {
      return property.stats.percentTransacted
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
