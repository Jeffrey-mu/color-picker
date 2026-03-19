<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

const currentShortcut = ref('');
const originalShortcut = ref('');
const recording = ref(false);
const keys = ref<Set<string>>(new Set());

// 任务队列，防止注销和注册快捷键发生竞态条件
let shortcutTask: Promise<void> = Promise.resolve();

const autoCopy = ref(true);
const copyFormat = ref('#RRGGBB');
const lowercaseHex = ref(false);

const formatOptions = [
  '#RRGGBB',
  'RRGGBB',
  'rgb(R, G, B)',
  'rgb(R%, G%, B%)',
  'hsl(H, S%, L%)'
];

const saveSettings = async () => {
  try {
    await invoke('set_app_settings', {
      settings: {
        auto_copy: autoCopy.value,
        copy_format: copyFormat.value,
        lowercase_hex: lowercaseHex.value
      }
    });
  } catch (e) {
    console.error('Failed to save settings:', e);
  }
};

watch([autoCopy, copyFormat, lowercaseHex], () => {
  saveSettings();
});

onMounted(async () => {
  try {
    currentShortcut.value = await invoke<string>('get_shortcut');
    originalShortcut.value = currentShortcut.value;
    
    const settings = await invoke<any>('get_app_settings');
    autoCopy.value = settings.auto_copy;
    copyFormat.value = settings.copy_format;
    lowercaseHex.value = settings.lowercase_hex;
  } catch (e) {
    console.error(e);
  }
});

const startRecording = () => {
  if (!currentShortcut.value.includes('请按下') && !currentShortcut.value.includes('无效') && !currentShortcut.value.includes('失败')) {
    originalShortcut.value = currentShortcut.value.replace(' (已保存)', '');
  }
  recording.value = true;
  keys.value.clear();
  currentShortcut.value = '请按下快捷键...';
  
  // 加入队列，确保按顺序执行
  shortcutTask = shortcutTask.then(() => invoke<void>('unregister_shortcut')).catch(e => {
    console.error('Failed to unregister shortcut:', e);
  });
};

const cancelRecording = () => {
  recording.value = false;
  keys.value.clear();
  currentShortcut.value = originalShortcut.value;
  
  // 恢复原始快捷键，加入队列
  let formatted = originalShortcut.value;
  formatted = formatted.replace('Command', 'Super');
  shortcutTask = shortcutTask.then(() => invoke<void>('set_shortcut', { newShortcut: formatted })).catch(e => {
    console.error('Failed to restore shortcut:', e);
  });
};

const handleKeyDown = (e: KeyboardEvent) => {
  if (!recording.value) {
    // 如果没有在录制快捷键，按下 Esc 直接关闭设置窗口
    if (e.key === 'Escape') {
      const window = getCurrentWebviewWindow();
      if (window) window.hide();
    }
    return;
  }
  
  e.preventDefault();
  
  let key = e.key;
  
  // Esc 取消录制
  if (key === 'Escape') {
    cancelRecording();
    return;
  }
  
  if (key === ' ') key = 'Space';
  
  if (key === 'Shift' || key === 'Control' || key === 'Alt' || key === 'Meta') {
    keys.value.add(key);
    updateDisplay();
  } else {
    // Capitalize single letters, leave others as is (e.g. Space, Enter, F1)
    if (key.length === 1) {
      keys.value.add(key.toUpperCase());
    } else {
      keys.value.add(key);
    }
    updateDisplay();
    finishRecording();
  }
};

const handleKeyUp = (e: KeyboardEvent) => {
  if (!recording.value) return;
  e.preventDefault();
  
  const key = e.key;
  if (key === 'Shift' || key === 'Control' || key === 'Alt' || key === 'Meta') {
    keys.value.delete(key);
    updateDisplay();
  }
};

const updateDisplay = () => {
  if (keys.value.size === 0) {
    currentShortcut.value = '请按下快捷键...';
    return;
  }
  
  const parts = [];
  if (keys.value.has('Control')) parts.push('Ctrl');
  if (keys.value.has('Alt')) parts.push('Alt');
  if (keys.value.has('Shift')) parts.push('Shift');
  if (keys.value.has('Meta')) parts.push('Command');
  
  for (const k of keys.value) {
    if (!['Control', 'Alt', 'Shift', 'Meta'].includes(k)) {
      parts.push(k);
    }
  }
  
  currentShortcut.value = parts.join('+');
};

const finishRecording = () => {
  recording.value = false;
  
  // 如果只按下了修饰键，视为无效，恢复原状
  if (Array.from(keys.value).every(k => ['Control', 'Alt', 'Shift', 'Meta'].includes(k))) {
    currentShortcut.value = '无效快捷键，请重新设置';
    
    let originalFormatted = originalShortcut.value.replace('Command', 'Super');
    shortcutTask = shortcutTask.then(() => invoke<void>('set_shortcut', { newShortcut: originalFormatted })).catch(console.error);
    
    setTimeout(() => {
      if (currentShortcut.value === '无效快捷键，请重新设置') {
        currentShortcut.value = originalShortcut.value;
      }
    }, 2000);
    return;
  }
  
  let formatted = currentShortcut.value;
  // Tauri 快捷键中 Command 映射为 Super
  formatted = formatted.replace('Command', 'Super');
  
  shortcutTask = shortcutTask.then(() => invoke<void>('set_shortcut', { newShortcut: formatted }))
    .then(() => {
      console.log('Shortcut saved');
      originalShortcut.value = currentShortcut.value;
      currentShortcut.value = currentShortcut.value + ' (已保存)';
      setTimeout(() => {
        if (currentShortcut.value.includes('(已保存)')) {
          currentShortcut.value = currentShortcut.value.replace(' (已保存)', '');
        }
      }, 2000);
    })
    .catch((err) => {
      console.error('Failed to save shortcut:', err);
      currentShortcut.value = '保存失败: ' + err;
      
      let originalFormatted = originalShortcut.value.replace('Command', 'Super');
      shortcutTask = shortcutTask.then(() => invoke<void>('set_shortcut', { newShortcut: originalFormatted })).catch(console.error);
      
      setTimeout(() => {
        if (currentShortcut.value.includes('保存失败')) {
          currentShortcut.value = originalShortcut.value;
        }
      }, 2000);
    });
};

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  window.addEventListener('keyup', handleKeyUp);
});
</script>

<template>
  <div class="p-6 text-gray-900 font-sans min-h-screen box-border bg-[#f8fafc] flex flex-col">
    <!-- Header -->
    <div class="mb-8">
      <h2 class="m-0 text-2xl font-bold tracking-tight text-gray-900">设置</h2>
    </div>
    
    <!-- Shortcut Section -->
    <div class="bg-white rounded-xl p-5 shadow-sm border border-gray-100 mb-6 transition-all hover:shadow-md">
      <div class="text-xs font-bold text-gray-400 mb-5 tracking-wider uppercase">系统快捷键</div>
      <div class="flex justify-between items-center">
        <div class="flex-1 pr-4">
          <div class="text-base font-semibold text-gray-800 mb-1">唤醒 / 隐藏取色器</div>
          <div class="text-sm text-gray-500">全局快捷键，用于快速打开或关闭屏幕取色器</div>
        </div>
        <button 
          class="bg-gray-50 border border-gray-200 py-2.5 px-5 rounded-lg text-sm font-mono cursor-pointer transition-all duration-200 min-w-[140px] text-center hover:bg-gray-100 hover:border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500/50" 
          :class="{ 'bg-blue-50 border-blue-400 text-blue-700 ring-4 ring-blue-500/20 shadow-inner': recording }"
          @click="startRecording"
        >
          <span v-if="recording" class="inline-block animate-pulse mr-2 h-2 w-2 rounded-full bg-blue-600"></span>
          {{ currentShortcut }}
        </button>
      </div>
    </div>
    
    <!-- Color Copy Section -->
    <div class="bg-white rounded-xl p-5 shadow-sm border border-gray-100 mb-6 transition-all hover:shadow-md">
      <div class="text-xs font-bold text-gray-400 mb-5 tracking-wider uppercase">颜色复制</div>
      
      <div class="flex flex-col gap-6">
        <!-- 自动复制 Toggle -->
        <div class="flex items-center justify-between">
          <div class="flex-1 pr-4">
            <div class="text-base font-semibold text-gray-800 mb-1">自动复制</div>
            <div class="text-sm text-gray-500">拾取颜色后自动复制到系统剪贴板</div>
          </div>
          <button 
            @click="autoCopy = !autoCopy"
            :class="autoCopy ? 'bg-blue-600' : 'bg-gray-200'"
            class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-600 focus:ring-offset-2"
          >
            <span :class="autoCopy ? 'translate-x-5' : 'translate-x-0'" class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"></span>
          </button>
        </div>
        
        <!-- 复制格式 Select -->
        <div class="flex items-center justify-between transition-opacity duration-200" :class="{ 'opacity-40 pointer-events-none': !autoCopy }">
          <span class="text-base font-semibold text-gray-800">颜色格式</span>
          <div class="relative">
            <select 
              v-model="copyFormat" 
              :disabled="!autoCopy"
              class="appearance-none bg-gray-50 border border-gray-200 text-gray-800 text-sm rounded-lg focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500 block py-2 pl-4 pr-10 min-w-[160px] outline-none transition-colors hover:bg-gray-100 cursor-pointer font-mono font-medium"
            >
              <option v-for="fmt in formatOptions" :key="fmt" :value="fmt">{{ fmt }}</option>
            </select>
            <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-3 text-gray-500">
              <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path></svg>
            </div>
          </div>
        </div>

        <div class="h-px bg-gray-100 my-1"></div>

        <!-- 小写十六进制 Toggle -->
        <div class="flex items-center justify-between transition-opacity duration-200" :class="{ 'opacity-40 pointer-events-none': copyFormat !== '#RRGGBB' && copyFormat !== 'RRGGBB' }">
          <div class="flex-1 pr-4">
            <div class="text-base font-semibold text-gray-800 mb-1">小写字母</div>
            <div class="text-sm text-gray-500">使用小写字母表示十六进制颜色 (如 #ffffff)</div>
          </div>
          <button 
            @click="lowercaseHex = !lowercaseHex"
            :disabled="copyFormat !== '#RRGGBB' && copyFormat !== 'RRGGBB'"
            :class="lowercaseHex ? 'bg-blue-600' : 'bg-gray-200'"
            class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-600 focus:ring-offset-2 disabled:cursor-not-allowed"
          >
            <span :class="lowercaseHex ? 'translate-x-5' : 'translate-x-0'" class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"></span>
          </button>
        </div>
      </div>
    </div>
    
    <!-- Footer Tip -->
    <div class="mt-auto pt-4 pb-2 text-center">
      <div class="text-xs text-gray-400 flex items-center justify-center gap-1.5">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
        提示：点击快捷键按钮后，直接按下新的组合键。按 Esc 键取消。
      </div>
    </div>
  </div>
</template>
