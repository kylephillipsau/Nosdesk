<!-- AudioPreview.vue -->
<script setup lang="ts">
import UserAvatar from "@/components/UserAvatar.vue";
import AudioPlayer from "@/components/ticketComponents/AudioPlayer.vue";

interface Props {
  blob: Blob;
  author: string;
  timestamp: string;
  showRecordingControls?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  showRecordingControls: false
});

const emit = defineEmits<{
  (e: 'confirm'): void;
  (e: 'reRecord'): void;
  (e: 'cancel'): void;
}>();

const urlCreator = window.URL || window.webkitURL;
</script>

<template>
  <div class="bg-slate-800 rounded-lg p-3">
    <div class="flex items-center justify-between mb-2">
      <div class="flex items-center gap-2">
        <UserAvatar :name="author" :showName="false" />
        <div class="flex flex-col">
          <span class="text-sm text-slate-200">{{ author }}</span>
          <span class="text-xs text-slate-400">{{ timestamp }}</span>
        </div>
      </div>
      <button
        type="button"
        @click="emit('cancel')"
        class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-700 rounded transition-colors"
        title="Delete audio"
      >
        <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
        </svg>
      </button>
    </div>
    <AudioPlayer
      :src="urlCreator.createObjectURL(blob)"
    />
    <div class="flex justify-end gap-2 mt-3">
      <template v-if="showRecordingControls">
        <button
          type="button"
          @click="emit('reRecord')"
          class="px-3 py-1.5 text-slate-300 hover:text-white transition-colors"
        >
          Re-record
        </button>
        <button
          type="button"
          @click="emit('confirm')"
          class="px-3 py-1.5 bg-green-500 text-white text-sm rounded hover:bg-green-600 transition-colors"
        >
          Confirm
        </button>
      </template>
      <template v-else>
        <button
          type="button"
          @click="emit('confirm')"
          class="px-3 py-1.5 bg-green-500 text-white text-sm rounded hover:bg-green-600 transition-colors"
        >
          Confirm
        </button>
      </template>
      <button
        type="button"
        @click="emit('cancel')"
        class="px-3 py-1.5 text-slate-300 hover:text-white transition-colors"
      >
        Cancel
      </button>
    </div>
  </div>
</template> 