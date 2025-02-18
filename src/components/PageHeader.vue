// components/PageHeader.vue
<script setup lang="ts">
defineProps<{
  title: string
}>()

defineSlots<{
  actions(): any
}>()
</script>

<template>
  <div class="flex justify-between items-center p-6 border-b border-gray-800">
    <h1 class="text-2xl font-semibold text-white">{{ title }}</h1>
    <div class="flex items-center gap-4">
      <slot name="actions"></slot>
    </div>
  </div>
</template>