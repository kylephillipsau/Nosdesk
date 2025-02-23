<!-- components/ticketComponents/DeviceModal.vue -->
<script setup lang="ts">
import { ref } from 'vue';
import type { Device } from '@/types/ticket';
import Modal from '@/components/Modal.vue';

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'add-device', device: Device): void;
}>();

const createEmptyDevice = (): Device => ({
  id: crypto.randomUUID(),
  name: '',
  hostname: '',
  serialNumber: '',
  model: '',
  warrantyStatus: ''
});

const device = ref<Device>(createEmptyDevice());

const handleSubmit = () => {
  emit('add-device', { ...device.value });
  device.value = createEmptyDevice();
  emit('close');
};
</script>

<template>
  <Modal :show="show" title="Add Device" @close="emit('close')">
    <form @submit.prevent="handleSubmit" class="flex flex-col gap-4">
      <!-- Name -->
      <div class="flex flex-col gap-1">
        <label for="name" class="text-sm text-slate-400">Name</label>
        <input
          id="name"
          v-model="device.name"
          type="text"
          required
          class="bg-slate-700 text-slate-200 rounded-lg p-2 border-none focus:ring-2 focus:ring-blue-500"
          placeholder="Enter device name"
        />
      </div>

      <!-- Hostname -->
      <div class="flex flex-col gap-1">
        <label for="hostname" class="text-sm text-slate-400">Hostname</label>
        <input
          id="hostname"
          v-model="device.hostname"
          type="text"
          required
          class="bg-slate-700 text-slate-200 rounded-lg p-2 border-none focus:ring-2 focus:ring-blue-500"
          placeholder="Enter hostname"
        />
      </div>

      <!-- Serial Number -->
      <div class="flex flex-col gap-1">
        <label for="serialNumber" class="text-sm text-slate-400">Serial Number</label>
        <input
          id="serialNumber"
          v-model="device.serialNumber"
          type="text"
          required
          class="bg-slate-700 text-slate-200 rounded-lg p-2 border-none focus:ring-2 focus:ring-blue-500"
          placeholder="Enter serial number"
        />
      </div>

      <!-- Model -->
      <div class="flex flex-col gap-1">
        <label for="model" class="text-sm text-slate-400">Model</label>
        <input
          id="model"
          v-model="device.model"
          type="text"
          required
          class="bg-slate-700 text-slate-200 rounded-lg p-2 border-none focus:ring-2 focus:ring-blue-500"
          placeholder="Enter model"
        />
      </div>

      <!-- Warranty Status -->
      <div class="flex flex-col gap-1">
        <label for="warrantyStatus" class="text-sm text-slate-400">Warranty Status</label>
        <input
          id="warrantyStatus"
          v-model="device.warrantyStatus"
          type="text"
          required
          class="bg-slate-700 text-slate-200 rounded-lg p-2 border-none focus:ring-2 focus:ring-blue-500"
          placeholder="Enter warranty status"
        />
      </div>

      <!-- Buttons -->
      <div class="flex justify-end gap-3 mt-4">
        <button
          type="button"
          @click="emit('close')"
          class="px-4 py-2 text-sm text-slate-300 hover:text-slate-100"
        >
          Cancel
        </button>
        <button
          type="submit"
          class="px-4 py-2 text-sm bg-blue-500 text-white rounded-lg hover:bg-blue-600"
        >
          Add Device
        </button>
      </div>
    </form>
  </Modal>
</template> 