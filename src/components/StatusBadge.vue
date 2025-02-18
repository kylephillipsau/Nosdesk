// components/StatusBadge.vue
<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  type: 'status' | 'priority'
  value: 'open' | 'in-progress' | 'closed' | 'low' | 'medium' | 'high'
  customClasses?: string
  short?: boolean
}>()

const statusColors = {
  open: 'bg-yellow-500',
  'in-progress': 'bg-blue-500',
  closed: 'bg-green-500'
}

const priorityColors = {
  low: 'text-green-400',
  medium: 'text-yellow-400',
  high: 'text-red-400'
}

const displayText = computed(() => {
  // If customClasses includes w- and h- (likely a circle indicator), don't show text
  if (props.customClasses?.includes('w-') && props.customClasses?.includes('h-')) {
    return ''
  }
  
  if (props.type === 'status') {
    return props.value
  }
  
  // For priority type: use short form if short prop is true
  if (props.short) {
    return props.value
  }
  
  return `${props.value} priority`
})

const badgeClasses = computed(() => {
  const baseClasses = props.type === 'status' 
    ? [
        statusColors[props.value as 'open' | 'in-progress' | 'closed'],
        'px-3 py-1 rounded-full text-sm'
      ]
    : [
        priorityColors[props.value as 'low' | 'medium' | 'high'],
        'font-medium'
      ]

  return props.customClasses 
    ? [...baseClasses, props.customClasses]
    : baseClasses
})
</script>

<template>
  <span :class="badgeClasses">
    {{ displayText }}
  </span>
</template>