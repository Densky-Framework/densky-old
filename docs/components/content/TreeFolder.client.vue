<script setup>
import { ref, useSlots, computed } from "vue";

const opened = ref(true);
const slots = useSlots();

function toggle() {
  opened.value = !opened.value;
}

defineProps({
  name: {
    type: String,
    required: true
  },
  diff: {
    type: String,
    required: false,
    validation(val) {
      return ["added", "removed", "ignored"].includes(val);
    }
  }
});

function hasSlots() {
  return slots.default?.().length > 0;
}
</script>

<template>
  <div class="folder" :class='{ "diff-add": diff === "added", "diff-remove": diff === "removed", "ignored": diff === "ignored" }'>
    <button class="folder-name" @click="hasSlots() && toggle()">
      <Icon :name='opened && hasSlots() ? "ph:folder-open" : "ph:folder"' />
      {{ name }}
    </button>
    <div class="folder-content" v-show="opened" v-if="hasSlots()">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.folder {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.folder-name {
  display: flex;
  gap: 0.75rem;
  line-height: 1;
  align-items: center;
  font-weight: 200;
  font-size: 0.9rem;
}

.folder-content {
  margin-left: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: .75rem;
}

.diff-add {
  color: #2D7;
}
</style>
