<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { marked } from 'marked'
import type { MarkedOptions } from 'marked'
import hljs from 'highlight.js'
import 'highlight.js/styles/github-dark.css'

interface Props {
  content?: string;
}

const props = defineProps<Props>()
const parsedContent = ref('')

const parseMarkdown = async (content: string) => {
  // Configure marked with highlight.js for code highlighting
  const options = {
    async: false,
    breaks: true,
    gfm: true,
    highlight: (code: string, lang?: string) => {
      if (lang && hljs.getLanguage(lang)) {
        return hljs.highlight(code, { language: lang }).value
      }
      return hljs.highlightAuto(code).value
    }
  } as MarkedOptions

  marked.setOptions(options)

  // Parse markdown content
  const parsed = await marked.parse(content)
  parsedContent.value = parsed
}

// Watch for content changes
watch(() => props.content, (newContent) => {
  if (newContent) {
    parseMarkdown(newContent)
  } else {
    parsedContent.value = ''
  }
}, { immediate: true })

onMounted(async () => {
  if (props.content) {
    await parseMarkdown(props.content)
  }
})
</script>

<template>
  <div 
    class="prose-content"
    v-html="parsedContent"
  ></div>
</template>

<style scoped>
.prose-content {
  color: #cbd5e1; /* text-slate-300 */
  max-width: none;
}

.prose-content :deep(h1), 
.prose-content :deep(h2), 
.prose-content :deep(h3), 
.prose-content :deep(h4) {
  color: white;
  font-weight: 500;
  margin-top: 2rem;
  margin-bottom: 1rem;
}

.prose-content :deep(h1) {
  font-size: 1.875rem;
}

.prose-content :deep(h2) {
  font-size: 1.5rem;
}

.prose-content :deep(h3) {
  font-size: 1.25rem;
}

.prose-content :deep(p) {
  margin-bottom: 1rem;
}

.prose-content :deep(a) {
  color: #60a5fa; /* text-blue-400 */
  text-decoration: none;
}

.prose-content :deep(a:hover) {
  color: #93c5fd; /* text-blue-300 */
}

.prose-content :deep(code) {
  background-color: #334155; /* bg-slate-700 */
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  font-size: 0.875rem;
}

.prose-content :deep(pre) {
  background-color: #1e293b; /* bg-slate-800 */
  border-radius: 0.5rem;
  padding: 1rem;
  margin-top: 1rem;
  margin-bottom: 1rem;
}

.prose-content :deep(pre code) {
  background-color: transparent;
  padding: 0;
}

.prose-content :deep(ul), 
.prose-content :deep(ol) {
  margin-top: 1rem;
  margin-bottom: 1rem;
  padding-left: 1.5rem;
}

.prose-content :deep(li) {
  margin-bottom: 0.5rem;
}

.prose-content :deep(blockquote) {
  border-left: 4px solid #475569; /* border-slate-600 */
  padding-left: 1rem;
  margin-top: 1rem;
  margin-bottom: 1rem;
  font-style: italic;
}

.prose-content :deep(img) {
  border-radius: 0.5rem;
  margin-top: 1rem;
  margin-bottom: 1rem;
}

.prose-content :deep(table) {
  width: 100%;
  margin-top: 1rem;
  margin-bottom: 1rem;
}

.prose-content :deep(table th) {
  background-color: #334155; /* bg-slate-700 */
  padding: 0.5rem;
  text-align: left;
}

.prose-content :deep(table td) {
  border-top: 1px solid #334155; /* border-slate-700 */
  padding: 0.5rem;
}
</style> 