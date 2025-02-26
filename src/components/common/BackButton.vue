<script setup lang="ts">
import { ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';

const props = defineProps<{
  // Default fallback route if no previous route exists
  fallbackRoute?: string;
  // Custom label for the button (defaults to "Go back")
  label?: string;
}>();

const route = useRoute();
const router = useRouter();
const previousRoute = ref<string>('');

// Watch route changes to store previous route
watch(() => route.fullPath, (newPath, oldPath) => {
  if (oldPath) {
    previousRoute.value = oldPath;
  }
}, { immediate: true });

const handleBack = () => {
  if (previousRoute.value) {
    router.push(previousRoute.value);
  } else if (props.fallbackRoute) {
    router.push(props.fallbackRoute);
  } else {
    // Default fallback to home if no fallback route is provided
    router.push('/');
  }
};
</script>

<template>
  <button
    @click="handleBack"
    class="text-slate-400 hover:text-white text-sm flex items-center gap-1 group"
  >
    <span class="text-xs group-hover:-translate-x-0.5 transition-transform">←</span>
    {{ label || 'Go back' }}
  </button>
</template> 