<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import ColorPicker from './ColorPicker.vue';
import Settings from './Settings.vue';

const appWindow = getCurrentWebviewWindow();
const isSettingsWindow = appWindow.label === 'settings';

onMounted(() => {
  if (isSettingsWindow) {
    document.body.classList.add('settings-mode');
  } else {
    document.body.classList.add('picker-mode');
  }
});
</script>

<template>
  <Settings v-if="isSettingsWindow" />
  <ColorPicker v-else />
</template>

<style>
/* Remove default body margin and set transparent background */
body, html, #app {
  margin: 0;
  padding: 0;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

body.picker-mode {
  background-color: transparent !important;
  cursor: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="white" stroke="black" stroke-width="1.5"><path d="M14.5 2.5l7 7-2 2-7-7 2-2z"/><path d="M12.5 4.5l7 7-10 10-4 1-1-4 10-10z"/></svg>') 5 22, crosshair !important;
}

body.settings-mode {
  background-color: #f5f5f5 !important;
  overflow-y: auto;
}
</style>
