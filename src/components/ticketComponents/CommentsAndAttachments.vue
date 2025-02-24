<!-- CommentsAndAttachments.vue -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import UserAvatar from "@/components/UserAvatar.vue";
import VoiceRecorder from "@/components/ticketComponents/VoiceRecorder.vue";
import AudioPlayer from "@/components/ticketComponents/AudioPlayer.vue";

interface NoteWithAttachments {
  id: number;
  content: string;
  author: string;
  createdAt: string;
  attachments?: { url: string; name: string }[];
}

const props = defineProps<{
  notes: NoteWithAttachments[];
  currentUser: string;
}>();

// Reactive data for new note and attachments
const newNoteContent = ref<string>("");
const newAttachments = ref<{ url: string; name: string }[]>([]);
const showAttachmentMenu = ref(false);
const attachmentButtonRef = ref<HTMLElement | null>(null);
const attachmentMenuRef = ref<HTMLElement | null>(null);
const shouldShowAbove = ref(false);
const showRecordingInterface = ref(false);
const showPreviewInterface = ref(false);
const currentRecording = ref<{ blob: Blob; duration: number } | null>(null);
const urlCreator = window.URL || window.webkitURL;

const emit = defineEmits<{
  (e: "addComment", value: { content: string; attachments: { url: string; name: string }[] }): void;
  (e: "deleteAttachment", value: { noteId: number; attachmentIndex: number }): void;
}>();

const addNote = () => {
  if (newNoteContent.value.trim() || newAttachments.value.length > 0) {
    emit("addComment", {
      content: newNoteContent.value,
      attachments: [...newAttachments.value],
    });
    newNoteContent.value = "";
    newAttachments.value = [];
  }
};

const handleFileUpload = (event: Event) => {
  const input = event.target as HTMLInputElement;
  if (input.files) {
    const files = Array.from(input.files);
    newAttachments.value = files.map((file) => ({
      url: URL.createObjectURL(file),
      name: file.name,
    }));
    showAttachmentMenu.value = false;
  }
};

const handleClickOutside = (event: MouseEvent) => {
  if (!attachmentButtonRef.value || !attachmentMenuRef.value) return;

  const target = event.target as Node;
  if (
    !attachmentButtonRef.value.contains(target) &&
    !attachmentMenuRef.value.contains(target)
  ) {
    showAttachmentMenu.value = false;
  }
};

const toggleAttachmentMenu = () => {
  showAttachmentMenu.value = !showAttachmentMenu.value;

  if (showAttachmentMenu.value && attachmentButtonRef.value) {
    const buttonRect = attachmentButtonRef.value.getBoundingClientRect();
    const windowHeight = window.innerHeight;
    const spaceBelow = windowHeight - buttonRect.bottom;
    shouldShowAbove.value = spaceBelow < 200;
  }
};

const startVoiceRecording = () => {
  showRecordingInterface.value = true;
  showAttachmentMenu.value = false;
};

const handleRecordingComplete = (recording: { blob: Blob; duration: number }) => {
  currentRecording.value = recording;
  showRecordingInterface.value = false;
  showPreviewInterface.value = true;
};

const handleRecordingCancel = () => {
  showRecordingInterface.value = false;
  currentRecording.value = null;
};

const confirmRecording = () => {
  if (currentRecording.value) {
    const url = urlCreator.createObjectURL(currentRecording.value.blob);
    newAttachments.value.push({
      url,
      name: `Voice Note ${new Date().toLocaleTimeString()}.webm`,
    });
  }
  showPreviewInterface.value = false;
  currentRecording.value = null;
};

const reRecord = () => {
  showPreviewInterface.value = false;
  currentRecording.value = null;
  startVoiceRecording();
};

const cancelPreview = () => {
  showPreviewInterface.value = false;
  currentRecording.value = null;
};

const deleteAttachment = (noteId: number, attachmentIndex: number) => {
  emit("deleteAttachment", { noteId, attachmentIndex });
};

onMounted(() => {
  document.addEventListener("mousedown", handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener("mousedown", handleClickOutside);
});

const formattedDate = (dateString: string): string => {
  const date = new Date(dateString);
  return date.toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
};
</script>

<template>
  <div class="flex flex-col bg-slate-800 rounded-2xl p-4 gap-2 shadow-lg h-fit">
    <h2 class="text-lg font-medium text-slate-100">Comments and Attachments</h2>

    <!-- List of Notes -->
    <div class="flex flex-col gap-2 space-y-3">
      <div v-for="note in props.notes" :key="note.id" class="flex flex-col gap-1 bg-slate-700 p-3 rounded-xl shadow-inner">
        <div class="flex flex-row gap-2">
          <UserAvatar :name="note.author" :showName="false" />
          <div class="flex-grow">
            <p class="text-slate-200">{{ note.content }}</p>
            <small class="text-slate-500">{{ note.author }} - {{ formattedDate(note.createdAt) }}</small>
          </div>
        </div>
        <div v-if="note.attachments && note.attachments.length > 0" class="mt-2">
          <template v-for="(attachment, index) in note.attachments" :key="attachment.url">
            <div 
              v-if="attachment.name.toLowerCase().includes('voice note')"
              class="flex flex-col gap-2"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <UserAvatar :name="note.author" :showName="false" />
                  <div class="flex flex-col">
                    <span class="text-sm text-slate-200">{{ note.author }}</span>
                    <span class="text-xs text-slate-400">{{ formattedDate(note.createdAt) }}</span>
                  </div>
                </div>
                <button
                  @click="deleteAttachment(note.id, index)"
                  class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-700 rounded transition-colors"
                  title="Delete recording"
                >
                  <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
                  </svg>
                </button>
              </div>
              <AudioPlayer
                :src="attachment.url"
                class="w-full"
              />
            </div>
            <img v-else :src="attachment.url" :alt="attachment.name" class="w-50 rounded-lg object-cover mb-2">
          </template>
        </div>
      </div>
    </div>

    <!-- Add New Note Form -->
    <div class="bg-slate-700 p-3 rounded-xl shadow-inner">
      <form @submit.prevent="addNote" class="flex flex-col gap-4">
        <div class="relative">
          <textarea
            v-model="newNoteContent"
            class="w-full bg-slate-800 text-slate-100 border border-slate-600 rounded-md p-3 placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
            placeholder="Add a new comment..."
            rows="3"
          ></textarea>
        </div>
        <div v-if="newAttachments.length > 0" class="mt-2">
          <template v-for="attachment in newAttachments" :key="attachment.url">
            <div 
              v-if="attachment.name.toLowerCase().includes('voice note')"
              class="flex flex-col gap-2"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <UserAvatar :name="props.currentUser" :showName="false" />
                  <div class="flex flex-col">
                    <span class="text-sm text-slate-200">{{ props.currentUser }}</span>
                    <span class="text-xs text-slate-400">{{ formattedDate(new Date().toISOString()) }}</span>
                  </div>
                </div>
                <button
                  @click="newAttachments = newAttachments.filter(a => a.url !== attachment.url)"
                  class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-700 rounded transition-colors"
                  title="Delete recording"
                >
                  <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
                  </svg>
                </button>
              </div>
              <AudioPlayer
                :src="attachment.url"
                class="w-full"
              />
            </div>
            <img v-else :src="attachment.url" :alt="attachment.name" class="w-50 rounded-lg object-cover mb-2">
          </template>
        </div>

        <!-- Voice Recorder and Preview Components -->
        <VoiceRecorder
          v-if="showRecordingInterface"
          @recording-complete="handleRecordingComplete"
          @cancel="handleRecordingCancel"
        />

        <div v-if="showPreviewInterface && currentRecording" class="bg-slate-800 rounded-lg p-3">
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <UserAvatar :name="props.currentUser" :showName="false" />
              <div class="flex flex-col">
                <span class="text-sm text-slate-200">{{ props.currentUser }}</span>
                <span class="text-xs text-slate-400">{{ formattedDate(new Date().toISOString()) }}</span>
              </div>
            </div>
          </div>
          <AudioPlayer
            :src="urlCreator.createObjectURL(currentRecording.blob)"
          />
          <div class="flex justify-end gap-2 mt-3">
            <button
              type="button"
              @click="reRecord"
              class="px-3 py-1.5 text-slate-300 hover:text-white transition-colors"
            >
              Re-record
            </button>
            <button
              type="button"
              @click="confirmRecording"
              class="px-3 py-1.5 bg-green-500 text-white text-sm rounded hover:bg-green-600 transition-colors"
            >
              Confirm
            </button>
            <button
              type="button"
              @click="cancelPreview"
              class="px-3 py-1.5 text-slate-300 hover:text-white transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>

        <div class="flex gap-2">
          <button type="submit" class="flex-1 bg-blue-500 text-white h-10 px-4 rounded-md hover:bg-blue-600 transition-colors">
            Add
          </button>
          <div class="relative">
            <button
              ref="attachmentButtonRef"
              type="button"
              @click="toggleAttachmentMenu"
              class="h-10 px-4 bg-slate-600 text-white rounded-md hover:bg-slate-500 transition-colors flex items-center justify-center"
              aria-label="Add attachment"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
              </svg>
            </button>
            <div
              v-if="showAttachmentMenu"
              ref="attachmentMenuRef"
              class="absolute w-48 bg-slate-800 rounded-lg shadow-lg py-1 z-10 border border-slate-600"
              :class="[shouldShowAbove ? 'bottom-full right-0 mb-2' : 'top-full right-0 mt-2']"
            >
              <label class="flex items-center px-4 py-2 text-sm text-slate-300 hover:bg-slate-700 cursor-pointer">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                  <path d="M5.5 13a3.5 3.5 0 01-.369-6.98 4 4 0 117.753-1.977A4.5 4.5 0 1113.5 13H11V9.413l1.293 1.293a1 1 0 001.414-1.414l-3-3a1 1 0 00-1.414 0l-3 3a1 1 0 001.414 1.414L9 9.414V13H5.5z" />
                  <path d="M9 13h2v5a1 1 0 11-2 0v-5z" />
                </svg>
                Upload File
                <input type="file" @change="handleFileUpload" multiple class="hidden" />
              </label>
              <button
                @click="startVoiceRecording"
                class="w-full flex items-center px-4 py-2 text-sm text-slate-300 hover:bg-slate-700"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M7 4a3 3 0 016 0v4a3 3 0 11-6 0V4zm4 10.93A7.001 7.001 0 0017 8a1 1 0 10-2 0A5 5 0 015 8a1 1 0 00-2 0 7.001 7.001 0 006 6.93V17H6a1 1 0 100 2h8a1 1 0 100-2h-3v-2.07z" clip-rule="evenodd" />
                </svg>
                Voice Note
              </button>
            </div>
          </div>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.attachment-menu-enter-active,
.attachment-menu-leave-active {
  transition: opacity 0.15s ease-in-out, transform 0.15s ease-in-out;
}

.attachment-menu-enter-from,
.attachment-menu-leave-to {
  opacity: 0;
  transform: translateY(-0.25rem);
}

.attachment-menu-enter-from[data-position="above"],
.attachment-menu-leave-to[data-position="above"] {
  transform: translateY(0.25rem);
}
</style>