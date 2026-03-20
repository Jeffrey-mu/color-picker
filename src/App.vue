<script setup lang="ts">
import { onMounted } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { listen } from '@tauri-apps/api/event';
import ColorPicker from './ColorPicker.vue';
import Settings from './Settings.vue';

const appWindow = getCurrentWebviewWindow();
const isSettingsWindow = appWindow?.label === 'settings';

console.log('App window label:', appWindow?.label);

onMounted(async () => {
  // 设置 html 和 #app 的基础全屏无边距样式
  document.documentElement.className = "m-0 p-0 w-screen h-screen overflow-hidden";
  const appEl = document.getElementById('app');
  if (appEl) {
    appEl.className = "m-0 p-0 w-screen h-screen overflow-hidden";
  }

  if (isSettingsWindow) {
    // 设置页面样式
    document.body.className = "m-0 p-0 w-screen h-screen overflow-y-auto bg-[#f5f5f5]";
  } else {
    // 取色器页面样式
    const pickerActiveClasses = "m-0 p-0 w-screen h-screen overflow-hidden bg-transparent cursor-[url('/app-icon-32.png')_0_32,crosshair]";
    const pickerInactiveClasses = "m-0 p-0 w-screen h-screen overflow-hidden bg-transparent";

    await listen('start-picking', () => {
      document.body.className = pickerActiveClasses;
    });
    
    await listen('exit-picking', () => {
      document.body.className = pickerInactiveClasses;
    });

    await listen('color-copied', () => {
      document.body.className = pickerInactiveClasses;
    });
    
    // 初始化为取色模式
    document.body.className = pickerActiveClasses;
  }
});
</script>

<template>
  <Settings v-if="isSettingsWindow" />
  <ColorPicker v-else />
</template>
