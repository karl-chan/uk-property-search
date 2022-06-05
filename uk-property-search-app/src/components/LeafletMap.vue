<template lang='pug'>
div(:id='mapId')
</template>

<script lang="ts">
import * as L from 'leaflet'
import type { PropType } from 'vue'
import { defineComponent, onMounted, ref, toRefs, watch } from 'vue'

export default defineComponent({
  name: 'LeafletMap',
  props: {
    coordinates: {
      type: Array as unknown as PropType<[number, number]>,
      default: () => [51.50354, -0.127695]
    },
    zoom: {
      type: Number,
      default: 13
    },
    markers: {
      type: Array as PropType<L.Layer[]>,
      default: () => []
    }
  },
  setup (props) {
    const { coordinates, zoom, markers } = toRefs(props)
    const mapId = ref(generateMapId())
    let map : L.Map | undefined

    onMounted(() => {
      map = initialise({
        id: mapId.value,
        coordinates: coordinates.value,
        zoom: zoom.value,
        markers: markers.value
      })
    })

    watch(coordinates, () => {
      if (map !== undefined) {
        map.panTo(coordinates.value)
      }
    })
    watch(zoom, () => {
      if (map !== undefined) {
        map.setZoom(zoom.value)
      }
    })
    watch(markers, (newMarkers: L.Layer[], oldMarkers: L.Layer[]) => {
      if (map !== undefined) {
        const newSet = new Set(newMarkers)
        const oldSet = new Set(oldMarkers)

        const addMarkers = newMarkers.filter(m => !oldSet.has(m))
        const removeMarkers = oldMarkers.filter(m => !newSet.has(m))

        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        removeMarkers.forEach(marker => marker.removeFrom(map!))
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        addMarkers.forEach(marker => marker.addTo(map!))
      }
    })

    return {
      mapId
    }
  }
})

function initialise ({
  id,
  coordinates,
  zoom,
  markers
}: {
  id: string;
  coordinates: [number, number];
  zoom: number;
  markers: L.Layer[];
}): L.Map {
  const map = L.map(id, { preferCanvas: true })
    .setView(coordinates, zoom)
    .addLayer(
      L.tileLayer(
        'http://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png',
        { subdomains: ['a', 'b', 'c'] })
    )
  markers.forEach(marker => marker.addTo(map))
  return map
}

function generateMapId (): string {
  return `map-${Math.floor(Math.random() * 1000000)}`
}
</script>any
