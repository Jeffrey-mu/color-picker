<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const color = ref('#FFFFFF');
const locked = ref(false);
const showCopied = ref(false);

interface ColorEvent {
  hex: string;
  grid: string[];
}

const radius = ref(5); // default 11x11 grid
const colorGrid = ref<string[]>([]);

const isPickingUi = ref(false);
const mouseX = ref(-1000);
const mouseY = ref(-1000);

const rgbColor = computed(() => {
  const hex = color.value.replace('#', '');
  if (hex.length !== 6) return 'RGB(255, 255, 255)';
  const r = parseInt(hex.substring(0, 2), 16);
  const g = parseInt(hex.substring(2, 4), 16);
  const b = parseInt(hex.substring(4, 6), 16);
  return `RGB(${r}, ${g}, ${b})`;
});

// Compute optimal position for the magnifier based on mouse position relative to screen edges
// We use window.innerWidth and window.innerHeight because the Vue app spans the full screen.
const magnifierPos = computed(() => {
  const boxWidth = 160; // Approximate width of the magnifier card
  const boxHeight = 200; // Approximate height of the magnifier card
  const offset = 20; // Distance from cursor
  
  // By default, put it bottom-right
  let left = `calc(50% + ${offset}px)`;
  let top = `calc(50% + ${offset}px)`;
  
  // We use mouseX and mouseY (which represent global screen coordinates)
  // to determine which quadrant of the screen we are in.
  // Wait until we have valid mouse coordinates
  if (mouseX.value !== -1000 && mouseY.value !== -1000) {
    const screenWidth = window.screen.width || window.innerWidth;
    const screenHeight = window.screen.height || window.innerHeight;
    
    // If too close to the right edge, flip to left
    if (mouseX.value + offset + boxWidth > screenWidth) {
      left = `calc(50% - ${offset + boxWidth}px)`;
    }
    
    // If too close to the bottom edge, flip to top
    if (mouseY.value + offset + boxHeight > screenHeight) {
      top = `calc(50% - ${offset + boxHeight}px)`;
    }
  }
  
  return { left, top };
});

const fetchColorData = async () => {
  if (!isPickingUi.value || isUpdating || targetX === -1 || targetY === -1) return;
  isUpdating = true;
  try {
    const colorData = await invoke<ColorEvent | null>('get_current_color', { 
      radius: radius.value,
      x: Math.round(targetX),
      y: Math.round(targetY)
    });
    if (colorData) {
      color.value = colorData.hex;
      colorGrid.value = colorData.grid;
    }
  } catch (e: any) {
    console.error('Update error', e);
  }
  isUpdating = false;
};

const handleWheel = async (e: WheelEvent) => {
  if (locked.value) return;
  const oldRadius = radius.value;
  if (e.deltaY < 0) {
    // Zoom in (fewer blocks -> decrease radius)
    radius.value = Math.max(radius.value - 1, 1);
  } else {
    // Zoom out (more blocks -> increase radius)
    radius.value = Math.min(radius.value + 1, 15);
  }
  if (oldRadius !== radius.value) {
    await fetchColorData();
  }
};

const handleKeyDown = async (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    // 立即清除所有 UI 状态
    isPickingUi.value = false;
    locked.value = false;
    showCopied.value = false;
    
    // 明确调用 stop_picking 而不是 toggle_window，防止状态翻转导致闪烁
    await invoke('stop_picking');
  }
};

let targetX = -1;
let targetY = -1;
let unlistenMoveFn: (() => void) | null = null;
let unlistenCopiedFn: (() => void) | null = null;
let unlistenExitFn: (() => void) | null = null;
let unlistenStartFn: (() => void) | null = null;
let isUpdating = false;

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  window.addEventListener('wheel', handleWheel);

  unlistenMoveFn = await listen<{x: number, y: number}>('mouse-move', async (event) => {
    if (locked.value) return;
    
    // Auto-recover isPickingUi if start-picking was missed
    if (!isPickingUi.value) {
      isPickingUi.value = true;
    }

    targetX = event.payload.x;
    targetY = event.payload.y;
    mouseX.value = targetX;
    mouseY.value = targetY;
    
    // Fetch color directly on move
    await fetchColorData();
  });

  unlistenStartFn = await listen<{x: number, y: number}>('start-picking', async (event) => {
    isPickingUi.value = true;
    locked.value = false;
    showCopied.value = false;
    if (event.payload.x !== -1) {
      targetX = event.payload.x;
      targetY = event.payload.y;
      mouseX.value = targetX;
      mouseY.value = targetY;
    }
  });

  unlistenExitFn = await listen('exit-picking', async () => {
    isPickingUi.value = false;
    locked.value = false;
    showCopied.value = false;
  });

  unlistenCopiedFn = await listen<string>('color-copied', async (event) => {
    color.value = event.payload;
    showCopied.value = true;
    locked.value = true;
    isPickingUi.value = false;
    
    // 延迟隐藏窗口，以便用户看到 Toast 提示
    setTimeout(() => {
      showCopied.value = false;
      
      // 等待 Toast 消失动画 (150ms) 结束后再隐藏物理窗口
      setTimeout(async () => {
        locked.value = false;
        await invoke('stop_picking');
      }, 200);
    }, 1200);
  });
});

onUnmounted(() => {
  if (unlistenMoveFn) unlistenMoveFn();
  if (unlistenCopiedFn) unlistenCopiedFn();
  if (unlistenExitFn) unlistenExitFn();
  if (unlistenStartFn) unlistenStartFn();
  window.removeEventListener('keydown', handleKeyDown);
  window.removeEventListener('wheel', handleWheel);
});
</script>

<template>
  <div class="relative w-[100vw] h-[100vh] overflow-hidden">
    
    <!-- Magnifier UI (Offset from center to simulate ColorZilla) -->
    <div v-show="isPickingUi" 
         class="absolute pointer-events-none flex flex-col z-50 transition-all duration-300 ease-out"
         :style="magnifierPos">
      
      <!-- Modern Magnifier Card -->
      <div class="bg-white/95 backdrop-blur-md border border-gray-200 shadow-2xl rounded-xl p-2 flex flex-col gap-2"
           :class="{'ring-2 ring-blue-500 ring-offset-1': locked}">
        
        <!-- Magnifier Grid -->
        <div class="relative rounded-lg overflow-hidden border border-gray-200/80 shadow-inner bg-gray-50"
             :style="{
               width: `140px`,
               height: `140px`
             }">
          
          <div class="absolute inset-0 grid"
               :style="{
                 gridTemplateColumns: `repeat(${radius * 2 + 1}, 1fr)`,
                 gridTemplateRows: `repeat(${radius * 2 + 1}, 1fr)`
               }">
            <div v-for="(hex, index) in colorGrid" :key="index"
                 class="box-border"
                 :style="{
                   backgroundColor: hex,
                   borderRight: radius <= 12 ? '1px solid rgba(136, 136, 136, 0.15)' : 'none',
                   borderBottom: radius <= 12 ? '1px solid rgba(136, 136, 136, 0.15)' : 'none'
                 }">
            </div>
          </div>

          <!-- Center Square Crosshair -->
          <div class="absolute flex items-center justify-center pointer-events-none"
               :style="{
                 left: '50%',
                 top: '50%',
                 transform: 'translate(-50%, -50%)',
                 width: `${100 / (radius * 2 + 1)}%`,
                 height: `${100 / (radius * 2 + 1)}%`,
               }">
               <div class="w-full h-full border border-white shadow-[0_0_0_1px_rgba(0,0,0,0.6)] box-border"></div>
          </div>
        </div>
        
        <!-- Info Panel -->
        <div class="flex items-center gap-3 px-1 pb-0.5">
          <!-- Color Preview Circle -->
          <div class="w-7 h-7 rounded-full shadow-[inset_0_2px_4px_rgba(0,0,0,0.1)] border border-gray-300 flex-shrink-0" 
               :style="{ backgroundColor: color }"></div>
          
          <!-- Text Info -->
          <div class="flex flex-col justify-center">
            <span class="font-mono text-[14px] font-bold text-gray-800 leading-none mb-1 tracking-wide">{{ color }}</span>
            <span class="font-mono text-[10px] text-gray-500 leading-none">{{ rgbColor }}</span>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Copied toast -->
    <transition enter-active-class="transition duration-200 ease-out"
                enter-from-class="opacity-0 scale-90"
                enter-to-class="opacity-100 scale-100"
                leave-active-class="transition duration-150 ease-in"
                leave-from-class="opacity-100 scale-100"
                leave-to-class="opacity-0 scale-90">
      <div v-if="showCopied" class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 bg-[#2C2C2E]/95 backdrop-blur-sm text-white px-5 py-3 rounded-2xl shadow-[0_8px_30px_rgb(0,0,0,0.24)] border border-white/10 flex items-center gap-3 z-50 pointer-events-none">
        <div class="flex items-center justify-center w-7 h-7 rounded-full bg-green-500/20">
          <svg class="w-4 h-4 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7"></path>
          </svg>
        </div>
        <span class="text-[15px] font-medium tracking-wide">颜色已复制</span>
        <div class="w-[1px] h-4 bg-white/20 mx-0.5"></div>
        <div class="flex items-center gap-2">
          <div class="w-4 h-4 rounded-sm shadow-[inset_0_0_0_1px_rgba(255,255,255,0.2)]" :style="{ backgroundColor: color }"></div>
          <span class="font-mono text-[14px] text-gray-200 pt-[1px]">{{ color }}</span>
        </div>
      </div>
    </transition>
  </div>
</template>
