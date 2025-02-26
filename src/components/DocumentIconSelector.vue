<!-- DocumentIconSelector.vue -->
<script setup lang="ts">
import { ref, watch } from 'vue';

interface Props {
  initialIcon?: string;
  size?: 'sm' | 'md' | 'lg';
}

const props = withDefaults(defineProps<Props>(), {
  initialIcon: '📄',
  size: 'md',
});

const emit = defineEmits(['update:icon']);

const currentIcon = ref(props.initialIcon);
const showPicker = ref(false);

// Common document-related emojis
const commonIcons = [
  '📄', '📝', '📑', '📃', '📜', '📋', '📁', '📂', 
  '📓', '📔', '📕', '📗', '📘', '📙', '📚', '📖',
  '🗒️', '🗓️', '📊', '📈', '📉', '🔍', '🔎', '🔖',
  '💡', '⚙️', '🔧', '🛠️', '🧰', '📌', '📎', '🔗'
];

// Watch for changes to the selected icon
watch(currentIcon, (newIcon) => {
  emit('update:icon', newIcon);
});

const selectIcon = (icon: string) => {
  currentIcon.value = icon;
  showPicker.value = false;
};

const togglePicker = () => {
  showPicker.value = !showPicker.value;
};

const sizeClasses = {
  sm: 'text-lg',
  md: 'text-xl',
  lg: 'text-2xl'
};
</script>

<template>
  <div class="relative">
    <!-- Current icon display -->
    <button 
      @click="togglePicker"
      class="flex items-center justify-center hover:bg-slate-700 rounded-lg p-1.5 transition-colors focus:outline-none"
      :class="[sizeClasses[size]]"
      aria-label="Select document icon"
    >
      <span class="select-none">{{ currentIcon }}</span>
    </button>

    <!-- Icon picker dropdown -->
    <div 
      v-if="showPicker" 
      class="absolute top-full left-0 mt-1 bg-slate-800 border border-slate-700 rounded-lg shadow-lg p-2 z-50"
    >
      <div class="grid grid-cols-8 gap-1 max-w-xs">
        <button
          v-for="icon in commonIcons"
          :key="icon"
          @click="selectIcon(icon)"
          class="flex items-center justify-center p-1.5 hover:bg-slate-700 rounded-md transition-colors"
          :class="{ 'bg-slate-700': currentIcon === icon }"
        >
          <span class="text-lg select-none">{{ icon }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Add any additional styles here */
</style> 