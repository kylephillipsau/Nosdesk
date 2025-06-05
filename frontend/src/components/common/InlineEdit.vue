<script setup lang="ts">
import { ref, watch } from 'vue';

interface Props {
  modelValue: string;
  placeholder?: string;
  textSize?: 'sm' | 'base' | 'lg' | 'xl' | '2xl';
  canEdit?: boolean;
  prefix?: string;
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: 'Enter text...',
  textSize: 'base',
  canEdit: true,
  prefix: ''
});

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const isEditing = ref(false);
const originalValue = ref(props.modelValue);

// Update original value when modelValue changes from parent
watch(() => props.modelValue, (newValue) => {
  originalValue.value = newValue;
});

const handleClick = () => {
  if (props.canEdit && !isEditing.value) {
    isEditing.value = true;
  }
};

const handleBlur = () => {
  if (isEditing.value) {
    isEditing.value = false;
    emit('update:modelValue', props.modelValue);
  }
};

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter') {
    event.preventDefault();
    (event.target as HTMLInputElement).blur();
  } else if (event.key === 'Escape') {
    emit('update:modelValue', originalValue.value);
    isEditing.value = false;
  }
};

// Text size classes
const textSizeClasses = {
  sm: 'text-sm',
  base: 'text-base',
  lg: 'text-lg',
  xl: 'text-xl',
  '2xl': 'text-2xl'
};
</script>

<template>
  <div class="flex items-center gap-3 group flex-1">
    <span 
      v-if="prefix"
      class="text-slate-400 font-medium flex items-center select-none"
      :class="[textSizeClasses[textSize], { 'opacity-50': isEditing }]"
    >
      {{ prefix }}
    </span>
    
    <div class="flex-1 relative">
      <input
        :value="modelValue"
        @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
        type="text"
        class="w-full bg-transparent text-white font-semibold px-1 py-0.5 rounded-lg hover:bg-slate-700/50 focus:bg-slate-700 focus:outline-none transition-all duration-150 border-2 border-transparent focus:border-blue-500/50"
        :class="[
          textSizeClasses[textSize],
          { 
            'bg-slate-700/50': isEditing, 
            'cursor-pointer': canEdit && !isEditing,
            'cursor-text': isEditing,
            'cursor-default': !canEdit
          }
        ]"
        :placeholder="placeholder"
        :readonly="!canEdit"
        @click="handleClick"
        @blur="handleBlur"
        @keydown="handleKeydown"
      />
      
      <!-- Edit indicator -->
      <span 
        v-if="!isEditing && canEdit"
        class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 text-sm opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none select-none"
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
}
</style> 