<script setup>
defineProps({
  name: {
    type: String,
    required: true
  }
});
</script>

<template>
  <div class="wrap">
    <TreeFolder :name="name">
      <slot />
    </TreeFolder>
  </div>
</template>

<style scoped>
.wrap {
  padding: 1rem;
  display: inline-block;
  min-width: 200px;
  border: solid 1px #666;
  border-radius: 0.5rem;
}

.wrap > div {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
</style>
