<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router';
import { computed } from 'vue';

const router = useRouter();
const route = useRoute();

interface Props {
  title?: string;
  showCreateButton?: boolean;
  useRouteTitle?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  useRouteTitle: false
});

defineSlots<{
  actions(): any;
}>();

// Computed property to handle title logic
const displayTitle = computed(() => {
  if (props.title) {
    return props.title;
  }
  
  if (props.useRouteTitle) {
    // Try to get title from route meta
    return route.meta.title || 
           // Fallback to route name capitalized
           (route.name as string)?.toString().replace(/([A-Z])/g, ' $1').trim() ||
           // Final fallback
           'Page Title';
  }
  
  return 'Page Title';
});

const navigateToCreateTicket = () => {
  router.push('/tickets/create');
};
</script>

<template>
  <div class="flex justify-between items-center p-6 border-b border-gray-800">
    <div class="flex items-center gap-4">
      <h1 class="text-2xl font-semibold text-white">{{ displayTitle }}</h1>
      <button
        v-if="showCreateButton"
        @click="navigateToCreateTicket"
        class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors flex items-center gap-2"
      >
        <span>+</span>
        Create Ticket
      </button>
    </div>
    <div class="flex items-center gap-4">
      <slot name="actions"></slot>
    </div>
  </div>
</template>