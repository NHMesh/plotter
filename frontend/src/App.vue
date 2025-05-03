<template>
  <div style="height:600px; width:800px">
    <l-map ref="map" v-model:zoom="zoom" :center="[43.197167282501276, -71.455078125]"  @click="addMarker">
      <l-tile-layer
        url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        layer-type="base"
        name="OpenStreetMap"
      ></l-tile-layer>

      <l-marker v-for="marker, index in markers" :lat-lng="marker" @click="removeMarker(index)"></l-marker>

      <l-tile-layer
        url="http://localhost:3000/tiles/{z}/{x}/{y}/tile.png"
        layer-type="base"
        opacity="0.5"
        name="Elevation"
      ></l-tile-layer>
    </l-map>
  </div>
  <button @click="selection" >Select Area</button>
  <button @click="los" >LOS?</button>
  <button @click="scan" >GeoScan</button>
  {{polyPoints}}
  <br>
  {{markers}}
</template>

<script>
import "leaflet/dist/leaflet.css";
import { LMap, LTileLayer, LMarker } from "@vue-leaflet/vue-leaflet";
import '@bopen/leaflet-area-selection/dist/index.css';
import { DrawAreaSelection } from '@bopen/leaflet-area-selection';
import axios from 'axios';

export default {
  mounted() {
    this.map = this.$refs.map;

    setTimeout(() => {
      this.areaSelection = new DrawAreaSelection({
        active: false,
        onPolygonReady: (a,b,c) => {
          this.polyPoints = a.getLatLngs()[0];
        }});
      this.map.leafletObject.addControl(this.areaSelection);
    }, 1000)
  },
  components: {
    LMap,
    LTileLayer,
    LMarker,
  },
  data() {
    return {
      zoom: 13,
      polyPoints: [],
      markers: [ ]
    };
  },

  methods: {
    selection() {
      this.areaSelection.activate();
    },

    removeMarker(index) {
      this.markers.splice(index, 1);
    },

    addMarker(event) {
      console.log('add marker', event.latlng)
      this.markers.push(event.latlng);
    },

    async los() {
      let res = await axios.post('http://localhost:3000/los', {
        from_lat: this.markers[0].lat,
        from_lon: this.markers[0].lng,
        to_lat: this.markers[1].lat,
        to_lon: this.markers[1].lng,
      });

      console.log(res);
    },

    async scan() {
      let res = await axios.post('http://localhost:3000/scan', {
        from_lat: this.markers[0].lat,
        from_lon: this.markers[0].lng,
        polygon: this.polyPoints.map((p) => [p.lat, p.lng])
      });

      console.log(res);
    }
  }
};
</script>

<style>

</style>
