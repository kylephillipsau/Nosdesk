<!-- TicketTitle.vue -->
<script setup lang="ts">
import { ref, watch } from 'vue';

interface Props {
  ticketId: number;
  initialTitle: string;
}

const props = defineProps<Props>();
const emit = defineEmits(['updateTitle']);

const title = ref(props.initialTitle);

watch(title, (newTitle) => {
  emit('updateTitle', newTitle);
});
</script>

<template>
  <div class="flex items-center justify-center gap-2 px-4">
    <div class="text-slate-400 text-sm font-medium flex items-center">#{{ props.ticketId }}</div>
    <input
      v-model="title"
      type="text"
      class="flex-1 bg-transparent text-white text-2xl font-semibold px-1 py-0.5 rounded hover:bg-slate-800/50 focus:bg-slate-800 focus:outline-none transition-colors duration-150"
      placeholder="Enter ticket title..."
    />
  </div>
</template>