<script setup>
import { useDesktopPet } from '../composables/DesktopPet'

const {
  isHovering,
  hasClipboardWindow,
  hasMenuWindow,
  isDragging,
  handlePointerEnter,
  handlePointerLeave,
  handlePointerDown,
  handleLeftClick,
  handleContextMenu,
  animationFrame
} = useDesktopPet()

// 根据动画帧计算图片路径
const petImagePath = computed(() => {
  return `/animations/${animationFrame.value}.png`
})
</script>

<template>
  <div
    class="desktop-pet"
    :style="{
      cursor: isDragging ? 'grabbing' : 'grab'
    }"
    @pointerenter="handlePointerEnter"
    @pointerleave="handlePointerLeave"
    @pointerdown="handlePointerDown"
    @click="handleLeftClick"
    @contextmenu="handleContextMenu"
  >
    <div class="pet-container">
      <img
        src="/pet.png"
        alt="Desktop Pet"
        draggable="false"
        :class="['pet-image', { 'hover': isHovering, 'has-window': hasMenuWindow }]"
      />
    </div>
  </div>
</template>

<style scoped>
.desktop-pet {
  position: fixed;
  width: 150px;
  height: 150px;
  z-index: 9999;
  user-select: none;
  pointer-events: auto;
  background: transparent;
}

.pet-container {
  width: 100%;
  height: 100%;
  display: flex;
  top: 10px;
  left: 10px;
  background: transparent;
  position: relative;
}

.pet-image {
  width: 130px;
  height: 75px;
  filter: drop-shadow(2px 2px 4px rgba(0, 0, 0, 0.3));
  transition: all 0.3s ease;
  background: transparent;
  flex-shrink: 0;
}

.pet-image.hover {
  transform: scale(1.1);
}

.pet-image.has-window {
  filter: drop-shadow(0 0 8px rgba(74, 144, 226, 0.6));
}
</style>