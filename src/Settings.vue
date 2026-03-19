<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const currentShortcut = ref('');
const recording = ref(false);
const keys = ref<Set<string>>(new Set());

onMounted(async () => {
  try {
    currentShortcut.value = await invoke('get_shortcut');
  } catch (e) {
    console.error(e);
  }
});

const startRecording = () => {
  recording.value = true;
  keys.value.clear();
  currentShortcut.value = '请按下快捷键...';
};

const handleKeyDown = (e: KeyboardEvent) => {
  if (!recording.value) return;
  e.preventDefault();
  
  let key = e.key;
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
  // If only modifiers are pressed, cancel
  if (Array.from(keys.value).every(k => ['Control', 'Alt', 'Shift', 'Meta'].includes(k))) {
    currentShortcut.value = '无效快捷键，请重新设置';
    return;
  }
  
  // Clean up format for Tauri
  let formatted = currentShortcut.value;
  formatted = formatted.replace('Command', 'Super');
  
  invoke('set_shortcut', { newShortcut: formatted })
    .then(() => {
      console.log('Shortcut saved');
      currentShortcut.value = formatted + ' (已保存)';
      setTimeout(() => {
        if (currentShortcut.value.includes('(已保存)')) {
          currentShortcut.value = formatted;
        }
      }, 2000);
    })
    .catch((err) => {
      console.error('Failed to save shortcut:', err);
      currentShortcut.value = '保存失败: ' + err;
    });
};

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  window.addEventListener('keyup', handleKeyUp);
});
</script>

<template>
  <div class="settings-container">
    <div class="header">
      <h2>设置</h2>
    </div>
    
    <div class="section">
      <div class="section-title">快捷键</div>
      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-name">唤醒/隐藏取色器</div>
          <div class="setting-desc">全局快捷键，用于快速打开或关闭屏幕取色器</div>
        </div>
        <button 
          class="shortcut-btn" 
          :class="{ recording }"
          @click="startRecording"
        >
          {{ currentShortcut }}
        </button>
      </div>
    </div>
    
    <div class="footer">
      <div class="tip">提示：点击快捷键按钮后，直接在键盘上按下新的组合键即可。</div>
    </div>
  </div>
</template>

<style scoped>
.settings-container {
  padding: 20px;
  color: #333;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  height: 100vh;
  box-sizing: border-box;
  background: #f5f5f5;
}

.header {
  margin-bottom: 24px;
}

.header h2 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
}

.section {
  background: white;
  border-radius: 8px;
  padding: 16px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
  margin-bottom: 20px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #666;
  margin-bottom: 16px;
  text-transform: uppercase;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.setting-info {
  flex: 1;
}

.setting-name {
  font-size: 16px;
  font-weight: 500;
  margin-bottom: 4px;
}

.setting-desc {
  font-size: 13px;
  color: #888;
}

.shortcut-btn {
  background: #f0f0f0;
  border: 1px solid #ddd;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-family: monospace;
  cursor: pointer;
  transition: all 0.2s;
  min-width: 120px;
  text-align: center;
}

.shortcut-btn:hover {
  background: #e4e4e4;
}

.shortcut-btn.recording {
  background: #e6f7ff;
  border-color: #91d5ff;
  color: #1890ff;
  box-shadow: 0 0 0 2px rgba(24,144,255,0.2);
}

.footer {
  margin-top: auto;
  padding-top: 20px;
}

.tip {
  font-size: 13px;
  color: #888;
}
</style>
