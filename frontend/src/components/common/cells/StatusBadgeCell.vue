<script setup lang="ts">
interface Props {
  value: string
  type?: 'status' | 'priority' | 'warranty' | 'role'
}

const props = withDefaults(defineProps<Props>(), {
  type: 'status'
})

const getStatusClasses = (value: string, type: string) => {
  const baseClasses = 'text-xs px-2 py-1 rounded-full whitespace-nowrap'
  
  if (type === 'warranty') {
    switch (value) {
      case 'Active':
        return `${baseClasses} text-green-400 bg-green-900/30`
      case 'Warning':
        return `${baseClasses} text-yellow-400 bg-yellow-900/30`
      case 'Expired':
        return `${baseClasses} text-red-400 bg-red-900/30`
      case 'Unknown':
        return `${baseClasses} text-slate-400 bg-slate-700`
      default:
        return `${baseClasses} text-slate-400 bg-slate-700`
    }
  }
  
  if (type === 'role') {
    return `${baseClasses} bg-slate-700 text-slate-200`
  }
  
  // Default for status/priority
  return `${baseClasses} bg-slate-700 text-slate-200`
}
</script>

<template>
  <span :class="getStatusClasses(value, type)">
    {{ value }}
  </span>
</template> 