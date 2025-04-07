<script setup lang="ts">
const props = defineProps<{
  title: string;
  description: string;
  modelValue: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>();

const toggleValue = () => {
  emit('update:modelValue', !props.modelValue);
};
</script>

<template>
  <div class="bg-slate-700/50 rounded-md p-4">
    <div class="flex justify-between items-center">
      <div>
        <h3 class="text-sm font-medium text-white">{{ props.title }}</h3>
        <p class="text-sm text-slate-400 mt-1">{{ props.description }}</p>
      </div>
      <label class="relative inline-flex items-center cursor-pointer" @click="toggleValue">
        <input type="checkbox" class="sr-only peer" :checked="props.modelValue" />
        <div class="w-11 h-6 bg-slate-600 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
      </label>
    </div>
  </div>
</template> 