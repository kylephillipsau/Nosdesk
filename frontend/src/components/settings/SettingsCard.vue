<script setup lang="ts">
defineProps<{
  title: string;
}>();
</script>

<template>
  <div class="bg-slate-800 rounded-lg p-6">
    <h2 class="text-lg font-medium text-white mb-4">{{ title }}</h2>
    <div class="flex flex-col gap-4">
      <slot></slot>
    </div>
  </div>
</template> 