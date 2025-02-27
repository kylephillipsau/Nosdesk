<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import UserAvatar from "@/components/UserAvatar.vue";
import VoiceRecorder from "@/components/ticketComponents/VoiceRecorder.vue";
import AttachmentPreview from "@/components/ticketComponents/AttachmentPreview.vue";
import AudioPreview from "@/components/ticketComponents/AudioPreview.vue";

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
const isDraggingFile = ref(false);

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
    files.forEach(file => {
      if (file.type.startsWith('audio/')) {
        const url = URL.createObjectURL(file);
        currentRecording.value = { blob: file, duration: 0 };
        showPreviewInterface.value = true;
      } else {
        newAttachments.value.push({
          url: URL.createObjectURL(file),
          name: file.name,
        });
      }
    });
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

const handleAudioSubmit = (data: { blob: Blob; name: string }) => {
  const url = urlCreator.createObjectURL(data.blob);
  newAttachments.value.push({
    url,
    name: data.name
  });
  showPreviewInterface.value = false;
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

const handleDragEnter = (event: DragEvent) => {
  event.preventDefault();
  event.stopPropagation();
  isDraggingFile.value = true;
};

const handleDragLeave = (event: DragEvent) => {
  event.preventDefault();
  event.stopPropagation();
  const target = event.currentTarget as HTMLElement;
  const relatedTarget = event.relatedTarget as Node;
  if (!target?.contains(relatedTarget)) {
    isDraggingFile.value = false;
  }
};

const handleDragOver = (event: DragEvent) => {
  event.preventDefault();
  event.stopPropagation();
};

const handleDrop = (event: DragEvent) => {
  event.preventDefault();
  event.stopPropagation();
  isDraggingFile.value = false;
  const files = Array.from(event.dataTransfer?.files || []);
  files.forEach(file => {
    if (file.type.startsWith('audio/')) {
      currentRecording.value = { blob: file, duration: 0 };
      showPreviewInterface.value = true;
    } else {
      newAttachments.value.push({
        url: URL.createObjectURL(file),
        name: file.name,
      });
    }
  });
};
</script>

<template>
  <div class="flex flex-col bg-slate-800 rounded-2xl p-4 gap-2 shadow-lg h-fit">
    <h2 class="text-lg font-medium text-slate-100">Comments and Attachments</h2>

    <!-- List of Notes -->
    <div class="flex flex-col gap-2 space-y-3">
      <div v-for="note in props.notes" :key="note.id" class="flex flex-col gap-2 bg-slate-700 px-3 py-2 rounded-xl shadow-inner">
        <div class="flex flex-row gap-2 justify-between">
          <div class="flex gap-2 justify-center items-center">
            <UserAvatar :name="note.author" :showName="false" size="sm"/>
            <div class="flex flex-col flex-grow">
              <p class="text-slate-200">{{ note.content }}</p>
              <small class="text-slate-400">{{ note.author }} - {{ formattedDate(note.createdAt) }}</small>
            </div>
          </div>
          <button
            type="button"
            @click="deleteAttachment(note.id, 0)"
            class="p-1.5 text-slate-400 hover:text-white hover:bg-slate-600 rounded transition-colors"
            title="Delete note"
          >
            <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
            </svg>
          </button>
        </div>
        <div v-if="note.attachments && note.attachments.length > 0" class="flex flex-col gap-2">
          <AttachmentPreview
            v-for="(attachment, index) in note.attachments"
            :key="attachment.url"
            :attachment="attachment"
            :author="note.author"
            :timestamp="formattedDate(note.createdAt)"
            :show-delete="false"
            @delete="deleteAttachment(note.id, index)"
          />
        </div>
      </div>
    </div>

    <!-- Add New Note Form -->
    <div 
      class="bg-slate-700 p-3 rounded-xl shadow-inner relative"
      @dragenter="handleDragEnter"
      @dragleave="handleDragLeave"
      @dragover="handleDragOver"
      @drop="handleDrop"
    >
      <!-- Drag overlay with pointer-events-none to avoid capturing mouse events -->
      <div 
        v-if="isDraggingFile"
        class="absolute inset-0 bg-blue-500/10 border-2 border-blue-500 border-dashed rounded-xl flex items-center justify-center pointer-events-none"
        style="z-index: 30;"
      >
        <div class="bg-slate-800 rounded-lg px-4 py-2 text-blue-500 flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path d="M5.5 13a3.5 3.5 0 01-.369-6.98 4 4 0 117.753-1.977A4.5 4.5 0 1113.5 13H11V9.413l1.293 1.293a1 1 0 001.414-1.414l-3-3a1 1 0 00-1.414 0l-3 3a1 1 0 001.414 1.414L9 9.414V13H5.5z" />
            <path d="M9 13h2v5a1 1 0 11-2 0v-5z" />
          </svg>
          Drop files here
        </div>
      </div>

      <form @submit.prevent="addNote" class="flex flex-col gap-2">
        <div class="relative">
          <textarea
            v-model="newNoteContent"
            class="w-full bg-slate-900/50 text-slate-100 border border-slate-600 rounded-md p-3 placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
            placeholder="Add a new comment..."
            rows="3"
          ></textarea>
        </div>

        <!-- New attachments -->
        <div v-if="newAttachments.length > 0" class="mt-2">
          <AttachmentPreview
            v-for="attachment in newAttachments"
            :key="attachment.url"
            :attachment="attachment"
            :author="props.currentUser"
            :timestamp="formattedDate(new Date().toISOString())"
            :is-new="true"
            :show-delete="true"
            @delete="newAttachments = newAttachments.filter(a => a.url !== attachment.url)"
            @submit="addNote"
          />
        </div>

        <!-- Voice Recorder and Preview Components -->
        <VoiceRecorder
          v-if="showRecordingInterface"
          @recording-complete="handleRecordingComplete"
          @cancel="handleRecordingCancel"
        />

        <AudioPreview
          v-if="showPreviewInterface && currentRecording"
          :blob="currentRecording.blob"
          :author="props.currentUser"
          :timestamp="formattedDate(new Date().toISOString())"
          :show-recording-controls="showRecordingInterface"
          @confirm="confirmRecording"
          @submit="handleAudioSubmit"
          @re-record="reRecord"
          @cancel="cancelPreview"
        />

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