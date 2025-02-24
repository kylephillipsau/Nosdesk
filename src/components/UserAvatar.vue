<!-- # src/components/UserAvatar.vue -->
<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  name: string;
  showName?: boolean;
  size?: 'xs' | 'sm' | 'md' | 'lg';
  avatar?: string | null;
}

const props = withDefaults(defineProps<Props>(), {
  showName: true,
  size: 'md',
  avatar: null
})

const getInitials = (name: string) => {
  return name
    .split(' ')
    .map(word => word.charAt(0))
    .join('')
    .toUpperCase()
    .slice(0, 2)
}

const getBackgroundColor = (name: string) => {
  if (!name) return 'hsl(0, 70%, 35%)'
  
  // Get the first letter and convert to uppercase
  const firstLetter = name.charAt(0).toUpperCase()
  
  // Get position in alphabet (A = 0, B = 1, etc.)
  const position = firstLetter.charCodeAt(0) - 65
  
  // Convert position to a value between 0 and 360 (hue range)
  // We'll mod by 26 (alphabet length) first to handle non-letters
  const hue = (position % 26) * (360 / 26)
  
  // Return HSL color with fixed saturation and lightness
  return `hsl(${hue}, 70%, 35%)`
}

const sizeClasses = computed(() => {
  // Base sizes in rem units
  const sizes = {
    xs: {
      base: 'h-[1.25rem] w-[1.25rem]', // 20px at default font size
      text: 'text-[10px]',
      responsive: 'sm:h-[1.375rem] sm:w-[1.375rem]' // 22px on small screens
    },
    sm: {
      base: 'h-[1.5rem] w-[1.5rem]', // 24px at default font size
      text: 'text-xs',
      responsive: 'sm:h-[1.75rem] sm:w-[1.75rem]' // Slightly larger on small screens
    },
    md: {
      base: 'h-[2rem] w-[2rem]', // 32px at default font size
      text: 'text-sm',
      responsive: 'sm:h-[2.25rem] sm:w-[2.25rem]' // Slightly larger on small screens
    },
    lg: {
      base: 'h-[2.25rem] w-[2.25rem]', // 36px at default font size
      text: 'text-sm',
      responsive: 'sm:h-[2.5rem] sm:w-[2.5rem]' // 40px on small screens
    }
  }

  return sizes[props.size] || sizes.md
})

// Computed class for the name text that scales with viewport
const nameTextClasses = computed(() => {
  const baseClasses = 'text-white transition-all duration-200'
  
  switch (props.size) {
    case 'xs':
      return `${baseClasses} text-[10px] sm:text-xs`
    case 'sm':
      return `${baseClasses} text-xs sm:text-sm`
    case 'lg':
      return `${baseClasses} text-sm` // Keeping text size consistent for header
    default: // md
      return `${baseClasses} text-xs sm:text-sm`
  }
})
</script>

<template>
  <div class="flex items-center gap-2">
    <div 
      :class="[sizeClasses.base, sizeClasses.responsive]"
      class="rounded-full flex items-center justify-center flex-shrink-0 font-medium text-white transition-all duration-200"
      :style="{ backgroundColor: getBackgroundColor(name) }"
    >
      <img 
        v-if="avatar" 
        :src="avatar" 
        :alt="name"
        class="w-full h-full rounded-full object-cover"
      />
      <span v-else :class="sizeClasses.text">{{ getInitials(name) }}</span>
    </div>
    <span 
      v-if="showName" 
      :class="nameTextClasses"
    >
      {{ name }}
    </span>
  </div>
</template>