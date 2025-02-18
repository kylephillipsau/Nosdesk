<!-- # src/components/UserAvatar.vue -->
<script setup lang="ts">
interface Props {
  name: string
  showName?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showName: true
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
</script>

<template>
  <div class="flex items-center gap-1">
    <div 
      class="w-7 h-7 rounded-full flex items-center justify-center flex-shrink-0 text-xs font-medium text-white"
      :style="{ backgroundColor: getBackgroundColor(name) }"
    >
      {{ getInitials(name) }}
    </div>
    <span v-if="showName">{{ name }}</span>
  </div>
</template>