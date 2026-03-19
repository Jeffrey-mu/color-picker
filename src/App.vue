<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import ColorPicker from './ColorPicker.vue';
import Settings from './Settings.vue';

const appWindow = getCurrentWebviewWindow();
const isSettingsWindow = appWindow.label === 'settings';
const isPickingMode = ref(false);

onMounted(async () => {
  if (isSettingsWindow) {
    document.body.classList.add('settings-mode');
  } else {
    // Listen for start/exit events to toggle the cursor
    await listen('start-picking', () => {
      isPickingMode.value = true;
      document.body.classList.add('picker-mode');
    });
    
    await listen('exit-picking', () => {
      isPickingMode.value = false;
      document.body.classList.remove('picker-mode');
    });
    
    // Initially add picker-mode if we're not settings
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
  cursor: url('/app-icon-32.png') 0 32, crosshair !important;
}

body.settings-mode {
  background-color: #f5f5f5 !important;
  overflow-y: auto;
}
</style>
