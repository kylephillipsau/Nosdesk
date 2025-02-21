<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';

const props = defineProps<{
  title: string;
}>();

const currentTitle = ref('');
const isAnimating = ref(false);

// Initialize the title on mount
onMounted(() => {
  currentTitle.value = props.title;
});

watch(() => props.title, async (newTitle) => {
  if (isAnimating.value) return;
  
  isAnimating.value = true;
  await new Promise(resolve => setTimeout(resolve, 300));
  currentTitle.value = newTitle;
  isAnimating.value = false;
}, { immediate: true });
</script>

<template>
  <div class="relative overflow-hidden">
    <h1 
      class="text-xl font-semibold text-white transition-all duration-300 ease-in-out"
      :class="{ '-translate-x-full opacity-0': isAnimating }"
    >
      {{ currentTitle }}
    </h1>
  </div>
</template>
