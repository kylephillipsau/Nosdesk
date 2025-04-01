<!-- HeaderTitle.vue -->
<script setup lang="ts">
import { ref, watch } from 'vue';

interface Props {
  initialTitle: string;
  prefix?: string;
  placeholderText?: string;
}

const props = defineProps<Props>();
const emit = defineEmits(['updateTitle', 'updateTitlePreview']);

const title = ref(props.initialTitle);
const isEditing = ref(false);
const originalTitle = ref(props.initialTitle);

// Emit title updates as the user types for real-time preview
watch(title, (newValue) => {
  if (isEditing.value) {
    emit('updateTitlePreview', newValue);
  }
});

// Only emit final update when editing is complete
const handleBlur = () => {
  isEditing.value = false;
  if (title.value !== originalTitle.value) {
    emit('updateTitle', title.value);
    originalTitle.value = title.value;
  }
};

const handleClick = () => {
  isEditing.value = true;
};

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter') {
    event.preventDefault();
    (event.target as HTMLInputElement).blur();
  } else if (event.key === 'Escape') {
    // Reset to original value on Escape
    title.value = originalTitle.value;
    isEditing.value = false;
    // Also emit the original title to reset the preview
    emit('updateTitlePreview', originalTitle.value);
  }
};
</script>

<template>
  <div class="flex items-center gap-3 group flex-1">
    <span 
      v-if="prefix"
      class="text-slate-400 text-base font-medium flex items-center select-none"
      :class="{ 'opacity-50': isEditing }"
    >
      {{ prefix }}
    </span>
    
    <div class="flex-1 relative">
      <input
        v-model="title"
        type="text"
        @blur="handleBlur"
        @click="handleClick"
        @keydown="handleKeydown"
        class="w-full bg-transparent text-white text-xl font-semibold px-2 py-1 rounded-lg hover:bg-slate-700/50 focus:bg-slate-700 focus:outline-none transition-all duration-150"
        :class="{ 'bg-slate-700/50': isEditing }"
        :placeholder="placeholderText || 'Enter title...'"
      />
      
      <!-- Edit indicator -->
      <span 
        class="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 text-sm opacity-0 group-hover:opacity-100 transition-opacity duration-200"
        :class="{ 'opacity-0': isEditing }"
      >
        Click to edit
      </span>
    </div>
  </div>
</template>

<style scoped>
.transition-all {
  transition-property: all;
  transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
}

@media (prefers-reduced-motion: reduce) {
  .transition-all {
    transition: opacity 0.1s ease-in-out;
    transform: none;
  }

  /* Remove slide animation from edit indicator */
  .transition-all[enter-from],
  .transition-all[leave-to] {
    transform: none;
  }
}
</style>