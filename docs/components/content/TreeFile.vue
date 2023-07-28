<script setup lang="ts">
import { useSlots } from "vue";
const slots = useSlots();

const map = {
  "ts": "vscode-icons:file-type-typescript-official"
} as const;
const document = "ic:outline-insert-drive-file";

const props = defineProps({
  icon: {
    type: String,
    required: false
  },
  link: {
    type: String,
    required: false
  },
  diff: {
    type: String,
    required: false,
    validation(val) {
      return ["added", "removed", "ignored"].includes(val);
    }
  }
});

function calculateIcon() {
  if (props.icon) return props.icon;

  const filename = slots.default?.()?.[0]?.children;
  const extension = filename.split(".").slice(-1)[0];

  return map[extension] ?? document;
}
</script>

<template>
  <NuxtLink v-if="link" :to="link" class="file">
    <Icon :name="calculateIcon()" />
    <slot />
  </NuxtLink>
  <div v-else class="file" :class='{ "diff-add": diff === "added", "diff-remove": diff === "removed", "ignored": diff === "ignored" }'>
    <Icon :name="calculateIcon()" />
    <slot />
  </div>
</template>

<style scoped>
.file {
  display: flex;
  gap: 0.75rem;
  line-height: 1;
  align-items: center;
  font-weight: 200;
  font-size: 0.9rem;
}

.diff-add {
  color: #2D7;
}
</style>
