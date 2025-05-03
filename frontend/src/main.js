import './assets/main.css'

import { createApp } from 'vue'
import App from './App.vue'

import L from 'leaflet';

globalThis.L = L;
createApp(App).mount('#app')
