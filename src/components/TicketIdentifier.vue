<!-- TicketIdentifier.vue -->
<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  ticketId?: number | string;
  showPrefix?: boolean;
  size?: 'sm' | 'md' | 'lg';
}

const props = withDefaults(defineProps<Props>(), {
  showPrefix: true,
  size: 'md'
});

// Compute the display text based on the ticket ID
const displayText = computed(() => {
  if (!props.ticketId) return '';
  return props.showPrefix ? `#${props.ticketId}` : `${props.ticketId}`;
});

// Size classes for different display sizes
const sizeClasses = computed(() => {
  switch (props.size) {
    case 'sm': return 'text-sm';
    case 'lg': return 'text-lg font-semibold';
    default: return 'text-base';
  }
});
</script>

<template>
  <div v-if="ticketId" class="flex items-center">
    <span 
      class="text-slate-400 font-medium flex items-center select-none"
      :class="sizeClasses"
    >
      {{ displayText }}
    </span>
  </div>
</template>

<style scoped>
/* Add any additional styles here */
</style> 