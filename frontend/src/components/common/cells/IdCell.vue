<script setup lang="ts">
interface Props {
  id: string | number
  prefix?: string
}

const props = withDefaults(defineProps<Props>(), {
  prefix: '#'
})
</script>

<template>
  <span class="text-slate-300 text-sm">
    {{ prefix }}{{ id }}
  </span>
</template> 